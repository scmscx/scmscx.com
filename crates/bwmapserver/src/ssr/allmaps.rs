use crate::middleware::UserSession;
use crate::ssr::get_navbar_langmap;
use actix_web::{get, web, HttpMessage, HttpResponse, Responder};
use actix_web::{HttpRequest, Result};
use serde_json::json;

#[get("/experiments/allmaps")]
async fn handler(
    req: HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
) -> Result<impl Responder, bwcommon::MyError> {
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

    #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
    struct Map {
        id: i64,
        scenario_name: String,
        last_modified: i64,
    }

    let allow_nsfw = bwcommon::check_auth4(&req, (**pool).clone())
        .await?
        .is_some();

    let maps = pool.get().await?.query("
                select * from (
                    select distinct map.id, map.denorm_scenario, uploaded_time, min(filetime.modified_time) as modified_time from map
                    left join filetime on filetime.map = map.id
                    where (nsfw = false or $1) and outdated = false and unfinished = false and broken = false and blackholed = false and denorm_scenario is not null
                    group by map.id, map.denorm_scenario, uploaded_time) sq
                order by random()", &[&allow_nsfw]).await?.into_iter().map(|row|
            {
                anyhow::Ok(Map {
                    id: row.try_get(0)?,
                    scenario_name: row.try_get(1)?,
                    last_modified: row.try_get::<_, Option<i64>>("modified_time")?.unwrap_or(-1),
                })
            }).collect::<Result<Vec<_>, _>>()?;

    let new_html = hb.render(
        "allmaps",
        &json!({
            "search_results": serde_json::to_string(&maps)?,
            "langmap": json!({ "navbar": get_navbar_langmap(lang) }),
            "is_logged_in": user_username.1,
            "username": user_username.0,
        }),
    )?;

    Ok(HttpResponse::Ok().content_type("text/html").body(new_html))
}
