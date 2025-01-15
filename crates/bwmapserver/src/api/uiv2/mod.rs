pub mod filenames;
pub mod filenames2;
pub mod logout;
pub mod map_info;
pub mod replays;
pub mod search;
pub mod timestamps;
pub mod units;
pub mod upload;

use crate::db;
use crate::middleware::UserSession;
use actix_web::post;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::Result;
use actix_web::{get, web, HttpResponse, Responder};
use bwcommon::insert_extension;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize)]
struct MapRow {
    map_id: String,
    scenario_name: String,
    uploaded_time: i64,
    views: Option<i64>,
    downloads: Option<i64>,
    last_viewed: Option<i64>,
    last_downloaded: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReplayRow {
    replay_id: i64,
    scenario_name: String,
    uploaded_time: i64,
    map_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FeaturedMap {
    map_id: String,
    scenario_name: String,
}

#[get("/api/uiv2/featured_maps")]
#[instrument(skip_all, name = "/api/uiv2/featured_maps")]
async fn featured_maps(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let con = pool.get().await?;

    let ret: Vec<_> = con
        .query(
            "select map.id, map.denorm_scenario
        from featuredmaps
        join map on featuredmaps.map_id = map.id
        order by rank desc",
            &[],
        )
        .await?
        .into_iter()
        .map(|x| {
            Ok(FeaturedMap {
                map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
                scenario_name: x.try_get::<_, String>(1)?,
            })
        })
        .collect::<Result<_, anyhow::Error>>()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(ret)
        .customize())
}

#[get("/api/uiv2/last_viewed_maps")]
#[instrument(skip_all, name = "/api/uiv2/last_viewed_maps")]
async fn last_viewed_maps(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let con = pool.get().await?;

    let ret: Vec<_> = con.query(
        "select id, denorm_scenario, uploaded_time, views, downloads, last_viewed, last_downloaded
        from map
        where denorm_scenario is not null and last_viewed is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
        order by last_viewed desc limit 5", &[]).await?.into_iter().map(|x| Ok(MapRow {
            map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
            scenario_name: x.try_get::<_, String>(1)?,
            uploaded_time: x.try_get(2)?,
            views: x.try_get(3)?,
            downloads: x.try_get(4)?,
            last_viewed: x.try_get(5)?,
            last_downloaded: x.try_get(6)?,
        })).collect::<Result<_, anyhow::Error>>()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(ret)
        .customize())
}

#[get("/api/uiv2/last_downloaded_maps")]
#[instrument(skip_all, name = "/api/uiv2/last_downloaded_maps")]
async fn last_downloaded_maps(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let con = pool.get().await?;

    let ret: Vec<_> = con.query(
        "select id, denorm_scenario, uploaded_time, views, downloads, last_viewed, last_downloaded
        from map
        where denorm_scenario is not null and last_downloaded is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
        order by last_downloaded desc limit 5", &[]).await?.into_iter().map(|x| Ok(MapRow {
            map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
            scenario_name: x.try_get::<_, String>(1)?,
            uploaded_time: x.try_get(2)?,
            views: x.try_get(3)?,
            downloads: x.try_get(4)?,
            last_viewed: x.try_get(5)?,
            last_downloaded: x.try_get(6)?,
        })).collect::<Result<_, anyhow::Error>>()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(ret)
        .customize())
}

#[get("/api/uiv2/last_uploaded_maps")]
#[instrument(skip_all, name = "/api/uiv2/last_uploaded_maps")]
async fn last_uploaded_maps(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let con = pool.get().await?;

    let ret: Vec<_> = con.query(
        "select id, denorm_scenario, uploaded_time, views, downloads, last_viewed, last_downloaded
        from map
        where denorm_scenario is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
        order by uploaded_time desc limit 5", &[]).await?.into_iter().map(|x| Ok(MapRow {
            map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
            scenario_name: x.try_get::<_, String>(1)?,
            uploaded_time: x.try_get(2)?,
            views: x.try_get(3)?,
            downloads: x.try_get(4)?,
            last_viewed: x.try_get(5)?,
            last_downloaded: x.try_get(6)?,
        })).collect::<Result<_, anyhow::Error>>()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(ret)
        .customize())
}

