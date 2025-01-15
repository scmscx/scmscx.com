use crate::middleware::UserSession;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::{get, middleware, post, web, App, HttpResponse, Responder};

//use anyhow::Context;

//use crossbeam_channel::Sender;
//use rustls_pemfile::{certs};

//use image::DynamicImage;

use serde::{Deserialize, Serialize};

use crate::db;
use crate::hacks;

use bwcommon::insert_extension;
use bwcommon::{ApiSpecificInfoForLogging, MyError};

use anyhow::Result;
use backblaze::api::{b2_authorize_account, b2_download_file_by_name};
use handlebars::{DirectorySourceOptions, Handlebars};
// use r2d2_postgres::r2d2;
use crate::util::is_dev_mode;
use actix_files::Files;
use async_stream::stream;
use backblaze::api::b2_get_upload_url;
use backblaze::api::b2_upload_file;
use backblaze::api::B2AuthorizeAccount;
use bytes::BytesMut;
use futures::lock::Mutex;
use reqwest::Client;
use reqwest::ClientBuilder;
use std::collections::HashMap;
use tokio::io::AsyncReadExt;
use tracing::{error, info};

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
pub(crate) struct BackblazeAuth {
    pub(crate) version: usize,
    pub(crate) auth: Option<B2AuthorizeAccount>,
}

pub async fn get_auth(
    client: &reqwest::Client,
    backblaze_auth: web::Data<Mutex<BackblazeAuth>>,
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
        lock.auth = Some(
            b2_authorize_account(
                &client,
                &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
                &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
            )
            .await?,
        );

        lock.version = lock.version.checked_add(1).unwrap();
    }

    Ok((lock.version, lock.auth.clone().unwrap()))
}

