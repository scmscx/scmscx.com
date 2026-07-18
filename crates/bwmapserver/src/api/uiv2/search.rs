use crate::search2::search2;
use crate::search2::SearchParams;
use crate::webutil::{MaybeUser, Pool};
use axum::extract::Extension;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::with_logging_info;
use bwcommon::ApiSpecificInfoForLogging;
use serde_json::json;

async fn handler(
    allow_nsfw: bool,
    query: String,
    query_params: Query<SearchParams>,
    pool: Pool,
) -> Result<Response, bwcommon::MyError> {
    let query_params = query_params.0;

    match query_params.sort.as_str() {
        "relevancy" | "scenario" | "lastmodifiedold" | "lastmodifiednew" | "timeuploadedold"
        | "timeuploadednew" => {}
        _ => {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }
    }

    let maps = search2(query.as_str(), allow_nsfw, &query_params, pool.clone()).await?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(with_logging_info(
        info,
        Json(json!({
            "total_results": maps.0,
            "maps": maps.1,
        })),
    ))
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
