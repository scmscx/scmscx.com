//! The web layer: request handlers, shared application state, and the axum
//! server bootstrap.
//!
//! This file used to hold every actix-web handler. During the actix -> axum
//! migration its contents were ported in place: the handlers, the shared state
//! types (`ManifestChunk`, `BackblazeAuth`, `get_auth`), and the router /
//! `start()` wiring all still live here. Cross-cutting axum glue that never
//! belonged to a single handler (the `Pool` alias, the `MaybeUser` extractor,
//! cookie helpers, real-client-IP resolution) lives in `webutil`.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Extension, Path, Request};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router, ServiceExt};
use backblaze::api::{b2_authorize_account, b2_download_file_by_name, B2AuthorizeAccount};
use bwcommon::MyError;
use common::gsfs::gsfs_get_mapblob;
use common::{register_counter, register_gauge};
use futures::lock::Mutex;
use futures::StreamExt;
use handlebars::DirectorySourceOptions;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tower::Layer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::GovernorLayer;
use tower_http::compression::CompressionLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::services::ServeDir;
use tracing::{error, info};

use crate::pumpers::{start_backblaze_pumper, start_gsfs_pumper};
use crate::ratelimit::UsernameLoginLimiter;
use crate::util::{finalize_hash_of_hasher, is_dev_mode};
use crate::webutil::{MaybeUser, Pool};
use crate::{api, db, hacks, middleware as mw, static_pages, uiv2};

// Shared, cheaply-cloneable handles injected into handlers as `Extension`s.
pub type Manifest = Arc<std::collections::HashMap<String, ManifestChunk>>;
pub type Handlebars = Arc<handlebars::Handlebars<'static>>;
pub type BackblazeAuthState = Arc<Mutex<BackblazeAuth>>;

#[derive(Clone, Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ManifestChunk {
    pub file: String,
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub src: String,
    #[allow(dead_code)]
    pub isEntry: Option<bool>,
    pub css: Option<Vec<String>>,
}

#[derive(Default)]
pub struct BackblazeAuth {
    pub version: usize,
    pub auth: Option<B2AuthorizeAccount>,
}

pub async fn get_auth(
    client: &reqwest::Client,
    backblaze_auth: &Mutex<BackblazeAuth>,
    bad_version: Option<usize>,
) -> Result<(usize, B2AuthorizeAccount)> {
    let mut lock = backblaze_auth.lock().await;

    let mut reacquire = false;

    if let Some(bv) = bad_version {
        if lock.version <= bv {
            reacquire = true;
            lock.version = bv;
        }
    }

    if lock.auth.is_none() || reacquire {
        let auth = b2_authorize_account(
            client,
            &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
            &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
        )
        .await;
        register_counter!(
            "scmscx",
            backblaze_auth,
            "Backblaze B2 authorize-account calls, by result",
            result = if auth.is_ok() { "ok" } else { "error" }
        )
        .inc();
        lock.auth = Some(auth?);

        lock.version = lock.version.checked_add(1).unwrap();
    }

    Ok((lock.version, lock.auth.clone().unwrap()))
}

