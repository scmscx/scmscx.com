//! Map tags: `/api/tags/{map_id}` (read/replace) and `/api/addtags/{map_id}`.

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::webutil::{MaybeUser, Pool, PoolExt};

#[derive(Serialize, Deserialize)]
pub(crate) struct TagPost {
    key: String,
    value: String,
}

pub async fn get_tags(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    if crate::access::map_is_hidden(&pool, map_id, user.session()).await? {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let con = pool.acquire().await?;
    let tags = con
        .query(
            "
            select key, value
            from tagmap
            join tag on tagmap.tag = tag.id
            where tagmap.map = $1",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| {
            anyhow::Ok(TagPost {
                key: row.try_get(0)?,
                value: row.try_get(1)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(tags).into_response())
}

pub async fn set_tags(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
    Json(tags): Json<Vec<TagPost>>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(session) = user.session() else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let mut map = std::collections::hash_map::HashMap::new();

    for t in tags {
        map.insert(t.key, t.value);
    }

    let outcome = db::set_tags(map_id, map, session, pool).await?;

    match outcome {
        None => Ok(StatusCode::NOT_FOUND.into_response()),
        Some(false) => Ok(StatusCode::FORBIDDEN.into_response()),
        Some(true) => Ok(StatusCode::OK.into_response()),
    }
}

pub async fn add_tags(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((map_id,)): Path<(String,)>,
    Json(tags): Json<Vec<TagPost>>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(session) = user.session() else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let mut map = std::collections::hash_map::HashMap::new();

    for t in tags {
        map.insert(t.key, t.value);
    }

    let outcome = db::add_tags(map_id, map, session, pool).await?;

    match outcome {
        None => Ok(StatusCode::NOT_FOUND.into_response()),
        Some(false) => Ok(StatusCode::FORBIDDEN.into_response()),
        Some(true) => Ok(StatusCode::OK.into_response()),
    }
}
