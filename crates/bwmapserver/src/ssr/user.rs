use crate::middleware::UserSession;
use crate::ssr::get_navbar_langmap;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web::{HttpMessage, HttpRequest};
use handlebars::{RenderError, RenderErrorReason};
use serde_json::json;
use tracing::{info_span, instrument};

#[instrument(skip_all)]
#[get("/uiv1/user/{username}")]
async fn handler(
    req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
) -> Result<impl Responder, bwcommon::MyError> {
    let (username,) = path.into_inner();

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct ReplayRow {
        id: i64,
        uploaded_time: i64,
        denorm_scenario: String,
        denorm_time_saved: i64,
        denorm_frames: i64,
        map_id: i64,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct MapRow {
        id: String,
        uploaded_time: i64,
        denorm_scenario: String,
    }

    let user_id = {
        let username = username.clone();
        pool.get()
            .await?
            .query_one(
                "
                select id
                from account
                where username = $1",
                &[&username],
            )
            .await?
            .try_get::<_, i64>(0)?
    };

    let f1 = async {
        let pool = pool.clone();
        let con = pool.get().await?;
        let rows: Vec<_> = con.query("
                select replay.id, replay.uploaded_time, coalesce(map.denorm_scenario, ''), denorm_time_saved, denorm_frames, coalesce(map.id, -1)
                from replay
                full outer join map on map.chkblob = chkhash
                where replay.uploaded_by = $1", &[&user_id]).await?.into_iter().map(|row|
                {
                    ReplayRow {
                        id: row.get::<_, i64>(0),
                        uploaded_time: row.get::<_, i64>(1),
                        denorm_scenario: row.get::<_, String>(2),
                        denorm_time_saved: row.get::<_, i64>(3),
                        denorm_frames: row.get::<_, i64>(4),
                        map_id: row.get::<_, i64>(5),
                    }
                }).collect();

        Ok(rows)
    };

    let f2 = async {
        let con = pool.get().await?;
        let rows: Vec<_> = con.query("
                select map.id, uploaded_time, denorm_scenario
                from map
                where map.uploaded_by = $1 and outdated = false and broken = false and nsfw = false and unfinished = false and blackholed = false", &[&user_id]).await?.into_iter().map(|row|
                {
                    MapRow {
                        id: bwcommon::get_web_id_from_db_id(row.try_get(0).unwrap(), crate::util::SEED_MAP_ID).unwrap(),
                        uploaded_time: row.get::<_, i64>(1),
                        denorm_scenario: row.get::<_, String>(2),
                    }
                }).collect();

        Ok(rows)
    };

    let f3 = async {
        let con = pool.get().await?;
        let rows: Vec<_> = con
            .query(
                "
                select name
                from playlist
                where playlist.owner = $1",
                &[&user_id],
            )
            .await?
            .into_iter()
            .map(|row| anyhow::Ok(row.try_get::<_, String>("name")?))
            .collect::<Result<Vec<_>, _>>()?;

        anyhow::Ok(rows)
    };

    let (replays, maps, playlists) = futures::try_join!(f1, f2, f3)?;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct UserDump {
        username: String,
        maps: Vec<MapRow>,
        replays: Vec<ReplayRow>,
    }

    let ret = UserDump {
        username,
        maps,
        replays,
    };

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

    let new_html = info_span!("render").in_scope(|| {
        hb.render(
            "user",
            &json!({
                "user_dump": serde_json::to_string(&ret).map_err(|err| RenderError::from(RenderErrorReason::SerdeError(err)))?,
                "langmap": json!({ "navbar": get_navbar_langmap(lang) }),
                "is_logged_in": user_username.1,
                "username": user_username.0,
                "playlists": playlists,
            }),
        )
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(new_html))
}