#[get("/api/maps/{mapblob_hash}")]
async fn get_map(
    req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    backblaze_auth: web::Data<Mutex<BackblazeAuth>>,
) -> Result<impl Responder, MyError> {
    let (mapblob_hash,) = path.into_inner();

    let info = ApiSpecificInfoForLogging {
        mapblob_hash: Some(mapblob_hash.clone()),
        ..Default::default()
    };

    {
        let mapblob_hash = mapblob_hash.clone();
        if let Some(useragent) = req.headers().get("user-agent") {
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

    const MAPBLOB_BUCKET_NAME: &'static str = "seventyseven-mapblob";
    let client = Client::new();

    let mut retries_remaining = 5;
    let mut bad_version = None;

    while retries_remaining > 0 {
        let (version, api_info) = get_auth(&client, backblaze_auth.clone(), bad_version).await?;

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
            Ok(stream) => {
                return Ok(insert_extension(HttpResponse::Ok(), info)
                    .content_type("application/octet-stream")
                    .streaming(stream));
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    return Ok(insert_extension(HttpResponse::InternalServerError(), info).finish());
}

// #[get("/api/replay_dump/{replay_id}")]
// async fn get_replay_dump(
//     path: web::Path<(i64,)>,
//     pool: web::Data<
//         bb8_postgres::bb8::Pool<
//             bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
//         >,
//     >,
// ) -> Result<impl Responder, bwcommon::MyError> {
//     let (replay_id,) = path.into_inner();
//     let (uploaded_by, uploaded_time, replay_blob, denorm_scenario, chkhash, map_id) = {
//         let r = pool.get().await?.query_one("
//                 select account.username, replay.uploaded_time, replayblob.data, replay.denorm_scenario, replay.chkhash, map.id
//                 from replay
//                 join replayblob on replayblob.hash = replay.hash
//                 join account on account.id = uploaded_by
//                 full outer join map on map.chkblob = replay.chkhash
//                 where replay.id = $1", &[&replay_id]).await?;

//         (
//             r.try_get::<_, String>(0)?,
//             r.try_get::<_, i64>(1)?,
//             r.try_get::<_, Vec<u8>>(2)?,
//             r.try_get::<_, Vec<u8>>(3)?,
//             r.try_get::<_, String>(4)?,
//             r.try_get::<_, i64>(5)?,
//         )
//     };

//     #[derive(Debug, Serialize, Deserialize)]
//     struct ReplayInfo {
//         uploaded_by: String,
//         uploaded_time: i64,
//         replay_header: bwreplay::ReplayHeader,
//         denorm_scenario: String,
//         chkhash: String,
//         map_id: i64,
//     }

//     let ret = ReplayInfo {
//         uploaded_by,
//         uploaded_time,
//         replay_header: bwreplay::parse_replay_blob(replay_blob.as_slice())?.header,
//         denorm_scenario: encoding_rs::UTF_8
//             .decode(denorm_scenario.as_slice())
//             .0
//             .to_string(),
//         chkhash: chkhash,
//         map_id: map_id,
//     };

//     let info = ApiSpecificInfoForLogging {
//         replay_id: Some(replay_id),
//         ..Default::default()
//     };

//     Ok(insert_extension(HttpResponse::Ok(), info)
//         .content_type("application/json")
//         .body(serde_json::to_string(&ret)?))
// }

#[get("/api/replays/{replay_id}")]
async fn get_replay(
    path: web::Path<(i64,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let (replay_id,) = path.into_inner();

    let replay_blob =
        pool.get().await?
        .query_one("select replayblob.data from replay join replayblob on replayblob.hash = replay.hash where replay.id = $1", &[&replay_id])
        .await?.try_get::<_, Vec<u8>>(0)?;

    let info = ApiSpecificInfoForLogging {
        replay_id: Some(replay_id),
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/octet-stream")
        .body(replay_blob))
}

#[get("/api/recent_activity")]
async fn recent_activity(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let replay_activity = {
        let conn = pool.get().await?;
        let mut v = Vec::new();

        for row in conn
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
            .iter()
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

        for row in conn
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
            .iter()
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

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&ret).unwrap()))
}

#[get("/api/minimap/{chk_id}")]
async fn get_minimap(
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let (chk_id,) = path.into_inner();

    let minimap = db::get_minimap(chk_id, (**pool).clone()).await?.2;

    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(minimap)
        .customize()
        .insert_header(("Cache-Control", "public, max-age=60, immutable")))
}

#[get("/api/search_result_popup/{map_id}")]
async fn get_search_result_popup(
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let (chkhash, scenario) = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select chkblob, denorm_scenario
                from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;

        (row.try_get::<_, String>(0)?, row.try_get::<_, String>(1)?)
    };

    let minimap = db::get_minimap(chkhash.clone(), (**pool).clone()).await?.2;

    let info = ApiSpecificInfoForLogging {
        map_id: Some(map_id),
        chk_hash: Some(chkhash),
        ..Default::default()
    };

    use base64::Engine;

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&serde_json::json!({
            "scenario": scenario,
            "minimap": base64::prelude::BASE64_STANDARD.encode(&minimap)
        }))?)
        .customize()
        .insert_header(("Cache-Control", "public, max-age=60, immutable")))
}

#[get("/api/minimap_resized/{chk_id}")]
async fn get_minimap_resized(
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let (chk_id,) = path.into_inner();

    use image::ImageDecoder;

    let minimap = db::get_minimap(chk_id.clone(), (**pool).clone()).await?.2;

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

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("image/png")
        .body(png)
        .customize()
        .insert_header(("Cache-Control", "public, max-age=60, immutable")))
}