pub async fn get_map(
    Extension(pool): Extension<Pool>,
    Extension(backblaze_auth): Extension<BackblazeAuthState>,
    Extension(reqwest_client): Extension<reqwest::Client>,
    headers: HeaderMap,
    Path((mapblob_hash,)): Path<(String,)>,
) -> Result<Response, MyError> {
    {
        let mapblob_hash = mapblob_hash.clone();
        if let Some(useragent) = headers.get("user-agent") {
            if let Ok(useragent) = useragent.to_str() {
                if !useragent.contains("norecord") {
                    let time_since_epoch = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs() as i64;

                    let con = pool.get().await?;
                    let rows = con.execute(
                            "update map set downloads = downloads + 1, last_downloaded = $1 where mapblob2 = $2", &[&time_since_epoch, &mapblob_hash]).await?;
                    (|| {
                        anyhow::ensure!(rows == 1);
                        anyhow::Ok(())
                    })()?;
                }
            }
        }
    }

    const MAPBLOB_BUCKET_NAME: &str = "seventyseven-mapblob";
    let client = Client::new();

    let mut retries_remaining = 5;
    let mut bad_version = None;

    if let Ok(endpoint) = std::env::var("GSFSFE_ENDPOINT") {
        match gsfs_get_mapblob(&reqwest_client, &endpoint, &mapblob_hash).await {
            Ok(mut stream) => {
                register_counter!(
                    "scmscx",
                    map_download,
                    "Map blob download attempts, by source that served the blob",
                    source = "gsfs"
                )
                .inc();
                return Ok(IntoResponse::into_response((
                    [(header::CONTENT_TYPE, "application/octet-stream")],
                    Body::from_stream(async_stream::stream! {
                        use sha2::Digest;
                        let mut hasher = sha2::Sha256::new();
                        let bytes_total = register_counter!(
                            "scmscx",
                            map_download_bytes,
                            "Total bytes streamed to clients for map downloads, by source",
                            source = "gsfs"
                        );

                        while let Some(chunk) = stream.next().await {
                            let chunk = chunk?;
                            bytes_total.inc_by(chunk.len() as u64);
                            hasher.update(&chunk);
                            yield Result::<_, anyhow::Error>::Ok(chunk);
                        }

                        if finalize_hash_of_hasher(hasher) != mapblob_hash {
                            yield Err(anyhow::anyhow!("Hash mismatch"));
                        }
                    }),
                )));
            }
            Err(error) => {
                error!("Failed to download from gsfs: {}", error);
            }
        }
    }

    while retries_remaining > 0 {
        let (version, api_info) = get_auth(&client, &backblaze_auth, bad_version).await?;

        retries_remaining -= 1;

        match b2_download_file_by_name(
            &client,
            &api_info,
            MAPBLOB_BUCKET_NAME,
            mapblob_hash.as_str(),
        )
        .await
        {
            Err(e) => {
                error!("Failed to download from backblaze: {}", e);
                bad_version = Some(version);
            }
            Ok(mut stream) => {
                register_counter!(
                    "scmscx",
                    map_download,
                    "Map blob download attempts, by source that served the blob",
                    source = "backblaze"
                )
                .inc();
                tokio::fs::create_dir_all("./pending/downloading").await?;

                let temp_filename = format!(
                    "./pending/downloading/{}.scx",
                    uuid::Uuid::new_v4().as_simple()
                );
                let mut temp_file = tokio::fs::File::create_new(&temp_filename).await;

                return Ok(IntoResponse::into_response((
                    [(header::CONTENT_TYPE, "application/octet-stream")],
                    Body::from_stream(async_stream::stream! {
                        use sha2::Digest;
                        let mut hasher = sha2::Sha256::new();
                        let bytes_total = register_counter!(
                            "scmscx",
                            map_download_bytes,
                            "Total bytes streamed to clients for map downloads, by source",
                            source = "backblaze"
                        );

                        while let Some(chunk) = stream.next().await {
                            let chunk = chunk?;
                            bytes_total.inc_by(chunk.len() as u64);
                            if let Ok(temp) = &mut temp_file {
                                if let Err(e) = temp.write_all(&chunk).await {
                                    error!("Failed to write to temp file: {e}, temp_filename: {temp_filename}");
                                    temp_file = Err(std::io::Error::from(std::io::ErrorKind::Other));
                                } else {
                                    hasher.update(&chunk);
                                }
                            }
                            yield Result::<_, anyhow::Error>::Ok(chunk);
                        }

                        if let Err(e) = tokio::fs::remove_file(&temp_filename).await {
                            error!("Failed to remove temp file: {e}, temp_filename: {temp_filename}");
                        }
                    }),
                )));
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    register_counter!(
        "scmscx",
        map_download,
        "Map blob download attempts, by source that served the blob",
        source = "failed"
    )
    .inc();
    Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub async fn get_replay(
    Extension(pool): Extension<Pool>,
    Path((replay_id,)): Path<(i64,)>,
) -> Result<Response, MyError> {
    let replay_blob =
        pool.get().await?
        .query_one("select replayblob.data from replay join replayblob on replayblob.hash = replay.hash where replay.id = $1", &[&replay_id])
        .await?.try_get::<_, Vec<u8>>(0)?;

    Ok(IntoResponse::into_response((
        [(header::CONTENT_TYPE, "application/octet-stream")],
        replay_blob,
    )))
}

pub async fn recent_activity(Extension(pool): Extension<Pool>) -> Result<Response, MyError> {
    let replay_activity = {
        let conn = pool.get().await?;
        let mut v = Vec::new();

        for row in &conn
            .query(
                "
                select replay.id, denorm_scenario, account.username, replay.uploaded_time
                from replay
                join account on account.id = uploaded_by
                where uploaded_by != 10
                order by uploaded_time desc
                limit 2000",
                &[],
            )
            .await?
        {
            v.push((
                row.try_get::<_, i64>(0)?,
                encoding_rs::UTF_8
                    .decode(row.try_get::<_, Vec<u8>>(1)?.as_slice())
                    .0
                    .to_string(),
                row.try_get::<_, String>(2)?,
                row.try_get::<_, i64>(3)?,
            ));
        }

        v
    };

    let map_activity = {
        let mut v = Vec::new();
        let conn = pool.get().await?;

        for row in &conn
            .query(
                "
            select map.id, denorm_scenario, account.username, uploaded_time
            from map
            join account on account.id = uploaded_by
            where uploaded_by != 10 and nsfw = false and unfinished = false
            order by uploaded_time desc
            limit 3000",
                &[],
            )
            .await?
        {
            v.push((
                bwcommon::get_web_id_from_db_id(
                    row.try_get::<_, i64>(0)?,
                    crate::util::SEED_MAP_ID,
                )?,
                row.try_get::<_, String>(1)?,
                row.try_get::<_, String>(2)?,
                row.try_get::<_, i64>(3)?,
            ));
        }
        v
    };

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum TypeOfActivity {
        UploadReplay { replay_id: i64, scenario: String },
        UploadMap { map_id: String, scenario: String },
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ActivityInfo {
        username: String,
        time: i64,
        activity: TypeOfActivity,
    }

    let mut ret = Vec::new();

    for i in replay_activity {
        ret.push(ActivityInfo {
            username: i.2,
            time: i.3,
            activity: TypeOfActivity::UploadReplay {
                replay_id: i.0,
                scenario: i.1,
            },
        });
    }

    for i in map_activity {
        ret.push(ActivityInfo {
            username: i.2,
            time: i.3,
            activity: TypeOfActivity::UploadMap {
                map_id: i.0,
                scenario: i.1,
            },
        });
    }

    ret.sort_by(|a, b| a.time.cmp(&b.time).reverse());

    Ok(Json(ret).into_response())
}

enum ChkAccess {
    Allowed,
    NotFound,
    Unauthorized,
}

async fn check_chk_access(
    pool: &Pool,
    chk_id: &str,
    user_id: Option<i64>,
) -> Result<ChkAccess, anyhow::Error> {
    // A chk inherits the most-restrictive flag of any map that references
    // it: if even one map is blackholed, treat the whole chk as blackholed;
    // if any is NSFW, treat the whole chk as NSFW.
    let row = pool
        .get()
        .await?
        .query_one(
            "select
                count(*) > 0 as exists_any,
                coalesce(bool_or(blackholed), false) as any_blackholed,
                coalesce(bool_or(nsfw), false) as any_nsfw
             from map
             where chkblob = $1",
            &[&chk_id],
        )
        .await?;

    let exists_any: bool = row.try_get("exists_any")?;
    let any_blackholed: bool = row.try_get("any_blackholed")?;
    let any_nsfw: bool = row.try_get("any_nsfw")?;

    let is_admin = user_id == Some(4);

    if !exists_any {
        return Ok(ChkAccess::NotFound);
    }
    if any_blackholed && !is_admin {
        return Ok(ChkAccess::NotFound);
    }
    if any_nsfw && user_id.is_none() {
        return Ok(ChkAccess::Unauthorized);
    }

    Ok(ChkAccess::Allowed)
}

pub async fn get_minimap(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((chk_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let user_id = user.id();

    match check_chk_access(&pool, &chk_id, user_id).await? {
        ChkAccess::NotFound => {
            return Ok(
                (StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-cache")]).into_response(),
            );
        }
        ChkAccess::Unauthorized => {
            return Ok((
                StatusCode::UNAUTHORIZED,
                [(header::CACHE_CONTROL, "no-cache")],
            )
                .into_response());
        }
        ChkAccess::Allowed => {}
    }

    let minimap = db::get_minimap(chk_id, pool.clone()).await?.2;

    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=60, immutable"),
        ],
        minimap,
    )
        .into_response())
}

pub async fn get_search_result_popup(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let user_id = user.id();

    let (chkhash, scenario, uploaded_by, nsfw, blackholed) = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select chkblob, denorm_scenario, uploaded_by, nsfw, blackholed
                from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;

        (
            row.try_get::<_, String>("chkblob")?,
            row.try_get::<_, String>("denorm_scenario")?,
            row.try_get::<_, i64>("uploaded_by")?,
            row.try_get::<_, bool>("nsfw")?,
            row.try_get::<_, bool>("blackholed")?,
        )
    };

    if blackholed && user_id != Some(uploaded_by) && user_id != Some(4) {
        return Ok((StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-cache")]).into_response());
    }

    if nsfw && user_id.is_none() {
        return Ok((
            StatusCode::UNAUTHORIZED,
            [(header::CACHE_CONTROL, "no-cache")],
        )
            .into_response());
    }

    let minimap = db::get_minimap(chkhash.clone(), pool.clone()).await?.2;

    use base64::Engine;

    let body = serde_json::to_string(&serde_json::json!({
        "scenario": scenario,
        "minimap": base64::prelude::BASE64_STANDARD.encode(&minimap)
    }))?;

    Ok(IntoResponse::into_response((
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "public, max-age=60, immutable"),
        ],
        body,
    )))
}

pub async fn get_minimap_resized(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((chk_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let user_id = user.id();

    match check_chk_access(&pool, &chk_id, user_id).await? {
        ChkAccess::NotFound => {
            return Ok(
                (StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-cache")]).into_response(),
            );
        }
        ChkAccess::Unauthorized => {
            return Ok((
                StatusCode::UNAUTHORIZED,
                [(header::CACHE_CONTROL, "no-cache")],
            )
                .into_response());
        }
        ChkAccess::Allowed => {}
    }

    use image::ImageDecoder;

    let minimap = db::get_minimap(chk_id.clone(), pool.clone()).await?.2;

    let cursor = std::io::Cursor::new(minimap.as_slice());
    let png = image::codecs::png::PngDecoder::new(cursor)?;
    let (x, y) = png.dimensions();

    let mut image_data = vec![0; png.total_bytes() as usize];

    (|| {
        anyhow::ensure!(png.color_type() == image::ColorType::Rgb8);
        anyhow::Ok(())
    })()?;

    png.read_image(image_data.as_mut_slice())?;

    let image: image::ImageBuffer<image::Rgb<u8>, _> =
        image::ImageBuffer::from_vec(x, y, image_data).unwrap();

    let scaling_factor = std::cmp::min(512 / x, 512 / y);

    let image = image::imageops::resize(
        &image,
        x * scaling_factor,
        y * scaling_factor,
        image::imageops::Nearest,
    );

    let mut png = Vec::<u8>::new();
    use image::ImageEncoder;
    image::codecs::png::PngEncoder::new(&mut png).write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgb8,
    )?;

    Ok(IntoResponse::into_response((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=60, immutable"),
        ],
        png,
    )))
}

pub async fn get_selection_of_random_maps(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let user_id = user.id();
    if !matches!(user_id, Some(4 | 5 | 18 | 24 | 32)) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct MapRow {
        map_id: i64,
        chkhash: String,
    }

    let rows = {
        let rows: Result<Vec<_>, MyError> = {
            let con = pool.get().await?;
            con.query(
                "
               select * from (
                   select map.id, map.chkblob from map
                   where nsfw = false and blackholed = false
                   except
                   select map.id, map.chkblob from map
                   join tagmap on tagmap.map = map.id
                   join tag on tag.id = tagmap.tag
                   where (key = 'minimap_checked' and value = 'true')
               ) a
               where chkblob is not null
               order by random()
               ",
                &[],
            )
            .await?
        }
        .into_iter()
        .map(|x| {
            Ok(MapRow {
                map_id: x.try_get(0)?,
                chkhash: x.try_get(1)?,
            })
        })
        .collect();

        rows?
    };

    Ok(Json(rows).into_response())
}

pub async fn get_selection_of_random_nsfw_maps(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let user_id = user.id();
    if !matches!(user_id, Some(4 | 18 | 24 | 32)) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct MapRow {
        map_id: i64,
        chkhash: String,
    }

    let rows = {
        let rows: Result<Vec<_>, MyError> = {
            let con = pool.get().await?;
            con.query(
                "
                select distinct map.id, map.chkblob
                from map
                where nsfw = false and blackholed = false
                ",
                &[],
            )
            .await?
        }
        .into_iter()
        .map(|x| {
            Ok(MapRow {
                map_id: x.try_get(0)?,
                chkhash: x.try_get(1)?,
            })
        })
        .collect();

        rows?
    };

    Ok(Json(rows).into_response())
}

#[derive(Serialize, Deserialize)]
pub(crate) struct TagPost {
    key: String,
    value: String,
}

pub async fn get_tags(
    Extension(pool): Extension<Pool>,
    _user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let con = pool.get().await?;
    let tags = con
        .query(
            "
            select key, value
            from tagmap
            join tag on tagmap.tag = tag.id
            where tagmap.map = $1",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| {
            anyhow::Ok(TagPost {
                key: row.try_get(0)?,
                value: row.try_get(1)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(tags).into_response())
}

pub async fn set_tags(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
    Json(tags): Json<Vec<TagPost>>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(user_id) = user.id() else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let mut map = std::collections::hash_map::HashMap::new();

    for t in tags {
        map.insert(t.key, t.value);
    }

    let outcome = db::set_tags(map_id, map, user_id, pool).await?;

    match outcome {
        None => Ok(StatusCode::NOT_FOUND.into_response()),
        Some(false) => Ok(StatusCode::FORBIDDEN.into_response()),
        Some(true) => Ok(StatusCode::OK.into_response()),
    }
}

pub async fn add_tags(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
    Json(tags): Json<Vec<TagPost>>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(user_id) = user.id() else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let mut map = std::collections::hash_map::HashMap::new();

    for t in tags {
        map.insert(t.key, t.value);
    }

    let outcome = db::add_tags(map_id, map, user_id, pool).await?;

    match outcome {
        None => Ok(StatusCode::NOT_FOUND.into_response()),
        Some(false) => Ok(StatusCode::FORBIDDEN.into_response()),
        Some(true) => Ok(StatusCode::OK.into_response()),
    }
}

async fn setup_db() -> Result<Pool> {
    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        std::env::var("DB_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .as_str(),
        std::env::var("DB_PORT").unwrap().as_str(),
        std::env::var("DB_USER").unwrap().as_str(),
        std::env::var("DB_PASSWORD").unwrap().as_str(),
        std::env::var("DB_DATABASE")
            .unwrap_or_else(|_| std::env::var("DB_USER").unwrap())
            .as_str(),
    );
    let manager = bb8_postgres::PostgresConnectionManager::new(
        connection_string.parse()?,
        bb8_postgres::tokio_postgres::NoTls,
    );

    let pool = bb8_postgres::bb8::Pool::builder()
        .max_size(
            std::env::var("DB_CONNECTIONS")
                .unwrap_or_else(|_| "16".to_string())
                .parse::<u32>()?,
        )
        .min_idle(Some(1))
        .max_lifetime(Some(std::time::Duration::from_mins(1)))
        .idle_timeout(Some(std::time::Duration::from_secs(30)))
        .test_on_check_out(true)
        .build(manager)
        .await?;

    anyhow::Ok(pool)
}

fn register_handlebars() -> Result<Handlebars> {
    let mut registry = handlebars::Handlebars::new();

    registry.set_strict_mode(true);

    if is_dev_mode() {
        info!("DEV_MODE activated, template hot reloading");
        registry.set_dev_mode(true);
    }

    let mut options = DirectorySourceOptions::default();
    ".hbs".clone_into(&mut options.tpl_extension);
    registry
        .register_templates_directory(
            std::path::Path::new(std::env::var("ROOT_DIR").unwrap().as_str()).join("uiv2"),
            options,
        )
        .unwrap();

    registry
        .register_partial(
            "header.hbs",
            std::fs::read_to_string(
                std::path::Path::new(std::env::var("ROOT_DIR").unwrap().as_str())
                    .join("uiv2/header.hbs"),
            )?
            .as_str(),
        )
        .map_err(|err| anyhow::anyhow!("failed to unwrap. err: {:?}", err))?;

    registry
        .register_partial(
            "body.hbs",
            std::fs::read_to_string(
                std::path::Path::new(std::env::var("ROOT_DIR").unwrap().as_str())
                    .join("uiv2/body.hbs"),
            )?
            .as_str(),
        )
        .map_err(|err| anyhow::anyhow!("failed to unwrap. err: {:?}", err))?;

    Ok(Arc::new(registry))
}

/// Dev-mode fallback: proxy any unmatched request to the local vite server.
async fn dev_proxy(
    Extension(client): Extension<reqwest::Client>,
    req: Request,
) -> Result<Response, MyError> {
    let path_query = req
        .uri()
        .path_and_query()
        .map_or_else(|| req.uri().path(), |pq| pq.as_str());

    let url = format!("http://localhost:3000{path_query}");
    info!("proxying to {}", url);

    let upstream = client.get(&url).send().await?;

    let mut builder = Response::builder().status(upstream.status());
    for (name, value) in upstream.headers() {
        builder = builder.header(name, value);
    }
    Ok(builder
        .body(Body::from_stream(upstream.bytes_stream()))
        .unwrap())
}

pub(crate) async fn start() -> Result<()> {
    // Telemetry: install the panic hook, bring up the Prometheus scrape server
    // (PROMETHEUS_ENDPOINT, required), and start the runtime/process reporter.
    common::telemetry::init(env!("CARGO_PKG_VERSION")).await;

    // Fail loud on a Backblaze misconfiguration rather than silently not
    // uploading maps. Backblaze must be turned off explicitly, never by accident.
    if std::env::var("BACKBLAZE_DISABLED").as_deref() != Ok("true") {
        for var in [
            "BACKBLAZE_KEY_ID",
            "BACKBLAZE_APPLICATION_KEY",
            "BACKBLAZE_MAPBLOB_BUCKET",
        ] {
            std::env::var(var).map_err(|_| {
                anyhow::anyhow!(
                    "{var} not set (set BACKBLAZE_DISABLED=true to run without Backblaze)"
                )
            })?;
        }
    }

    let db_pool = setup_db().await?;

    // Telemetry: start the background reporter for the DB pool gauges.
    {
        let pool = db_pool.clone();
        tokio::spawn(async move {
            loop {
                let state = pool.state();
                for (label, value) in [
                    ("total", state.connections),
                    ("idle", state.idle_connections),
                ] {
                    register_gauge!(
                        "scmscx",
                        db_pool_connections,
                        "bb8 database pool connection count, by state",
                        state = label
                    )
                    .set(i64::from(value));
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    let handlebars = register_handlebars()?;

    let manifest: Manifest = Arc::new(serde_json::from_str::<
        std::collections::HashMap<String, ManifestChunk>,
    >(
        tokio::fs::read_to_string("./dist/.vite/manifest.json")
            .await?
            .as_str(),
    )?);

    let backblaze_auth: BackblazeAuthState = Arc::new(Mutex::new(BackblazeAuth::default()));

    // Pump files up to backblaze
    let reqwest_client = reqwest::Client::new();

    start_backblaze_pumper(reqwest_client.clone()).await?;
    start_gsfs_pumper(reqwest_client.clone()).await?;

    let username_limiter = Arc::new(UsernameLoginLimiter::new());

    // Per-IP rate limits for the auth endpoints (was actix-governor).
    let login_governor = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .period(std::time::Duration::from_secs(3))
            .burst_size(20)
            .finish()
            .expect("valid governor config"),
    );
    let register_governor = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .period(std::time::Duration::from_mins(20))
            .burst_size(3)
            .finish()
            .expect("valid governor config"),
    );

    // DashMap state stores grow per distinct key with no built-in eviction.
    // Periodically drop entries whose GCRA cell has fully refilled.
    {
        let ip_login = login_governor.limiter().clone();
        let ip_register = register_governor.limiter().clone();
        let username = username_limiter.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_mins(1));
            tick.tick().await;
            loop {
                tick.tick().await;
                ip_login.retain_recent();
                ip_login.shrink_to_fit();
                ip_register.retain_recent();
                ip_register.shrink_to_fit();
                username.retain_recent();
            }
        });
    }

    let router = Router::new()
        // core (formerly inline in actix.rs)
        .route("/api/maps/{mapblob_hash}", get(get_map))
        .route("/api/replays/{replay_id}", get(get_replay))
        .route("/api/recent_activity", get(recent_activity))
        .route("/api/minimap/{chk_id}", get(get_minimap))
        .route(
            "/api/search_result_popup/{map_id}",
            get(get_search_result_popup),
        )
        .route("/api/minimap_resized/{chk_id}", get(get_minimap_resized))
        .route(
            "/api/get_selection_of_random_maps",
            get(get_selection_of_random_maps),
        )
        .route(
            "/api/get_selection_of_random_nsfw_maps",
            get(get_selection_of_random_nsfw_maps),
        )
        .route("/api/tags/{map_id}", get(get_tags).post(set_tags))
        .route("/api/addtags/{map_id}", post(add_tags))
        // api
        .route(
            "/api/flags/{map_id}/{flag}",
            get(api::flags::get_flag).post(api::flags::set_flag),
        )
        .route(
            "/api/change-password",
            post(api::change_password::post_handler),
        )
        .route(
            "/api/change-username",
            post(api::change_username::post_handler),
        )
        .route(
            "/api/login",
            post(api::login::post_handler).layer(GovernorLayer::new(login_governor)),
        )
        .route(
            "/api/register",
            post(api::register::post_handler).layer(GovernorLayer::new(register_governor)),
        )
        .route("/api/logout", get(api::logout::handler))
        .route("/sitemap.txt", get(api::sitemap::handler))
        .route("/a.txt", get(api::sitemap::handlera))
        .route("/b.txt", get(api::sitemap::handlerb))
        .route("/c.txt", get(api::sitemap::handlerc))
        .route("/api/chk/strings/{map_id}", get(api::chk::get_chk_strings))
        .route(
            "/api/chk/riff_chunks/{map_id}",
            get(api::chk::get_chk_riff_chunks),
        )
        .route("/api/chk/json/{map_id}", get(api::chk::get_chk_json))
        .route("/api/chk/trig/{map_id}", get(api::chk::get_chk_trig_json))
        .route("/api/chk/mbrf/{map_id}", get(api::chk::get_chk_mbrf_json))
        .route("/api/chk/eups/{map_id}", get(api::chk::get_eups))
        .route("/api/chk/{chk_hash}", get(api::chk::download_chk))
        .route(
            "/api/similar_maps/{map_id}",
            get(api::similar_maps::handler),
        )
        // uiv2 api
        .route("/api/uiv2/featured_maps", get(api::uiv2::featured_maps))
        .route(
            "/api/uiv2/last_viewed_maps",
            get(api::uiv2::last_viewed_maps),
        )
        .route(
            "/api/uiv2/last_downloaded_maps",
            get(api::uiv2::last_downloaded_maps),
        )
        .route(
            "/api/uiv2/last_uploaded_maps",
            get(api::uiv2::last_uploaded_maps),
        )
        .route(
            "/api/uiv2/last_uploaded_replays",
            get(api::uiv2::last_uploaded_replays),
        )
        .route(
            "/api/uiv2/most_viewed_maps",
            get(api::uiv2::most_viewed_maps),
        )
        .route(
            "/api/uiv2/most_downloaded_maps",
            get(api::uiv2::most_downloaded_maps),
        )
        .route("/api/uiv2/minimap/{map_id}", get(api::uiv2::get_minimap))
        .route(
            "/api/uiv2/is_session_valid",
            post(api::uiv2::is_session_valid),
        )
        .route(
            "/api/uiv2/map_info/{map_id}",
            get(api::uiv2::map_info::map_info),
        )
        .route(
            "/api/uiv2/filenames/{map_id}",
            get(api::uiv2::filenames::filenames),
        )
        .route(
            "/api/uiv2/timestamps/{map_id}",
            get(api::uiv2::timestamps::timestamps),
        )
        .route(
            "/api/uiv2/filenames2/{map_id}",
            get(api::uiv2::filenames2::filenames2),
        )
        .route(
            "/api/uiv2/replays/{map_id}",
            get(api::uiv2::replays::replays),
        )
        .route("/api/uiv2/units/{map_id}", get(api::uiv2::units::units))
        .route(
            "/api/uiv2/search/{query}",
            get(api::uiv2::search::search_query),
        )
        .route("/api/uiv2/search", get(api::uiv2::search::search))
        .route("/api/uiv2/img/{map_id}", get(api::uiv2::get_map_image))
        .route("/api/uiv2/random/{query}", get(api::random::handler))
        .route("/api/uiv2/random", get(api::random::handler_noquery))
        .route("/api/uiv2/upload-map", post(api::uiv2::upload::upload_map))
        .route("/api/uiv2/logout", get(api::uiv2::logout::logout2))
        // uiv2 ssr
        .route("/", get(uiv2::index::index))
        .route("/search", get(uiv2::index::search_no_query))
        .route("/search/{query}", get(uiv2::index::search_query))
        .route("/map/{map_id}", get(uiv2::index::map))
        .route("/upload", get(uiv2::index::upload))
        .route("/about", get(uiv2::index::about))
        .route("/user/{username}", get(uiv2::index::user))
        .route("/login", get(uiv2::index::login))
        .route("/moderation", get(uiv2::index::moderation))
        .route("/site.webmanifest", get(uiv2::index::webmanifest))
        // hacks
        .route("/api/denormalize/{map_id}", get(hacks::denormalize))
        .route("/api/denormalize_all", get(hacks::denormalize_all))
        // static pages
        .route("/map", get(static_pages::redirect_map))
        .route("/replay", get(static_pages::redirect_replay))
        // static assets
        .nest_service("/assets", ServeDir::new("./dist/assets"))
        .nest_service("/uiv2/assets", ServeDir::new("./dist/assets"))
        // Upload can be large; lift axum's 2 MB default body cap.
        .layer(DefaultBodyLimit::disable());

    // Fallback: dev proxy to the vite server, or the public/ static dir in prod.
    let router = if is_dev_mode() {
        info!("dev mode active, adding local proxy to localhost:3000");
        router.fallback(dev_proxy)
    } else {
        router.fallback_service(ServeDir::new("./public").append_index_html_on_directories(true))
    };

    // Shared state as request extensions, plus the middleware stack. Order is
    // outermost-first: TraceID sees every request first and logs the final
    // status last; UserSession/TrackingAnalytics run before PostgresLogging so
    // their extensions are populated when it captures them.
    let router = router.layer(
        tower::ServiceBuilder::new()
            .layer(Extension(db_pool.clone()))
            .layer(Extension(reqwest_client.clone()))
            .layer(Extension(handlebars))
            .layer(Extension(manifest))
            .layer(Extension(backblaze_auth))
            .layer(Extension(username_limiter))
            .layer(axum::middleware::from_fn(mw::trace_id))
            .layer(axum::middleware::from_fn(mw::tracking_analytics))
            .layer(axum::middleware::from_fn(mw::language))
            .layer(axum::middleware::from_fn({
                let pool = db_pool.clone();
                move |req, next| mw::user_session(pool.clone(), req, next)
            }))
            .layer(axum::middleware::from_fn({
                let pool = db_pool.clone();
                move |req, next| mw::postgres_logging(pool.clone(), req, next)
            }))
            .layer(axum::middleware::from_fn(mw::cache_html))
            .layer(axum::middleware::from_fn(mw::metrics))
            .layer(CompressionLayer::new()),
    );

    // NormalizePath must wrap the router (before routing) so trailing slashes
    // are trimmed prior to route matching.
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    // Prefer an inherited, already-bound listener (`BIND_FD`) over binding
    // `BIND_ADDR`. The E2E harness hands the socket down this way so the port is
    // chosen and held race-free — no ephemeral-port grab/close/re-bind window.
    // See `common::telemetry::take_listener_from_env`.
    let listener = if let Some(std_listener) = common::telemetry::take_listener_from_env("BIND_FD")
    {
        std_listener.set_nonblocking(true)?;
        tokio::net::TcpListener::from_std(std_listener)?
    } else {
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        tokio::net::TcpListener::bind(&bind_addr).await?
    };
    info!("listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    )
    .await?;

    anyhow::Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_chunk_deserializes_from_vite_manifest() {
        // A realistic slice of dist/.vite/manifest.json.
        let json = r#"{
            "index.html": {
                "file": "assets/index-abc123.js",
                "name": "index",
                "src": "index.html",
                "isEntry": true,
                "css": ["assets/index-def456.css"]
            }
        }"#;

        let map: std::collections::HashMap<String, ManifestChunk> =
            serde_json::from_str(json).unwrap();
        let chunk = &map["index.html"];
        assert_eq!(chunk.file, "assets/index-abc123.js");
        assert_eq!(chunk.name.as_deref(), Some("index"));
        assert_eq!(chunk.isEntry, Some(true));
        assert_eq!(
            chunk.css.as_deref(),
            Some(&["assets/index-def456.css".to_string()][..])
        );
    }

    #[test]
    fn manifest_chunk_tolerates_missing_optional_fields() {
        // Non-entry chunks omit isEntry/css.
        let json = r#"{ "file": "assets/x.js", "src": "x.ts" }"#;
        let chunk: ManifestChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.file, "assets/x.js");
        assert!(chunk.name.is_none());
        assert!(chunk.isEntry.is_none());
        assert!(chunk.css.is_none());
    }

    #[test]
    fn backblaze_auth_default_is_empty() {
        let auth = BackblazeAuth::default();
        assert_eq!(auth.version, 0);
        assert!(auth.auth.is_none());
    }
}

#[cfg(test)]
mod router_tests {
    //! Wiring tests for the router-level tower layers. `start()` itself needs a
    //! DB/manifest/env and can't run here, but the `NormalizePathLayer` wrapping
    //! the router (see the comment above) is a pure routing concern we can lock
    //! down against version bumps of axum/tower-http.

    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use tower::{Layer, ServiceExt};
    use tower_http::normalize_path::NormalizePathLayer;

    async fn status(uri: &str) -> StatusCode {
        let router = Router::new()
            .route("/about", get(|| async { "about" }))
            .route("/map/{id}", get(|| async { "map" }));
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        app.oneshot(HttpRequest::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn trailing_slash_is_trimmed_before_routing() {
        // The whole reason NormalizePath wraps (rather than layers inside) the
        // router: `/about/` must match the `/about` route.
        assert_eq!(status("/about").await, StatusCode::OK);
        assert_eq!(status("/about/").await, StatusCode::OK);
        assert_eq!(status("/map/5/").await, StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_is_404() {
        assert_eq!(status("/does-not-exist").await, StatusCode::NOT_FOUND);
    }
}
