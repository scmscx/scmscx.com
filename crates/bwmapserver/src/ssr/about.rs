use crate::middleware::UserSession;
use crate::ssr::get_navbar_langmap;
use actix_web::{get, web, HttpMessage, HttpResponse, Responder};
use actix_web::{HttpRequest, Result};
use serde_json::json;

#[get("/uiv1/about")]
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

    let map_count = async {
        let con = pool.get().await?;

        anyhow::Ok(
            con.query_one("select count(*) from map", &[])
                .await?
                .try_get::<_, i64>(0)?,
        )
    };

    let map_views_last_1_day = async {
        let con = pool.get().await?;

        anyhow::Ok(
            con.query_one("select views from user_stats where days = 1", &[])
                .await?
                .try_get::<_, i64>(0)?,
        )
    };

    let map_views_last_7_days = async {
        let con = pool.get().await?;

        anyhow::Ok(
            con.query_one("select views from user_stats where days = 7", &[])
                .await?
                .try_get::<_, i64>(0)?,
        )
    };

    let map_views_last_30_days = async {
        let con = pool.get().await?;

        anyhow::Ok(
            con.query_one("select views from user_stats where days = 30", &[])
                .await?
                .try_get::<_, i64>(0)?,
        )
    };

    let (map_count, map_views_last_1_day, map_views_last_7_days, map_views_last_30_days) = futures_util::try_join!(
        map_count,
        map_views_last_1_day,
        map_views_last_7_days,
        map_views_last_30_days
    )?;

    let new_html = hb.render(
        "about",
        &json!({
            "map_count": map_count,
            "map_views_last_1_day": map_views_last_1_day,
            "map_views_last_7_days": map_views_last_7_days,
            "map_views_last_30_days": map_views_last_30_days,
            "langmap": json!({ "navbar": get_navbar_langmap(lang) }),
            "is_logged_in": user_username.1,
            "username": user_username.0,
        }),
    )?;

    Ok(HttpResponse::Ok().content_type("text/html").body(new_html))
}