// #[get("/api/random_bound")]
// async fn get_random_bound(
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl Responder, bwcommon::MyError> {
//     let map_id: i64 = {
//         web::block(move || {
//             let mut con = pool.get().unwrap();
//             con.query_one(
//                 "
//                 select mapid from (
//                     select distinct map as mapid from stringmap2
//                     where 'bound' <% data and ((scenario_name = true) or (scenario_description = true) or (force_names = true) or (file_names = true))
//                 ) as sq1
//                 join map on map.id = mapid
//                 where nsfw = false and broken = false and outdated = false and unfinished = false
//                 order by random()
//                 limit 1",
//                 &[],
//             )
//         })
//         .await??
//         .get(0)
//     };

//     let info = ApiSpecificInfoForLogging {
//         map_id: Some(map_id),
//         ..Default::default()
//     };

//     Ok(insert_extension(HttpResponse::TemporaryRedirect(), info)
//         .finish()
//         .customize()
//         .insert_header(("Location", format!("/map/{}", map_id))))
// }

// #[get("/api/random")]
// async fn get_random(
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl Responder, bwcommon::MyError> {
//     let map_id: i64 = {
//         web::block(move || {
//             let mut con = pool.get().unwrap();

//             con.query_one(
//                 "
//                select map.id from map
//                where nsfw = false and outdated = false and unfinished = false and broken = false
//                order by random()
//                limit 1",
//                 &[],
//             )
//         })
//         .await??
//         .get(0)
//     };

//     let info = ApiSpecificInfoForLogging {
//         map_id: Some(map_id),
//         ..Default::default()
//     };

//     Ok(insert_extension(HttpResponse::TemporaryRedirect(), info)
//         .finish()
//         .customize()
//         .insert_header(("Location", format!("/map/{}", map_id))))
// }

#[get("/api/get_selection_of_random_maps")]
async fn get_selection_of_random_maps(
    req: HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let _ = if let Some(user_id) = bwcommon::check_auth4(&req, (**pool).clone()).await? {
        if user_id == 4 || user_id == 5 || user_id == 18 || user_id == 24 || user_id == 32 {
            user_id
        } else {
            return Ok(HttpResponse::Unauthorized().finish());
        }
    } else {
        return Ok(HttpResponse::Unauthorized().finish());
    };

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

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&rows).unwrap()))
}

#[get("/api/get_selection_of_random_nsfw_maps")]
async fn get_selection_of_random_nsfw_maps(
    req: HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let _ = if let Some(user_id) = bwcommon::check_auth4(&req, (**pool).clone()).await? {
        if user_id == 4 || user_id == 18 || user_id == 24 || user_id == 32 {
            user_id
        } else {
            return Ok(HttpResponse::Unauthorized().finish());
        }
    } else {
        return Ok(HttpResponse::Unauthorized().finish());
    };

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

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&rows).unwrap()))
}

#[derive(Serialize, Deserialize)]
struct TagPost {
    key: String,
    value: String,
}

#[get("/api/tags/{map_id}")]
async fn get_tags(
    req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let user_id = req.extensions().get::<UserSession>().map(|x| x.id);

    let pool = pool.clone();

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
        user_id: user_id,
        map_id: Some(map_id),
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info).body(serde_json::to_string(&tags)?))
}

#[post("/api/tags/{map_id}")]
async fn set_tags(
    req: HttpRequest,
    path: web::Path<(String,)>,
    info: web::Json<Vec<TagPost>>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let user_id = if let Some(user_id) = bwcommon::check_auth4(&req, (**pool).clone()).await? {
        user_id
    } else {
        return Ok(HttpResponse::Unauthorized().finish());
    };

    let mut map = std::collections::hash_map::HashMap::new();

    for t in info.0 {
        map.insert(t.key, t.value);
    }

    db::set_tags(map_id, map, Some(user_id), pool).await?;

    let info = ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        map_id: Some(map_id),
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info).finish())
}

#[post("/api/addtags/{map_id}")]
async fn add_tags(
    req: HttpRequest,
    path: web::Path<(String,)>,
    info: web::Json<Vec<TagPost>>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let user_id = if let Some(user_id) = bwcommon::check_auth4(&req, (**pool).clone()).await? {
        user_id
    } else {
        return Ok(HttpResponse::Unauthorized().finish());
    };

    let mut map = std::collections::hash_map::HashMap::new();

    for t in info.0 {
        map.insert(t.key, t.value);
    }

    db::add_tags(map_id, map, pool).await?;

    let info = ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        map_id: Some(map_id),
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info).finish())
}

// #[post("/api/upload-replay")]
// async fn upload_replay(
//     req: HttpRequest,
//     mut payload: actix_multipart::Multipart,
//     pool: web::Data<
//         bb8_postgres::bb8::Pool<
//             bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
//         >,
//     >,
// ) -> Result<impl Responder, bwcommon::MyError> {
//     use std::io::Write;

//     let user_id = if let Some(user_id) = bwcommon::check_auth4(&req, (**pool).clone()).await? {
//         user_id
//     } else {
//         return Ok(HttpResponse::Unauthorized().finish().customize());
//     };

