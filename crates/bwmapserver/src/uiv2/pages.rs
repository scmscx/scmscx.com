//! The server-rendered page shells.
//!
//! Each handler here renders the HTML document that wraps the SolidJS app:
//! the `<head>` with its localized title and meta tags, the hashed asset
//! links, and `<html lang>`. The app itself is client-rendered, so these are
//! shells rather than full pages -- but they are what link-preview unfurlers
//! and search engines read, which is why the meta tags are translated.

use axum::extract::{Extension, Path, Query};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use bwcommon::MyError;
use bwmap::ParsedChk;
use tracing::error;
use tracing::instrument;

use crate::access;
use crate::middleware::Language;
use crate::search2;
use crate::search2::SearchParams;
use crate::server::{Handlebars, Manifest, ManifestChunk};
use crate::util::is_dev_mode;
use crate::util::scenario_and_description;
use crate::webutil::{MaybeUser, Pool, PoolExt};

fn html(body: String) -> Response {
    ([(header::CONTENT_TYPE, "text/html")], body).into_response()
}

/// Renders one of the SSR shells.
///
/// Every page needs the same seven values -- the resolved language, the hashed
/// asset filenames, and the dev flag -- so they live here rather than being
/// repeated per handler. `extra` is merged over the top for the two templates
/// that want more (the search title, the map's scenario fields).
///
/// `extra` is a `Map` rather than a `Value` so that merging is total: a
/// `Value` that happened not to be an object would have its fields silently
/// dropped instead of failing to compile.
///
/// The manifest lookups return an error rather than unwrapping: a missing entry
/// means the frontend build and the binary disagree, which is worth a 500 and a
/// log line instead of a panicked worker and a dropped connection.
fn render_page(
    hb: &Handlebars,
    manifest: &Manifest,
    lang: &str,
    template: &str,
    extra: serde_json::Map<String, serde_json::Value>,
) -> Result<Response, MyError> {
    let asset = |name: &str| -> Result<&ManifestChunk, MyError> {
        manifest
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("no manifest entry for {name}").into())
    };

    let entry = asset("app/index.tsx")?;
    let mut data = serde_json::json!({
        "lang": lang,
        "favicon_ico": asset("app/assets/favicon.ico")?.file,
        "favicon_svg": asset("app/assets/favicon.svg")?.file,
        "apple-touch-icon_png": asset("app/assets/apple-touch-icon-180x180.png")?.file,

        "jsFile": entry.file,
        "css": entry.css,
        "dev": is_dev_mode(),
    });

    if let Some(target) = data.as_object_mut() {
        target.extend(extra);
    }

    Ok(html(hb.render(template, &data)?))
}

/// The `extra` fields for a template that wants more than the common values.
fn fields<const N: usize>(
    pairs: [(&str, serde_json::Value); N],
) -> serde_json::Map<String, serde_json::Value> {
    pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
}

/// A shell that needs nothing beyond the common values, which is most of them.
fn render_shell(
    hb: &Handlebars,
    manifest: &Manifest,
    lang: &str,
    template: &str,
) -> Result<Response, MyError> {
    render_page(hb, manifest, lang, template, serde_json::Map::new())
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
    Extension(Language(lang)): Extension<Language>,
) -> Result<Response, MyError> {
    render_shell(&hb, &manifest, &lang, "uiv2-index")
}

#[instrument(skip_all, name = "/moderation")]
pub async fn moderation(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Extension(Language(lang)): Extension<Language>,
) -> Result<Response, MyError> {
    render_shell(&hb, &manifest, &lang, "uiv2-moderation")
}

