//! Core map/replay/minimap/tags endpoints, ported out of the old `actix.rs`.

use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::with_logging_info;
use bwcommon::{ApiSpecificInfoForLogging, MyError};
use common::gsfs::gsfs_get_mapblob;
use common::register_counter;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tracing::error;

use backblaze::api::b2_download_file_by_name;

use crate::db;
use crate::state::{get_auth, BackblazeAuthState};
use crate::util::finalize_hash_of_hasher;
use crate::webutil::{MaybeUser, Pool};

pub async fn get_map(
    Extension(pool): Extension<Pool>,
    Extension(backblaze_auth): Extension<BackblazeAuthState>,
    Extension(reqwest_client): Extension<reqwest::Client>,
    headers: HeaderMap,
    Path((mapblob_hash,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let info = ApiSpecificInfoForLogging {
        mapblob_hash: Some(mapblob_hash.clone()),
        ..Default::default()
    };

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
                return Ok(with_logging_info(
                    info,
                    (
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
                    ),
                ));
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

                return Ok(with_logging_info(
                    info,
                    (
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
                    ),
                ));
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
    Ok(with_logging_info(info, StatusCode::INTERNAL_SERVER_ERROR))
}

pub async fn get_replay(
    Extension(pool): Extension<Pool>,
    Path((replay_id,)): Path<(i64,)>,
) -> Result<Response, MyError> {
    let replay_blob =
        pool.get().await?
        .query_one("select replayblob.data from replay join replayblob on replayblob.hash = replay.hash where replay.id = $1", &[&replay_id])
        .await?.try_get::<_, Vec<u8>>(0)?;

    let info = ApiSpecificInfoForLogging {
        replay_id: Some(replay_id),
        ..Default::default()
    };

    Ok(with_logging_info(
        info,
        (
            [(header::CONTENT_TYPE, "application/octet-stream")],
            replay_blob,
        ),
    ))
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

    let info = ApiSpecificInfoForLogging {
        map_id: Some(map_id),
        chk_hash: Some(chkhash),
        ..Default::default()
    };

    use base64::Engine;

    let body = serde_json::to_string(&serde_json::json!({
        "scenario": scenario,
        "minimap": base64::prelude::BASE64_STANDARD.encode(&minimap)
    }))?;

    Ok(with_logging_info(
        info,
        (
            [
                (header::CONTENT_TYPE, "application/json"),
                (header::CACHE_CONTROL, "public, max-age=60, immutable"),
            ],
            body,
        ),
    ))
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

    let info = ApiSpecificInfoForLogging {
        chk_hash: Some(chk_id),
        ..Default::default()
    };

    Ok(with_logging_info(
        info,
        (
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "public, max-age=60, immutable"),
            ],
            png,
        ),
    ))
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
    user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let user_id = user.id();

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

    let info = ApiSpecificInfoForLogging {
        user_id,
        map_id: Some(map_id),
        ..Default::default()
    };

    Ok(with_logging_info(info, Json(tags)))
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

    let info = ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        map_id: Some(map_id),
        ..Default::default()
    };

    match outcome {
        None => Ok(with_logging_info(info, StatusCode::NOT_FOUND)),
        Some(false) => Ok(with_logging_info(info, StatusCode::FORBIDDEN)),
        Some(true) => Ok(with_logging_info(info, StatusCode::OK)),
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

    let info = ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        map_id: Some(map_id),
        ..Default::default()
    };

    match outcome {
        None => Ok(with_logging_info(info, StatusCode::NOT_FOUND)),
        Some(false) => Ok(with_logging_info(info, StatusCode::FORBIDDEN)),
        Some(true) => Ok(with_logging_info(info, StatusCode::OK)),
    }
}
