use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::{get_web_id_from_db_id, MyError};
use bwmap::ParsedChk;
use serde_json::json;
use tracing::instrument;

use crate::util::{parse_map_id, scenario_and_description, SEED_MAP_ID};
use crate::webutil::Pool;

/// Read-only per-map metadata for the Cloudflare Pages `/map/:id` edge Function,
/// which injects it as OpenGraph tags for link previews / crawlers.
///
/// Deliberately distinct from `map_info`: this endpoint has **no side effects**
/// (it does not bump `views`/`last_viewed`), so a social scraper fetching a preview
/// never inflates view counts. It also returns the canonical `web_id` so the edge
/// Function can redirect numeric or non-canonical ids, plus the `nsfw`/`blackholed`
/// flags so the Function can serve a generic preview for hidden maps (previews are
/// always fetched unauthenticated). The SPA + API still enforce real access on the
/// underlying map data.
#[instrument(skip_all, name = "/api/uiv2/map_meta")]
pub async fn map_meta(
    Extension(pool): Extension<Pool>,
    Path(map_id): Path<String>,
) -> Result<Response, MyError> {
    // Resolve any id form (short numeric db id or web id) to a db id, then compute
    // the canonical `web_id` so the edge Function can redirect non-canonical URLs.
    let Ok(db_id) = parse_map_id(map_id.as_str()) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let web_id = get_web_id_from_db_id(db_id, SEED_MAP_ID)?;

    let con = pool.get().await?;
    let rows = con
        .query(
            "select
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
            &[&db_id],
        )
        .await?;

    if rows.is_empty() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let nsfw = rows[0].try_get::<_, bool>("nsfw")?;
    let blackholed = rows[0].try_get::<_, bool>("blackholed")?;

    // Hidden maps must not be disclosed to an unauthenticated caller — this endpoint
    // is publicly reachable through the /api proxy. Return 404, exactly as the /map
    // SSR and map_info handlers do for an unauthorized viewer, so nothing (scenario,
    // web id, or even existence) leaks. The edge Function treats a missing and a
    // hidden map identically (generic preview); an owner still views the map via the
    // SPA, which calls the authenticated map_info.
    if nsfw || blackholed {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let length = rows[0].try_get::<_, i64>("length")? as usize;
    let ver = rows[0].try_get::<_, i64>("ver")?;
    let data = rows[0].try_get::<_, Vec<u8>>("data")?;

    bwcommon::ensure!(ver == 1);

    let chkblob = zstd::bulk::decompress(data.as_slice(), length)?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());
    let (scenario, description) = scenario_and_description(&parsed_chk);

    Ok(Json(json!({
        "web_id": web_id,
        "scenario": scenario,
        "scenario_description": description,
        "nsfw": nsfw,
        "blackholed": blackholed,
    }))
    .into_response())
}
