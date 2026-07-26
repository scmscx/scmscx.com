use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;

use crate::access;
use crate::webutil::{MaybeUser, Pool, PoolExt};

pub async fn timestamps(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    if access::map_is_hidden(&pool, map_id, user.session()).await? {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let pool = pool.clone();
    let con = pool.acquire().await?;

    let filetimes: Vec<i64> = con
        .query(
            "select distinct modified_time
            from map
            join filetime on filetime.map = map.id
            where map.id = $1
            order by modified_time",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| anyhow::Ok(row.try_get("modified_time")?))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(filetimes).into_response())
}