//     let mut dirpath;
//     let mut filepath = "/xxx".to_string();
//     // iterate over multipart stream
//     while let Ok(Some(mut field)) = payload.try_next().await {
//         let content_type = field
//             .content_disposition()
//             .ok_or(anyhow::anyhow!("no content disposition"))?;
//         let filename = content_type.get_filename().unwrap();
//         dirpath = format!(
//             "/tmp/replays/{}",
//             uuid::Uuid::new_v4().as_simple().to_string()
//         );
//         filepath = format!("{}/{}", dirpath, sanitize_filename::sanitize(&filename));

//         // File::create is blocking operation, use threadpool
//         web::block(|| std::fs::create_dir_all(dirpath)).await??;

//         let c = filepath.clone();
//         // File::create is blocking operation, use threadpool
//         let mut f = web::block(|| std::fs::File::create(c)).await??;

//         // Field in turn is stream of *Bytes* object
//         while let Some(chunk) = field.next().await {
//             let data = chunk.unwrap();
//             // filesystem operations are blocking, we have to use threadpool
//             f = web::block(move || f.write_all(&data).map(|_| f)).await??;
//         }
//     }

//     let replay_blob = web::block(move || std::fs::read(filepath)).await??;

//     let replay_id = db::insert_replay(replay_blob.as_slice(), user_id, pool).await?;

//     let info = ApiSpecificInfoForLogging {
//         user_id: Some(user_id),
//         replay_id: Some(replay_id),
//         ..Default::default()
//     };

//     Ok(insert_extension(HttpResponse::SeeOther(), info)
//         .finish()
//         .customize()
//         .insert_header(("location", format!("/replay/{}", replay_id))))
// }

// #[derive(serde::Deserialize)]
// struct ResetPasswordFormData {
//     password: String,
//     password_confirm: String,
// }

// #[post("/api/reset-password")]
// async fn reset_password(
//     req: HttpRequest,
//     form: web::Form<ResetPasswordFormData>,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<HttpResponse, bwcommon::MyError> {
//     let user_id = if let Some(user_id) = bwcommon::check_auth2(&req, pool.clone()).await {
//         user_id
//     } else {
//         return Ok(HttpResponse::Unauthorized().finish());
//     };

//     if form.password.len() == 0 {
//         return Ok(HttpResponse::Conflict().body("passwords needs to be at least 1 character long"));
//     }

//     if form.password != form.password_confirm {
//         return Ok(HttpResponse::Conflict().body("passwords did not match"));
//     }

//     db::change_password(user_id, form.password.clone(), (**pool).clone()).await?;

//     let info = ApiSpecificInfoForLogging {
//         user_id: Some(user_id),
//         ..Default::default()
//     };

//     Ok(insert_extension(HttpResponse::SeeOther(), info)
//         .header("location", "/")
//         .finish())
// }

// #[post("/api/register")]
// async fn register(
//     form: web::Form<FormData>,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<HttpResponse, bwcommon::MyError> {
//     if let Ok(token) = db::register(
//         form.username.clone(),
//         form.password.clone(),
//         (**pool).clone(),
//     )
//     .await
//     {
//         let info = ApiSpecificInfoForLogging {
//             username: Some(form.username.clone()),
//             ..Default::default()
//         };

//         Ok(insert_extension(HttpResponse::SeeOther(), info)
//             .header("location", "/")
//             .cookie(
//                 actix_http::http::Cookie::build("token", token)
//                     .path("/")
//                     .same_site(actix_http::cookie::SameSite::Lax)
//                     .secure(true)
//                     .permanent()
//                     .finish(),
//             )
//             .cookie(
//                 actix_http::http::Cookie::build("username", &form.username)
//                     .path("/")
//                     .same_site(actix_http::cookie::SameSite::Lax)
//                     .secure(true)
//                     .permanent()
//                     .finish(),
//             )
//             .finish())
//     } else {
//         Ok(HttpResponse::Unauthorized().finish())
//     }
// }

// #[derive(serde::Deserialize)]
// struct FormData {
//     username: String,
//     password: String,
// }

