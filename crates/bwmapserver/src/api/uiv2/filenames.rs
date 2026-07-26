use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;

use crate::access;
use crate::webutil::{MaybeUser, Pool, PoolExt};

pub async fn filenames(
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
    let filenames: Vec<String> = con
        .query(
            "select filename.filename
            from mapfilename
            join filename on mapfilename.filename = filename.id
            where mapfilename.map = $1",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| anyhow::Ok(row.try_get(0)?))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(filenames).into_response())
}
