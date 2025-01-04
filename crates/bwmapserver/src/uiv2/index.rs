use crate::actix::ManifestChunk;
use crate::middleware::UserSession;
use crate::search2;
use crate::search2::SearchParams;
use crate::util::sanitize_sc_string;
use actix_web::web::Data;
use actix_web::web::Query;
use actix_web::HttpMessage;
use actix_web::Result;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use bwcommon::MyError;
use bwmap::ParsedChk;
use std::collections::HashMap;
use tracing::error;
use tracing::instrument;

// fn convert_path_to_map_id(s: &str) -> anyhow::Result<i64> {
//     if s.chars().all(|x| x.is_ascii_digit()) && s.len() < 8 {
//         Ok(s.parse::<i64>()?)
//     } else {
//         Ok(bwcommon::get_db_id_from_web_id(
//             s,
//             crate::util::SEED_MAP_ID,
//         )?)
//     }
// }

#[get("/site.webmanifest")]
#[instrument(skip_all, name = "/site.webmanifest")]
pub async fn webmanifest(
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(
            serde_json::json!({
                "name": "scmscx.com",
                "short_name": "scmscx.com",
                "description": "scmscx.com",
                "theme_color": "#111111",
                "background_color":"#111111",
                "display":"standalone",
                "icons": [
                    {
                        "src": manifest.get("app/assets/pwa-64x64.png").unwrap().file,
                        "sizes": "64x64",
                        "type": "image/png",
                    },
                    {
                        "src": manifest.get("app/assets/pwa-192x192.png").unwrap().file,
                        "sizes": "192x192",
                        "type": "image/png",
                    },
                    {
                        "src": manifest.get("app/assets/pwa-512x512.png").unwrap().file,
                        "sizes": "512x512",
                        "type": "image/png",
                    },
                    {
                        "src": manifest.get("app/assets/maskable-icon-512x512.png").unwrap().file,
                        "sizes": "512x512",
                        "type": "image/png",
                        "purpose": "maskable",
                    }
                ]
            })
            .to_string(),
        )
        .customize()
        .append_header(("cache-control", "no-cache")))
}

#[get("/")]
#[instrument(skip_all, name = "/")]
pub async fn index(
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(
            "uiv2-index",
            &serde_json::json!({
                "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
                "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
                "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

                "jsFile": manifest.get("app/index.tsx").unwrap().file,
                "css": manifest.get("app/index.tsx").unwrap().css,
                "dev": std::env::var("DEV_MODE").unwrap_or("false".to_string()).as_str() == "true"
            }),
        )?)
        .customize())
}

#[get("/moderation")]
#[instrument(skip_all, name = "/moderation")]
pub async fn moderation(
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(
            "uiv2-moderation",
            &serde_json::json!({
                "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
                "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
                "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

                "jsFile": manifest.get("app/index.tsx").unwrap().file,
                "css": manifest.get("app/index.tsx").unwrap().css,
                "dev": std::env::var("DEV_MODE").unwrap_or("false".to_string()).as_str() == "true"
            }),
        )?)
        .customize())
}

#[get("/search")]
pub async fn search_no_query(
    search_params: Query<SearchParams>,
    req: HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    search_handler(
        "".to_owned(),
        &search_params.into_inner(),
        req,
        pool,
        hb,
        manifest,
    )
    .await
}

#[get("/search/{query}")]
pub async fn search_query(
    req: HttpRequest,
    search_params: Query<SearchParams>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
    query: web::Path<String>,
) -> Result<impl Responder, MyError> {
    search_handler(
        query.into_inner(),
        &search_params.into_inner(),
        req,
        pool,
        hb,
        manifest,
    )
    .await
}

#[instrument(skip_all, name = "/search")]
pub async fn search_handler(
    query: String,
    search_params: &SearchParams,
    _req: HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    // let lang = req
    //     .extensions()
    //     .get::<bwcommon::LangData>()
    //     .unwrap_or(&bwcommon::LangData::English)
    //     .to_owned();

    // let user_username = req
    //     .extensions()
    //     .get::<UserSession>()
    //     .map(|x| (x.username.clone(), true))
    //     .unwrap_or_default();

    let page_title = if query.is_empty() {
        "Search StarCraft: Brood War Maps".to_owned()
    } else {
        let num_results = search2::search2(query.as_str(), false, search_params, pool.clone())
            .await?
            .0;

        format!("{num_results} maps found for: {query}")
    };

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(
            "uiv2-search",
            &serde_json::json!({
                "page_title": page_title,

                "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
                "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
                "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

                "jsFile": manifest.get("app/index.tsx").unwrap().file,
                "css": manifest.get("app/index.tsx").unwrap().css,
                "dev": std::env::var("DEV_MODE").unwrap_or("false".to_string()).as_str() == "true"
            }),
        )?)
        .customize())
}