// #[post("/api/login")]
// async fn login(
//     form: web::Form<FormData>,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<HttpResponse, bwcommon::MyError> {
//     if let Ok(token) = db::login(
//         form.username.clone(),
//         form.password.clone(),
//         (**pool).clone(),
//     )
//     .await
//     {
//         let info = ApiSpecificInfoForLogging {
//             username: Some(form.username.clone()),
//             ..Default::default()
//         };

//         Ok(insert_extension(HttpResponse::SeeOther(), info)
//             .header("location", "/")
//             .cookie(
//                 actix_http::http::Cookie::build("token", token)
//                     .path("/")
//                     .same_site(actix_http::cookie::SameSite::Lax)
//                     .secure(true)
//                     .permanent()
//                     .finish(),
//             )
//             .cookie(
//                 actix_http::http::Cookie::build("username", &form.username)
//                     .path("/")
//                     .same_site(actix_http::cookie::SameSite::Lax)
//                     .secure(true)
//                     .permanent()
//                     .finish(),
//             )
//             .finish())
//     } else {
//         Ok(HttpResponse::Unauthorized().finish())
//     }
// }

fn parse_lst_files() -> std::collections::HashMap<u32, std::collections::HashMap<u16, [u8; 3]>> {
    fn parse_lst_file(path: &std::path::Path) -> std::collections::HashMap<u16, [u8; 3]> {
        use std::io::prelude::*;
        let reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
        let mut ret = std::collections::HashMap::new();

        for result in reader.lines() {
            let line: String = result.unwrap();
            if line.len() == 0 {
                continue;
            }

            let split: Vec<&str> = line.split("\t").collect();

            if split.len() != 5 {
                continue;
            }

            let id = split[0].parse::<u16>().unwrap();
            let rgb: Vec<u8> = (&split[1][1..split[1].len() - 1])
                .split(',')
                .map(|x| x.parse::<u8>().unwrap())
                .collect();

            ret.insert(id, [rgb[0], rgb[1], rgb[2]]);
        }

        ret
    }

    let mut root = std::path::Path::new(std::env::var("ROOT_DIR").unwrap().as_str()).join("lst");
    root = root.join("remaster");

    let mut ret = std::collections::HashMap::new();

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|x| !x.file_type().is_dir())
    {
        let tileset = entry
            .file_name()
            .to_string_lossy()
            .to_string()
            .parse::<u32>()
            .unwrap();

        ret.insert(tileset, parse_lst_file(entry.path()));
    }

    ret
}

async fn setup_db_new() -> Result<
    bb8_postgres::bb8::Pool<
        bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
    >,
> {
    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        std::env::var("DB_HOST")
            .unwrap_or("127.0.0.1".to_string())
            .as_str(),
        std::env::var("DB_PORT").unwrap().as_str(),
        std::env::var("DB_USER").unwrap().as_str(),
        std::env::var("DB_PASSWORD").unwrap().as_str(),
        std::env::var("DB_DATABASE")
            .unwrap_or(std::env::var("DB_USER").unwrap())
            .as_str(),
    );
    let manager = bb8_postgres::PostgresConnectionManager::new(
        connection_string.parse()?,
        bb8_postgres::tokio_postgres::NoTls,
    );

    let pool = bb8_postgres::bb8::Pool::builder()
        .max_size(
            std::env::var("DB_CONNECTIONS")
                .unwrap_or("16".to_string())
                .parse::<u32>()?,
        )
        .min_idle(Some(1))
        .max_lifetime(Some(std::time::Duration::from_secs(60)))
        .idle_timeout(Some(std::time::Duration::from_secs(30)))
        .test_on_check_out(true)
        .build(manager)
        .await?;

    anyhow::Ok(pool)
}

