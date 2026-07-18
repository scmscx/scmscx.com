use axum::extract::{Extension, Path, Query};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use bwcommon::MyError;
use bwmap::ParsedChk;
use tracing::error;
use tracing::instrument;

use crate::actix::{Handlebars, Manifest};
use crate::search2;
use crate::search2::SearchParams;
use crate::util::is_dev_mode;
use crate::util::scenario_and_description;
use crate::webutil::{MaybeUser, Pool};

fn html(body: String) -> Response {
    ([(header::CONTENT_TYPE, "text/html")], body).into_response()
}

#[instrument(skip_all, name = "/site.webmanifest")]
pub async fn webmanifest(Extension(manifest): Extension<Manifest>) -> Result<Response, MyError> {
    let body = serde_json::json!({
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
    .to_string();

    Ok((
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
        .into_response())
}

#[instrument(skip_all, name = "/")]
pub async fn index(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
) -> Result<Response, MyError> {
    Ok(html(hb.render(
        "uiv2-index",
        &serde_json::json!({
            "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
            "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
            "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

            "jsFile": manifest.get("app/index.tsx").unwrap().file,
            "css": manifest.get("app/index.tsx").unwrap().css,
            "dev": is_dev_mode()
        }),
    )?))
}

#[instrument(skip_all, name = "/moderation")]
pub async fn moderation(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
) -> Result<Response, MyError> {
    Ok(html(hb.render(
        "uiv2-moderation",
        &serde_json::json!({
            "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
            "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
            "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

            "jsFile": manifest.get("app/index.tsx").unwrap().file,
            "css": manifest.get("app/index.tsx").unwrap().css,
            "dev": is_dev_mode()
        }),
    )?))
}

pub async fn search_no_query(
    Query(search_params): Query<SearchParams>,
    Extension(pool): Extension<Pool>,
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
) -> Result<Response, MyError> {
    search_handler(String::new(), &search_params, pool, hb, manifest).await
}

pub async fn search_query(
    Query(search_params): Query<SearchParams>,
    Extension(pool): Extension<Pool>,
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Path(query): Path<String>,
) -> Result<Response, MyError> {
    search_handler(query, &search_params, pool, hb, manifest).await
}

#[instrument(skip_all, name = "/search")]
pub async fn search_handler(
    query: String,
    search_params: &SearchParams,
    pool: Pool,
    hb: Handlebars,
    manifest: Manifest,
) -> Result<Response, MyError> {
    let page_title = if query.is_empty() {
        "Search StarCraft: Brood War Maps".to_owned()
    } else {
        let num_results = search2::search2(query.as_str(), false, search_params, pool.clone())
            .await?
            .0;

        format!("{num_results} maps found for: {query}")
    };

    Ok(html(hb.render(
        "uiv2-search",
        &serde_json::json!({
            "page_title": page_title,

            "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
            "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
            "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

            "jsFile": manifest.get("app/index.tsx").unwrap().file,
            "css": manifest.get("app/index.tsx").unwrap().css,
            "dev": is_dev_mode()
        }),
    )?))
}

#[instrument(skip_all, name = "/map")]
pub async fn map(
    user: MaybeUser,
    Extension(pool): Extension<Pool>,
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Path(map_id): Path<String>,
) -> Result<Response, MyError> {
    let user_id = user.id();

    let map_id = if map_id.chars().all(char::is_numeric) && map_id.len() < 8 {
        return Ok(Redirect::permanent(&format!(
            "/map/{}",
            bwcommon::get_web_id_from_db_id(map_id.parse::<i64>()?, crate::util::SEED_MAP_ID)?
        ))
        .into_response());
    } else if let Ok(id) =
        bwcommon::get_db_id_from_web_id(map_id.as_str(), crate::util::SEED_MAP_ID)
    {
        id
    } else {
        return Ok(StatusCode::NOT_FOUND.into_response());
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

        if rows.is_empty() {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }

        if rows.len() != 1 {
            error!("There's more than 1 row for map_id: {map_id}, rows: {rows:?}");
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
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
    let (scenario, description) = scenario_and_description(&parsed_chk);

    if nsfw && user_id.is_none() {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    if blackholed && user_id != Some(uploaded_by) && user_id != Some(4) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    Ok(html(hb.render(
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
            "dev": is_dev_mode()
        }),
    )?))
}

#[instrument(skip_all, name = "/about")]
pub async fn about(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
) -> Result<Response, MyError> {
    Ok(html(hb.render(
        "uiv2-about",
        &serde_json::json!({
            "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
            "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
            "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

            "jsFile": manifest.get("app/index.tsx").unwrap().file,
            "css": manifest.get("app/index.tsx").unwrap().css,
            "dev": is_dev_mode()
        }),
    )?))
}

#[instrument(skip_all, name = "/user")]
pub async fn user(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
) -> Result<Response, MyError> {
    Ok(html(hb.render(
        "uiv2-user",
        &serde_json::json!({
            "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
            "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
            "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

            "jsFile": manifest.get("app/index.tsx").unwrap().file,
            "css": manifest.get("app/index.tsx").unwrap().css,
            "dev": is_dev_mode()
        }),
    )?))
}

#[instrument(skip_all, name = "/upload")]
pub async fn upload(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
) -> Result<Response, MyError> {
    Ok(html(hb.render(
        "uiv2-upload",
        &serde_json::json!({
            "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
            "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
            "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

            "jsFile": manifest.get("app/index.tsx").unwrap().file,
            "css": manifest.get("app/index.tsx").unwrap().css,
            "dev": is_dev_mode()
        }),
    )?))
}

#[instrument(skip_all, name = "/login")]
pub async fn login(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
) -> Result<Response, MyError> {
    Ok(html(hb.render(
        "uiv2-login",
        &serde_json::json!({
            "favicon_ico": manifest.get("app/assets/favicon.ico").unwrap().file,
            "favicon_svg": manifest.get("app/assets/favicon.svg").unwrap().file,
            "apple-touch-icon_png": manifest.get("app/assets/apple-touch-icon-180x180.png").unwrap().file,

            "jsFile": manifest.get("app/index.tsx").unwrap().file,
            "css": manifest.get("app/index.tsx").unwrap().css,
            "dev": is_dev_mode()
        }),
    )?))
}
