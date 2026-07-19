use crate::search2::search2;
use crate::search2::search_cache;
use crate::search2::SearchParams;
use crate::webutil::{MaybeUser, Pool};
use axum::extract::Extension;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// The sort values `search2`/`search_cache` understand; anything else is a 400 at
/// the edge (both search handlers reject it before touching the DB).
fn is_valid_sort(sort: &str) -> bool {
    matches!(
        sort,
        "relevancy"
            | "scenario"
            | "lastmodifiedold"
            | "lastmodifiednew"
            | "timeuploadedold"
            | "timeuploadednew"
    )
}

async fn handler(
    allow_nsfw: bool,
    query: String,
    query_params: Query<SearchParams>,
    pool: Pool,
) -> Result<Response, bwcommon::MyError> {
    let query_params = query_params.0;

    if !is_valid_sort(query_params.sort.as_str()) {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }

    let maps = search2(query.as_str(), allow_nsfw, &query_params, pool.clone()).await?;

    Ok(Json(json!({
        "total_results": maps.0,
        "maps": maps.1,
    }))
    .into_response())
}

pub async fn search_query(
    user: MaybeUser,
    Path(query): Path<String>,
    query_params: Query<SearchParams>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, bwcommon::MyError> {
    handler(user.0.is_some(), query, query_params, pool).await
}

pub async fn search(
    user: MaybeUser,
    query_params: Query<SearchParams>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, bwcommon::MyError> {
    handler(user.0.is_some(), String::new(), query_params, pool).await
}

/// Result count only, for the Cloudflare Pages `/search/:query` edge Function's
/// "{N} maps found for: {query}" preview title. `allow_nsfw = false` because link
/// previews are always fetched unauthenticated — matching the count the old
/// server-rendered search `<title>` used. Calls `search_cache` directly (its cache
/// key ignores `offset`) so the count is the full total, never a page-limited value.
pub async fn search_count(
    Path(query): Path<String>,
    query_params: Query<SearchParams>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, bwcommon::MyError> {
    let query_params = query_params.0;

    if !is_valid_sort(query_params.sort.as_str()) {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }

    let maps = search_cache(query.as_str(), false, &query_params, pool).await?;

    Ok(Json(json!({ "count": maps.len() })).into_response())
}
