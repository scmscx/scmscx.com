use anyhow::Result;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::get_web_id_from_db_id;
use serde::{Deserialize, Serialize};
use serde_json::json;

use tracing::instrument;

use crate::access;
use crate::webutil::{MaybeUser, Pool, PoolExt};

#[instrument(skip_all)]
pub async fn handler(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, bwcommon::MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    if access::map_is_hidden(&pool, map_id, user.session()).await? {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let con = pool.acquire().await?;

    let nearest2 = {
        let rows = con
            .query(
                "
                select 
                    map.id,
                    chkdenorm.scenario_name,
                    chkdenorm.chkblob,
                    min(modified_time) as lmt,
                    chkdenorm.width,
                    chkdenorm.height,
                    chkdenorm.tileset,
                    minimap.hamming_distance
                from (
                    select
                        minimap.chkhash,
                        vector <~> (select vector from minimap join map on map.chkblob = minimap.chkhash where map.id = $1 limit 1) as hamming_distance
                    from
                        minimap
                    order by
                        hamming_distance
                    limit 25
                ) minimap
                join map on map.chkblob = minimap.chkhash
                join chkdenorm on chkdenorm.chkblob = map.chkblob
                join filetime on filetime.map = map.id
                where
                    map.id != $1 and
                    nsfw = false and
                    outdated = false and
                    unfinished = false and
                    broken = false and
                    blackholed = false and
                    chkdenorm.scenario_name is not null
                group by
                    map.id, chkdenorm.scenario_name, chkdenorm.chkblob, hamming_distance, chkdenorm.width, chkdenorm.height, chkdenorm.tileset
                order by
                    hamming_distance
            ",
                &[&map_id],
            )
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(Chk {
                    map_id: get_web_id_from_db_id(row.try_get("id")?, crate::util::SEED_MAP_ID)?,
                    hamming_distance: row.try_get::<_, f64>("hamming_distance")? as i64,
                    scenario_name: row.try_get("scenario_name")?,
                    last_modified_time: row.try_get("lmt")?,
                    width: row.try_get("width")?,
                    height: row.try_get("height")?,
                    tileset: row.try_get("tileset")?,
                })
            })
            .collect::<Result<Vec<_>>>()
    }?;

    #[derive(Debug, Serialize, Deserialize)]
    struct Chk {
        map_id: String,
        hamming_distance: i64,
        scenario_name: String,
        last_modified_time: Option<i64>,
        width: i64,
        height: i64,
        tileset: i64,
    }

    Ok(Json(json!({
        // "v1": ret,
        "v2": nearest2}))
    .into_response())
}