#[get("/map/{map_id}")]
#[instrument(skip_all, name = "/map")]
pub async fn map(
    req: HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
    path: web::Path<String>,
) -> Result<impl Responder, MyError> {
    let user_id = req.extensions().get::<UserSession>().map(|x| x.id);

    let map_id = path.into_inner();
    let map_id = if map_id.chars().all(|x| x.is_numeric()) && map_id.len() < 8 {
        return Ok(HttpResponse::PermanentRedirect()
            .append_header((
                "Location",
                format!(
                    "/map/{}",
                    bwcommon::get_web_id_from_db_id(
                        map_id.parse::<i64>()?,
                        crate::util::SEED_MAP_ID
                    )?
                ),
            ))
            .finish()
            .customize());
    } else {
        if let Ok(id) = bwcommon::get_db_id_from_web_id(map_id.as_str(), crate::util::SEED_MAP_ID) {
            id
        } else {
            return Ok(HttpResponse::NotFound().finish().customize());
        }
    };

    let (chkblob, uploaded_by, nsfw, blackholed) = {
        let con = pool.get().await?;
        let rows = con
            .query(
                "select
                    uploaded_by,
                    nsfw,
                    blackholed,
                    length,
                    ver,
                    data
                from
                    map
                join
                    chkblob on chkblob.hash = map.chkblob
                where
                    map.id = $1",
                &[&map_id],
            )
            .await?;

        if rows.len() == 0 {
            return Ok(HttpResponse::NotFound().finish().customize());
        }

        if rows.len() != 1 {
            error!("There's more than 1 row for map_id: {map_id}, rows: {rows:?}");
            return Ok(HttpResponse::InternalServerError().finish().customize());
        }

        let length = rows[0].try_get::<_, i64>("length")? as usize;
        let ver = rows[0].try_get::<_, i64>("ver")?;
        let data = rows[0].try_get::<_, Vec<u8>>("data")?;

        bwcommon::ensure!(ver == 1);

        let chkblob = zstd::bulk::decompress(data.as_slice(), length)?;

        (
            chkblob,
            rows[0].try_get::<_, i64>("uploaded_by")?,
            rows[0].try_get::<_, bool>("nsfw")?,
            rows[0].try_get::<_, bool>("blackholed")?,
        )
    };

    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let (scenario, description) = if let Ok(sprp) = &parsed_chk.sprp {
        let scenario = if *sprp.scenario_name_string_number == 0 {
            "Untitled Scenario".to_string()
        } else {
            if let Ok(s) = parsed_chk.get_string(*sprp.scenario_name_string_number as usize) {
                sanitize_sc_string(s.as_str())
            } else {
                "<<Could not get scenario name>>".to_owned()
            }
        };

        let description = if *sprp.description_string_number == 0 {
            "".to_string()
        } else {
            if let Ok(s) = parsed_chk.get_string(*sprp.description_string_number as usize) {
                sanitize_sc_string(s.as_str())
            } else {
                "<<Could not get scenario description>>".to_owned()
            }
        };

        (scenario, description)
    } else {
        (
            "<<Could not get scenario name>>".to_owned(),
            "<<Could not get scenario description>>".to_owned(),
        )
    };

    if nsfw && user_id == None {
        return Ok(HttpResponse::Forbidden().finish().customize());
    }

    if blackholed && user_id != Some(uploaded_by) && user_id != Some(4) {
        return Ok(HttpResponse::NotFound().finish().customize());
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(
            "uiv2-map",
            &serde_json::json!({
                "sanitized_scenario_name": scenario,
                "sanitized_scenario_description": description,
                "map_id": bwcommon::get_web_id_from_db_id(map_id, crate::util::SEED_MAP_ID)?,

                "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
                "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
                "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

                "jsFile": manifest.get("app/index.tsx").unwrap().file,
                "css": manifest.get("app/index.tsx").unwrap().css,
                "dev": std::env::var("DEV_MODE").unwrap_or("false".to_string()).as_str() == "true"
            }),
        )?)
        .customize())
}

#[get("/about")]
#[instrument(skip_all, name = "/about")]
pub async fn about(
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(
            "uiv2-about",
            &serde_json::json!({
                "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
                "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
                "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

                "jsFile": manifest.get("app/index.tsx").unwrap().file,
                "css": manifest.get("app/index.tsx").unwrap().css,
                "dev": std::env::var("DEV_MODE").unwrap_or("false".to_string()).as_str() == "true"
            }),
        )?)
        .customize())
}

#[get("/user/{username}")]
#[instrument(skip_all, name = "/user")]
pub async fn user(
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(
            "uiv2-user",
            &serde_json::json!({

                "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
                "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
                "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

                "jsFile": manifest.get("app/index.tsx").unwrap().file,
                "css": manifest.get("app/index.tsx").unwrap().css,
                "dev": std::env::var("DEV_MODE").unwrap_or("false".to_string()).as_str() == "true"
            }),
        )?)
        .customize())
}

#[get("/upload")]
#[instrument(skip_all, name = "/upload")]
pub async fn upload(
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(
            "uiv2-upload",
            &serde_json::json!({

                "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
                "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
                "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

                "jsFile": manifest.get("app/index.tsx").unwrap().file,
                "css": manifest.get("app/index.tsx").unwrap().css,
                "dev": std::env::var("DEV_MODE").unwrap_or("false".to_string()).as_str() == "true"
            }),
        )?)
        .customize())
}

#[get("/login")]
#[instrument(skip_all, name = "/login")]
pub async fn login(
    hb: web::Data<handlebars::Handlebars<'_>>,
    manifest: Data<HashMap<String, ManifestChunk>>,
) -> Result<impl Responder, MyError> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(
            "uiv2-login",
            &serde_json::json!({

                "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
                "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
                "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

                "jsFile": manifest.get("app/index.tsx").unwrap().file,
                "css": manifest.get("app/index.tsx").unwrap().css,
                "dev": std::env::var("DEV_MODE").unwrap_or("false".to_string()).as_str() == "true"
            }),
        )?)
        .customize())
}
