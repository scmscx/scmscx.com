//! `/api/recent_activity`: recent map and replay uploads, merged and sorted.

use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;
use serde::{Deserialize, Serialize};

use crate::webutil::{Pool, PoolExt};

pub async fn recent_activity(Extension(pool): Extension<Pool>) -> Result<Response, MyError> {
    let replay_activity = {
        let conn = pool.acquire().await?;
        let mut v = Vec::new();

        for row in &conn
            .query(
                "
                select replay.id, denorm_scenario, account.username, replay.uploaded_time
                from replay
                join account on account.id = uploaded_by
                where uploaded_by != 10
                order by uploaded_time desc
                limit 2000",
                &[],
            )
            .await?
        {
            v.push((
                row.try_get::<_, i64>(0)?,
                encoding_rs::UTF_8
                    .decode(row.try_get::<_, Vec<u8>>(1)?.as_slice())
                    .0
                    .to_string(),
                row.try_get::<_, String>(2)?,
                row.try_get::<_, i64>(3)?,
            ));
        }

        v
    };

    let map_activity = {
        let mut v = Vec::new();
        let conn = pool.acquire().await?;

        for row in &conn
            .query(
                "
            select map.id, denorm_scenario, account.username, uploaded_time
            from map
            join account on account.id = uploaded_by
            where uploaded_by != 10 and nsfw = false and unfinished = false
            order by uploaded_time desc
            limit 3000",
                &[],
            )
            .await?
        {
            v.push((
                bwcommon::get_web_id_from_db_id(
                    row.try_get::<_, i64>(0)?,
                    crate::util::SEED_MAP_ID,
                )?,
                row.try_get::<_, String>(1)?,
                row.try_get::<_, String>(2)?,
                row.try_get::<_, i64>(3)?,
            ));
        }
        v
    };

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum TypeOfActivity {
        UploadReplay { replay_id: i64, scenario: String },
        UploadMap { map_id: String, scenario: String },
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ActivityInfo {
        username: String,
        time: i64,
        activity: TypeOfActivity,
    }

    let mut ret = Vec::new();

    for i in replay_activity {
        ret.push(ActivityInfo {
            username: i.2,
            time: i.3,
            activity: TypeOfActivity::UploadReplay {
                replay_id: i.0,
                scenario: i.1,
            },
        });
    }

    for i in map_activity {
        ret.push(ActivityInfo {
            username: i.2,
            time: i.3,
            activity: TypeOfActivity::UploadMap {
                map_id: i.0,
                scenario: i.1,
            },
        });
    }

    ret.sort_by(|a, b| a.time.cmp(&b.time).reverse());

    Ok(Json(ret).into_response())
}
