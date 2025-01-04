use crate::middleware::UserSession;
use crate::ssr::get_navbar_langmap;
use actix_web::HttpMessage;
use actix_web::Result;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use bwcommon::MyError;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[get("/uiv1")]
#[instrument(skip_all, name = "/")]
async fn handler(
    req: HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
) -> Result<impl Responder, MyError> {
    let lang = req
        .extensions()
        .get::<bwcommon::LangData>()
        .unwrap_or(&bwcommon::LangData::English)
        .to_owned();

    let user_username = req
        .extensions()
        .get::<UserSession>()
        .map(|x| (x.username.clone(), true))
        .unwrap_or_default();

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

    let last_uploaded_maps = async {
        let con = pool.get().await?;

        let ret: Result<Vec<_>, anyhow::Error> = con.query(
                "select id, denorm_scenario, uploaded_time, views, downloads, last_viewed, last_downloaded
                from map
                where denorm_scenario is not null and uploaded_by != 10 and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
                order by uploaded_time desc limit 5
                ", &[]).await?.into_iter().map(|x| Ok(MapRow {
                    map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
                    scenario_name: x.try_get::<_, String>(1)?,
                    uploaded_time: x.try_get(2)?,
                    views: x.try_get(3)?,
                    downloads: x.try_get(4)?,
                    last_viewed: x.try_get(5)?,
                    last_downloaded: x.try_get(6)?,
                })).collect();

        anyhow::Ok(ret?)
    };

    let most_viewed_maps = async {
        let con = pool.get().await?;

        let ret: Result<Vec<_>, anyhow::Error> = con.query(
                "select id, denorm_scenario, uploaded_time, views, downloads, last_viewed, last_downloaded
                from map
                where denorm_scenario is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
                order by views desc limit 5", &[]).await?.into_iter().map(|x| Ok(MapRow {
                    map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
                    scenario_name: x.try_get::<_, String>(1)?,
                    uploaded_time: x.try_get(2)?,
                    views: x.try_get(3)?,
                    downloads: x.try_get(4)?,
                    last_viewed: x.try_get(5)?,
                    last_downloaded: x.try_get(6)?,
                })).collect();

        anyhow::Ok(ret?)
    };

    let most_downloaded_maps = async {
        let con = pool.get().await?;

        let ret: Result<Vec<_>, anyhow::Error> = con.query(
                "select id, denorm_scenario, uploaded_time, views, downloads, last_viewed, last_downloaded
                from map
                where denorm_scenario is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
                order by downloads desc limit 5", &[]).await?.into_iter().map(|x| Ok(MapRow {
                    map_id: bwcommon::get_web_id_from_db_id(x.try_get(0)?, crate::util::SEED_MAP_ID)?,
                    scenario_name: x.try_get::<_, String>(1)?,
                    uploaded_time: x.try_get(2)?,
                    views: x.try_get(3)?,
                    downloads: x.try_get(4)?,
                    last_viewed: x.try_get(5)?,
                    last_downloaded: x.try_get(6)?,
                })).collect();

        anyhow::Ok(ret?)
    };

    let last_viewed_maps = async {
        let con = pool.get().await?;

        let ret: Result<Vec<_>, anyhow::Error> = con.query(
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
                })).collect();

        anyhow::Ok(ret?)
    };

    let last_downloaded_maps = async {
        let con = pool.get().await?;

        let ret: Result<Vec<_>, anyhow::Error> = con.query(
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
                })).collect();

        anyhow::Ok(ret?)
    };

    #[derive(Debug, Serialize, Deserialize)]
    struct ReplayRow {
        replay_id: i64,
        scenario_name: String,
        uploaded_time: i64,
    }

    let last_uploaded_replays = async {
        let con = pool.get().await?;

        let ret: Result<Vec<_>, anyhow::Error> = con.query(
                "select replay.id, map.denorm_scenario, replay.uploaded_time
                from replay
                join map on map.chkblob = replay.chkhash
                where map.denorm_scenario is not null and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false
                order by uploaded_time desc limit 5", &[]).await?.into_iter().map(|x| Ok(ReplayRow {
                    replay_id: x.try_get(0)?,
                    scenario_name: crate::util::sanitize_sc_string(x.try_get::<_, String>(1)?.as_str()),
                    uploaded_time: x.try_get(2)?,
                })).collect();

        anyhow::Ok(ret?)
    };
    let (
        last_uploaded_maps,
        most_viewed_maps,
        most_downloaded_maps,
        last_viewed_maps,
        last_downloaded_maps,
        last_uploaded_replays,
    ) = futures::try_join!(
        last_uploaded_maps,
        most_viewed_maps,
        most_downloaded_maps,
        last_viewed_maps,
        last_downloaded_maps,
        last_uploaded_replays,
    )?;

    let langmap = if lang == bwcommon::LangData::Korean {
        serde_json::json!({
            "h1": "scmscx.com에 오신 것을 환영합니다",
            "h3": "우주에서 가장 큰 스타크래프트: 브루드 워 맵 데이터베이스",
            "recently_viewed_maps": "최근에 본 지도",
            "recently_downloaded_maps": "최근 다운로드한 지도",
            "recently_uploaded_maps": "최근에 업로드한 지도",
            "recently_uploaded_replays": "최근에 업로드된 리플레이",
            "most_viewed_maps": "가장 많이 본 지도",
            "most_downloaded_maps": "가장 많이 다운로드한 지도",
            "navbar": get_navbar_langmap(lang)
        })
    } else {
        serde_json::json!({
            "h1": "Welcome to scmscx.com",
            "h3": "The largest StarCraft: Brood War map database in the universe",
            "recently_viewed_maps": "Recently Viewed Maps",
            "recently_downloaded_maps": "Recently Downloaded Maps",
            "recently_uploaded_maps": "Recently Uploaded Maps",
            "recently_uploaded_replays": "Recently Uploaded Replays",
            "most_viewed_maps": "Most Viewed Maps",
            "most_downloaded_maps": "Most Downloaded Maps",
            "navbar": get_navbar_langmap(lang)
        })
    };

    let new_html = hb.render(
        "index",
        &serde_json::json!({
            "last_viewed_maps": last_viewed_maps,
            "most_viewed_maps": most_viewed_maps,
            "last_downloaded_maps": last_downloaded_maps,
            "most_downloaded_maps": most_downloaded_maps,
            "last_uploaded_maps": last_uploaded_maps,
            "last_uploaded_replays": last_uploaded_replays,
            "langmap": langmap,
            "is_logged_in": user_username.1,
            "username": user_username.0,
        }),
    )?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(new_html)
        .customize())
}