#[get("/api/uiv2/last_uploaded_replays")]
#[instrument(skip_all, name = "/api/uiv2/last_uploaded_replays")]
async fn last_uploaded_replays(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let con = pool.get().await?;

    let ret: Vec<_> = con.query(
            "select replay.id, map.denorm_scenario, replay.uploaded_time, map.id
            from replay
            join map on map.chkblob = replay.chkhash
            where map.denorm_scenario is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
            order by replay.uploaded_time desc limit 5", &[]).await?.into_iter().map(|x| Ok(ReplayRow {
                replay_id: x.try_get(0)?,
                scenario_name: x.try_get::<_, String>(1)?,
                uploaded_time: x.try_get(2)?,
                map_id: bwcommon::get_web_id_from_db_id(x.try_get(3)?, crate::util::SEED_MAP_ID)?,
        })).collect::<Result<_, anyhow::Error>>()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(ret)
        .customize())
}

#[get("/api/uiv2/most_viewed_maps")]
#[instrument(skip_all, name = "/api/uiv2/most_viewed_maps")]
async fn most_viewed_maps(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let con = pool.get().await?;

    let ret: Vec<_> = con.query(
        "select id, denorm_scenario, uploaded_time, views, downloads, last_viewed, last_downloaded
        from map
        where denorm_scenario is not null and last_viewed is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
        order by views desc limit 5", &[]).await?.into_iter().map(|x| Ok(MapRow {
            map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
            scenario_name: x.try_get::<_, String>(1)?,
            uploaded_time: x.try_get(2)?,
            views: x.try_get(3)?,
            downloads: x.try_get(4)?,
            last_viewed: x.try_get(5)?,
            last_downloaded: x.try_get(6)?,
        })).collect::<Result<_, anyhow::Error>>()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(ret)
        .customize())
}

#[get("/api/uiv2/most_downloaded_maps")]
#[instrument(skip_all, name = "/api/uiv2/most_downloaded_maps")]
async fn most_downloaded_maps(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let con = pool.get().await?;

    let ret: Vec<_> = con.query(
        "select id, denorm_scenario, uploaded_time, views, downloads, last_viewed, last_downloaded
        from map
        where denorm_scenario is not null and last_viewed is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
        order by downloads desc limit 5", &[]).await?.into_iter().map(|x| Ok(MapRow {
            map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
            scenario_name: x.try_get::<_, String>(1)?,
            uploaded_time: x.try_get(2)?,
            views: x.try_get(3)?,
            downloads: x.try_get(4)?,
            last_viewed: x.try_get(5)?,
            last_downloaded: x.try_get(6)?,
        })).collect::<Result<_, anyhow::Error>>()?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(ret)
        .customize())
}

#[get("/api/uiv2/minimap/{map_id}")]
async fn get_minimap(
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

    let (chkhash, uploaded_by, nsfw, blackholed) = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select
                chkblob,
                uploaded_by,
                nsfw,
                blackholed
            from
                map
            where
                map.id = $1",
                &[&map_id],
            )
            .await?;

        (
            row.try_get::<_, String>("chkblob")?,
            row.try_get("uploaded_by")?,
            row.try_get("nsfw")?,
            row.try_get("blackholed")?,
        )
    };

    let user_id = req.extensions().get::<UserSession>().map(|x| x.id);

    if nsfw && user_id == None {
        return Ok(HttpResponse::Forbidden().finish().customize());
    }

    if blackholed && user_id != Some(uploaded_by) && user_id != Some(4) {
        return Ok(HttpResponse::NotFound().finish().customize());
    }

    let minimap = db::get_minimap(chkhash.clone(), (**pool).clone()).await?.2;

    let info = ApiSpecificInfoForLogging {
        map_id: Some(map_id),
        chk_hash: Some(chkhash),
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("image/png")
        .body(minimap)
        .customize())
}

#[post("/api/uiv2/is_session_valid")]
async fn is_session_valid(req: HttpRequest) -> Result<impl Responder, bwcommon::MyError> {
    let is_session_valid = req.extensions().get::<UserSession>().is_some();

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&json! {is_session_valid}).unwrap())
        .customize())
}
