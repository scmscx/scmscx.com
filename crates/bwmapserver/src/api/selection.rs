//! Bulk map listings for the minimap-checking tooling:
//! `/api/get_selection_of_random_maps` and `/api/get_selection_of_random_nsfw_maps`.
//! Both are restricted to a hard-coded set of user ids.

use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;
use serde::{Deserialize, Serialize};

use crate::webutil::{MaybeUser, Pool, PoolExt};

pub async fn get_selection_of_random_maps(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let user_id = user.id();
    if !matches!(user_id, Some(4 | 5 | 18 | 24 | 32)) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct MapRow {
        map_id: i64,
        chkhash: String,
    }

    let rows = {
        let rows: Result<Vec<_>, MyError> = {
            let con = pool.acquire().await?;
            con.query(
                "
               select * from (
                   select map.id, map.chkblob from map
                   where nsfw = false and blackholed = false
                   except
                   select map.id, map.chkblob from map
                   join tagmap on tagmap.map = map.id
                   join tag on tag.id = tagmap.tag
                   where (key = 'minimap_checked' and value = 'true')
               ) a
               where chkblob is not null
               order by random()
               ",
                &[],
            )
            .await?
        }
        .into_iter()
        .map(|x| {
            Ok(MapRow {
                map_id: x.try_get(0)?,
                chkhash: x.try_get(1)?,
            })
        })
        .collect();

        rows?
    };

    Ok(Json(rows).into_response())
}

pub async fn get_selection_of_random_nsfw_maps(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let user_id = user.id();
    if !matches!(user_id, Some(4 | 18 | 24 | 32)) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct MapRow {
        map_id: i64,
        chkhash: String,
    }

    let rows = {
        let rows: Result<Vec<_>, MyError> = {
            let con = pool.acquire().await?;
            con.query(
                "
                select distinct map.id, map.chkblob
                from map
                where nsfw = false and blackholed = false
                ",
                &[],
            )
            .await?
        }
        .into_iter()
        .map(|x| {
            Ok(MapRow {
                map_id: x.try_get(0)?,
                chkhash: x.try_get(1)?,
            })
        })
        .collect();

        rows?
    };

    Ok(Json(rows).into_response())
}
