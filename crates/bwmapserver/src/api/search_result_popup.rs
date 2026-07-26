//! `/api/search_result_popup/{map_id}`: scenario name plus a base64 minimap for
//! the search-result hover card.

use axum::extract::{Extension, Path};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use bwcommon::MyError;

use crate::access;
use crate::db;
use crate::webutil::{minimap_cache_control, MaybeUser, Pool, PoolExt};

pub async fn get_search_result_popup(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let (chkhash, scenario, nsfw, blackholed) = {
        let con = pool.acquire().await?;
        let row = con
            .query_one(
                "select chkblob, denorm_scenario, nsfw, blackholed
                from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;

        (
            row.try_get::<_, String>("chkblob")?,
            row.try_get::<_, String>("denorm_scenario")?,
            row.try_get::<_, bool>("nsfw")?,
            row.try_get::<_, bool>("blackholed")?,
        )
    };

    if access::blackholed_is_hidden_from(blackholed, user.session()) {
        return Ok((StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-cache")]).into_response());
    }

    if access::nsfw_requires_login(nsfw, user.session()) {
        return Ok((
            StatusCode::UNAUTHORIZED,
            [(header::CACHE_CONTROL, "no-cache")],
        )
            .into_response());
    }

    let minimap = db::get_minimap(chkhash.clone(), pool.clone()).await?.2;

    use base64::Engine;

    let body = serde_json::to_string(&serde_json::json!({
        "scenario": scenario,
        "minimap": base64::prelude::BASE64_STANDARD.encode(&minimap)
    }))?;

    Ok(IntoResponse::into_response((
        [
            (header::CONTENT_TYPE, "application/json"),
            (
                header::CACHE_CONTROL,
                minimap_cache_control(nsfw || blackholed),
            ),
        ],
        body,
    )))
}
