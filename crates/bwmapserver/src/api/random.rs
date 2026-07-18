// select map.id as id from (
//     select distinct map as id
//     from stringmap2
//     where $1 <% data and ((scenario_name = true and $3) or (scenario_description = true and $4) or (unit_names = true and $5) or (force_names = true and $6) or (file_names = true and $7))
// ) as sq
// join map on map.id = sq.id
// where ($2 or map.nsfw = false) and outdated = false and unfinished = false and broken = false
// order by random()
// limit 1

use crate::search2::{search_cache, SearchParams};
use crate::webutil::{MaybeUser, Pool};
use axum::extract::{Extension, Path, Query};
use axum::response::{IntoResponse, Response};
use axum::Json;
use rand::Rng;

async fn random_core(
    query: &str,
    allow_nsfw: bool,
    query_params: Query<SearchParams>,
    pool: Pool,
) -> Result<String, bwcommon::MyError> {
    let query_params = query_params.0;

    match query_params.sort.as_str() {
        "relevancy" | "scenario" | "lastmodifiedold" | "lastmodifiednew" | "timeuploadedold"
        | "timeuploadednew" => {}
        _ => {
            return Err(anyhow::anyhow!("unknown sort").into());
        }
    }

    let maps = search_cache(query, allow_nsfw, &query_params, pool.clone()).await?;

    if maps.is_empty() {
        return Err(anyhow::anyhow!("no maps found").into());
    }

    let mut rng = rand::rng();
    let random_number = rng.random_range(0..maps.len());

    Ok(maps[random_number].id.clone())
}

pub async fn handler(
    Path(query): Path<String>,
    user: MaybeUser,
    query_params: Query<SearchParams>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, bwcommon::MyError> {
    let allow_nsfw = user.0.is_some();

    let map_id = random_core(query.as_str(), allow_nsfw, query_params, pool).await?;

    Ok(Json(map_id).into_response())
}

pub async fn handler_noquery(
    user: MaybeUser,
    query_params: Query<SearchParams>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, bwcommon::MyError> {
    let allow_nsfw = user.0.is_some();

    let map_id = random_core("", allow_nsfw, query_params, pool).await?;

    Ok(Json(map_id).into_response())
}