/// Serves both `/search` and `/search/{query}`.
///
/// The path segment is optional rather than each route getting its own wrapper:
/// `Path` yields `None` when the route carries no parameters, so one handler
/// covers both and the tracing span sits on the thing actually handling the
/// request.
#[instrument(skip_all, name = "/search")]
pub async fn search(
    Query(search_params): Query<SearchParams>,
    Extension(pool): Extension<Pool>,
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Extension(Language(lang)): Extension<Language>,
    Extension(strings): Extension<std::sync::Arc<crate::i18n::Strings>>,
    path: Option<Path<String>>,
) -> Result<Response, MyError> {
    let query = path.map_or_else(String::new, |Path(q)| q);
    // Substituted here rather than in the template: the `{{t}}` helper writes a
    // string as-is, so a key with placeholders in it is a build error (see
    // tools/check-i18n.mjs). Server-side substitution is the supported way to
    // use one.
    //
    // `meta.search.results` is deliberately phrased as a label and a count
    // ("Maps found: 1") rather than as a sentence ("1 maps found"), in every
    // language. Nothing here selects a plural form, and there is no plural
    // machinery to select one with: English, German, Spanish and French would
    // all read wrong at a count of 1, French additionally wants the singular at
    // 0, and Russian needs three different forms. Keeping the number out of
    // agreement with the noun sidesteps all of that, so do not "fix" the
    // wording back into a sentence without adding real plural support first.
    let page_title = if query.is_empty() {
        strings.get("meta.search.title", &lang).to_owned()
    } else {
        let num_results = search2::search2(query.as_str(), false, &search_params, pool.clone())
            .await?
            .0;

        strings
            .get("meta.search.results", &lang)
            .replace("{count}", &num_results.to_string())
            .replace("{query}", &query)
    };

    render_page(
        &hb,
        &manifest,
        &lang,
        "uiv2-search",
        fields([("page_title", page_title.into())]),
    )
}

#[instrument(skip_all, name = "/map")]
pub async fn map(
    user: MaybeUser,
    Extension(pool): Extension<Pool>,
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Extension(Language(lang)): Extension<Language>,
    Path(map_id): Path<String>,
) -> Result<Response, MyError> {
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

    let (chkblob, denorm_scenario, nsfw, blackholed) = {
        let con = pool.acquire().await?;
        let rows = con
            .query(
                "select
                    nsfw,
                    blackholed,
                    denorm_scenario,
                    length,
                    ver,
                    data
                from
                    map
                -- LEFT, not inner: the chkblob only exists once the background
                -- processor has run, and a map must not 404 between being
                -- uploaded and being processed.
                left join
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

        // NULL together until the map has been processed.
        let chkblob = match (
            rows[0].try_get::<_, Option<i64>>("length")?,
            rows[0].try_get::<_, Option<i64>>("ver")?,
            rows[0].try_get::<_, Option<Vec<u8>>>("data")?,
        ) {
            (Some(length), Some(ver), Some(data)) => {
                bwcommon::ensure!(ver == 1);
                zstd::bulk::decompress(data.as_slice(), length as usize)?
            }
            _ => Vec::new(),
        };

        (
            chkblob,
            rows[0].try_get::<_, Option<String>>("denorm_scenario")?,
            rows[0].try_get::<_, bool>("nsfw")?,
            rows[0].try_get::<_, bool>("blackholed")?,
        )
    };

    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());
    let (scenario, description) = scenario_and_description(&parsed_chk);
    // Before processing there is no chk to read a title out of, but the upload
    // recorded one, so the page still gets a real name instead of a blank.
    let scenario = if chkblob.is_empty() {
        denorm_scenario.unwrap_or(scenario)
    } else {
        scenario
    };

    if access::nsfw_requires_login(nsfw, user.session()) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    if access::blackholed_is_hidden_from(blackholed, user.session()) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    render_page(
        &hb,
        &manifest,
        &lang,
        "uiv2-map",
        fields([
            ("sanitized_scenario_name", scenario.into()),
            ("sanitized_scenario_description", description.into()),
            (
                "map_id",
                bwcommon::get_web_id_from_db_id(map_id, crate::util::SEED_MAP_ID)?.into(),
            ),
        ]),
    )
}

#[instrument(skip_all, name = "/about")]
pub async fn about(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Extension(Language(lang)): Extension<Language>,
) -> Result<Response, MyError> {
    render_shell(&hb, &manifest, &lang, "uiv2-about")
}

#[instrument(skip_all, name = "/user")]
pub async fn user(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Extension(Language(lang)): Extension<Language>,
) -> Result<Response, MyError> {
    render_shell(&hb, &manifest, &lang, "uiv2-user")
}

#[instrument(skip_all, name = "/upload")]
pub async fn upload(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Extension(Language(lang)): Extension<Language>,
) -> Result<Response, MyError> {
    render_shell(&hb, &manifest, &lang, "uiv2-upload")
}

#[instrument(skip_all, name = "/login")]
pub async fn login(
    Extension(hb): Extension<Handlebars>,
    Extension(manifest): Extension<Manifest>,
    Extension(Language(lang)): Extension<Language>,
) -> Result<Response, MyError> {
    render_shell(&hb, &manifest, &lang, "uiv2-login")
}