pub(crate) async fn start() -> Result<()> {
    //env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let db_new = setup_db_new().await.unwrap();

    let handlebars = {
        let mut registry = Handlebars::new();

        registry.set_strict_mode(true);

        if is_dev_mode() {
            info!("DEV_MODE activated, template hot reloading");
            registry.set_dev_mode(true);
        }

        let mut options = DirectorySourceOptions::default();
        options.tpl_extension = ".hbs".to_owned();
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

        web::Data::new(registry)
    };

    //let manager = r2d2_sqlite::SqliteConnectionManager::file(std::env::var("DB_PATH").unwrap().as_str());

    let tx = bwcommon::create_mixpanel_channel();

    let cache_droper = web::Data::new(std::sync::Mutex::new(
        std::collections::HashSet::<String>::new(),
    ));

    let db_clone = db_new.clone();
    tokio::spawn(async move {
        use rand::distributions::Uniform;
        use rand::prelude::Distribution;

        let mut rng = rand::rngs::OsRng;

        loop {
            info!("Refreshing materialized view");

            match db_clone.get().await {
                Ok(con) => {
                    match con
                        .execute("REFRESH MATERIALIZED VIEW CONCURRENTLY user_stats", &[])
                        .await
                    {
                        Ok(r) => {
                            info!("Successfully refreshed materialized view. r: {:?}", r);
                        }
                        Err(err) => {
                            error!("Failed to refresh materialized view. err: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    error!(
                        "Failed to acquire connection for materialized view refreshing. err: {:?}",
                        err
                    );
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(
                Uniform::new(600, 1000).sample(&mut rng),
            ))
            .await;
        }
    });

    // let set = web::Data::new(Mutex::new(SearchDatabase::new()));

    // {
    //     let db_new = db_new.clone();
    //     let set = set.clone();
    //     tokio::spawn(async move {
    //         populate_search_database(set, db_new).await.unwrap();
    //     });
    // }

    // {
    //     let mut con = db_new.get().await?;
    //     let mut tx = con.transaction().await?;
    //     bwcommon::denormalize_map_tx(12325, &mut tx).await?;
    //     tx.commit().await?;
    // }

    // Pump files up to backblaze
    {
        let client = ClientBuilder::new().https_only(true).build()?;

        if let Err(e) = tokio::fs::create_dir_all("./pending").await {
            error!("failed to create pending directory: {e}");
        }

        if std::env::var("BACKBLAZE_DISABLED").is_err() {
            tokio::task::spawn(async move {
                'full_retry: loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                    let api_info = match b2_authorize_account(
                        &client,
                        &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
                        &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
                    )
                    .await
                    {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Failed to authorize account: {e}");
                            continue;
                        }
                    };

                    const MAPBLOB_BUCKET: &'static str = "784baffe8e56dc107ee50d1c";
                    // const TEST_BUCKET_2: &'static str = "082baf7e0e563c508ef50d1c";

                    let upload_info =
                        match b2_get_upload_url(&client, &api_info, MAPBLOB_BUCKET).await {
                            Ok(upload_info) => upload_info,
                            Err(e) => {
                                error!("Failed to get upload url, trying again: {e}");
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                continue;
                            }
                        };

                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                        let mut entries = match tokio::fs::read_dir("./pending").await {
                            Ok(v) => v,
                            Err(e) => {
                                error!("could not readdir: {e:?}");
                                continue;
                            }
                        };

                        while let Ok(Some(entry)) = entries.next_entry().await {
                            let Ok(filename) = entry.file_name().into_string() else {
                                error!("could not stringify filename: {:?}", entry.file_name());
                                continue;
                            };

                            info!("attempting to upload file: {filename}");

                            let mut split = filename.split('-');
                            let Some(sha1) = split.next() else {
                                error!("could not extract sha1 part: {:?}", filename);
                                continue;
                            };
                            let Some(sha256) = split.next() else {
                                error!("could not extract sha256 part: {:?}", filename);
                                continue;
                            };

                            let mut file = match tokio::fs::File::open(entry.path()).await {
                                Ok(v) => v,
                                Err(e) => {
                                    error!("failed to open file: {e:?}");
                                    continue;
                                }
                            };

                            let metadata = match file.metadata().await {
                                Ok(v) => v,
                                Err(e) => {
                                    error!("failed to get file metadata: {e:?}");
                                    continue;
                                }
                            };

                            let sm = stream! {
                                loop {
                                    let mut bytes = BytesMut::with_capacity(8 * 1024 * 1024);
                                    let bytes_read = file.read_buf(&mut bytes).await?;
                                    if bytes_read == 0 {
                                        break;
                                    }

                                    yield anyhow::Ok(bytes);
                                }
                            };

                            match b2_upload_file(
                                &client,
                                &upload_info,
                                sha256,
                                metadata.len() as usize,
                                sha1.to_owned(),
                                sm,
                            )
                            .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("failed to b2_upload_file: {e}");
                                    continue 'full_retry;
                                }
                            }

                            info!("Finished uploading file. sha256: {sha256}, sha1: {sha1}");

                            match tokio::fs::remove_file(entry.path()).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("failed to remove file: {e}");
                                    continue;
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    let manifest = {
        web::Data::new(serde_json::from_str::<HashMap<String, ManifestChunk>>(
            tokio::fs::read_to_string("./dist/.vite/manifest.json")
                .await?
                .as_str(),
        )?)
    };

    let server = actix_web::HttpServer::new(move || {
        let svc = App::new()
            .app_data(web::Data::new(cache_droper.clone()))
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(db_new.clone()))
            .app_data(web::Data::new(Mutex::new(BackblazeAuth::default())))
            .app_data(handlebars.clone())
            .app_data(parse_lst_files())
            .app_data(manifest.clone())
            .app_data(web::Data::new(awc::Client::default()))
            // .app_data(set.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(crate::middleware::CacheHtmlTransformer)
            .wrap(crate::middleware::PostgresLoggingTransformer)
            .wrap(crate::middleware::UserSessionTransformer)
            .wrap(crate::middleware::LanguageTransformer)
            .wrap(crate::middleware::TrackingAnalyticsTransformer)
            .wrap(crate::middleware::TraceIDTransformer)
            .service(get_map)
            // .service(search)
            // .service(search_by_query)
            .service(get_selection_of_random_maps)
            .service(get_selection_of_random_nsfw_maps)
            .service(set_tags)
            .service(add_tags)
            .service(get_tags)
            // .service(upload_replay)
            // .service(calculate_replay_denorm_data)
            // .service(upload_replay)
            // .service(get_merged_map_chunks)
            // .service(get_raw_map_chunks)
            .service(get_replay)
            // .service(get_replay_dump)
            // .service(search_replay)
            .service(recent_activity)
            // .service(ladder)
            // .service(search2_by_query)
            // .service(search3_by_query_get)
            //
            // .service(login)
            // .service(register)
            //.service(upload_replay2)
            //.service(process_a_lot_of_maps)
            .service(get_minimap)
            .service(get_minimap_resized)
            .service(get_search_result_popup)
            //.service(regen_filenames)
            //.service(autogen_tags)
            // .service(reset_password)
            // API
            .service(crate::api::flags::get_flag)
            .service(crate::api::flags::set_flag)
            .service(crate::api::bulkupload::post_handler)
            .service(crate::api::change_password::post_handler)
            .service(crate::api::change_username::post_handler)
            .service(crate::api::login::post_handler)
            .service(crate::api::register::post_handler)
            .service(crate::api::logout::handler)
            .service(crate::api::sitemap::handler)
            .service(crate::api::sitemap::handlera)
            .service(crate::api::sitemap::handlerb)
            .service(crate::api::sitemap::handlerc)
            .service(crate::api::chk::get_chk_strings)
            .service(crate::api::chk::get_chk_riff_chunks)
            .service(crate::api::chk::get_chk_json)
            .service(crate::api::tests::get_all_maps)
            .service(crate::api::random::handler)
            .service(crate::api::chk::get_chk_trig_json)
            .service(crate::api::chk::get_chk_mbrf_json)
            .service(crate::api::chk::get_eups)
            .service(crate::api::chk::download_chk)
            .service(crate::api::similar_maps::handler)
            //uiv2
            .service(crate::api::uiv2::featured_maps)
            .service(crate::api::uiv2::last_viewed_maps)
            .service(crate::api::uiv2::last_downloaded_maps)
            .service(crate::api::uiv2::last_uploaded_maps)
            .service(crate::api::uiv2::last_uploaded_replays)
            .service(crate::api::uiv2::most_viewed_maps)
            .service(crate::api::uiv2::most_downloaded_maps)
            .service(crate::api::uiv2::get_minimap)
            .service(crate::api::uiv2::is_session_valid)
            .service(crate::api::uiv2::map_info::map_info)
            .service(crate::api::uiv2::filenames::filenames)
            .service(crate::api::uiv2::timestamps::timestamps)
            .service(crate::api::uiv2::filenames2::filenames2)
            .service(crate::api::uiv2::replays::replays)
            .service(crate::api::uiv2::units::units)
            .service(crate::api::uiv2::search::search)
            .service(crate::api::uiv2::search::search_query)
            .service(crate::api::random::handler)
            .service(crate::api::random::handler_noquery)
            .service(crate::api::uiv2::upload::upload_map)
            .service(crate::api::uiv2::logout::logout2)
            // uiv2 ssr
            .service(crate::uiv2::index::index)
            .service(crate::uiv2::index::search_no_query)
            .service(crate::uiv2::index::search_query)
            .service(crate::uiv2::index::map)
            .service(crate::uiv2::index::upload)
            .service(crate::uiv2::index::about)
            .service(crate::uiv2::index::user)
            .service(crate::uiv2::index::login)
            .service(crate::uiv2::index::moderation)
            .service(crate::uiv2::index::webmanifest)
            // Hacks
            .service(hacks::denormalize)
            .service(hacks::denormalize_all)
            // Static pages
            // .service(crate::static_pages::author)
            // .service(crate::static_pages::viewer)
            // .service(crate::static_pages::viewer2)
            // .service(crate::static_pages::css)
            // .service(crate::static_pages::get_reset_password_static_page)
            // .service(crate::static_pages::ladder_static_page)
            // .service(crate::static_pages::recent_static_page)
            // .service(crate::static_pages::get_replays_static_page)
            // .service(crate::static_pages::get_replay_static_page)
            // .service(crate::static_pages::robots)
            // .service(crate::static_pages::get_login_page)
            // .service(crate::static_pages::get_register_page)
            // .service(crate::static_pages::get_upload_replay_page)
            // .service(crate::static_pages::favicon)
            // .service(crate::static_pages::libjs)
            .service(crate::static_pages::redirect_map)
            .service(crate::static_pages::redirect_replay)
            // .service(crate::static_pages::grp)
            // .service(crate::static_pages::bwrender_wasm_js)
            // .service(crate::static_pages::bwrender_wasm_bg_wasm)
            .service(
                Files::new("/assets", "./dist/assets/")
                    .use_etag(false)
                    .use_last_modified(false)
                    .prefer_utf8(true)
                    .disable_content_disposition(),
            )
            .service(
                Files::new("/uiv2/assets", "./dist/assets/")
                    .use_etag(false)
                    .use_last_modified(false)
                    .prefer_utf8(true)
                    .disable_content_disposition(),
            )
            .service(
                Files::new("/", "./public/")
                    .use_etag(false)
                    .use_last_modified(false)
                    .index_file("index.html")
                    .prefer_utf8(true)
                    .disable_content_disposition(),
            );

        let svc = if is_dev_mode() {
            info!("dev mode active, adding local proxy to localhost:3000");

            svc.default_service(web::to(
                |req: HttpRequest, client: web::Data<awc::Client>| async move {
                    use actix_proxy::IntoHttpResponse;

                    let path_query = req
                        .uri()
                        .path_and_query()
                        .map(|v| v.as_str())
                        .unwrap_or(req.uri().path());

                    let url = format!("http://localhost:3000{}", path_query);

                    info!("proxying to {}", url);

                    Result::<HttpResponse, MyError>::Ok(
                        client.get(&url).send().await?.into_http_response(),
                    )
                },
            ))
        } else {
            svc
        };

        svc
    });

    server
        .keep_alive(std::time::Duration::from_secs(120))
        .on_connect(|_x, _y| {
            // let x = x.downcast_ref::<actix_web::rt::net::TcpStream>().unwrap();

            // use std::os::fd::AsFd;
            // let fd = x.as_fd();

            // nix::sys::socket::setsockopt(&fd, nix::sys::socket::sockopt::RcvBuf, &(4 * 1024))
            //     .unwrap();
        })
        .bind("0.0.0.0:8080")
        .unwrap()
        .workers(4)
        .run()
        .await?;

    anyhow::Ok(())
}
