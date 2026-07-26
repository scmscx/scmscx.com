use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;

use crate::access;
use crate::webutil::{MaybeUser, Pool, PoolExt};

pub async fn replays(
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

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct ReplayInfo {
        id: i64,
        frames: i64,
        time_saved: i64,
        creator: String,
    }

    let replays: Vec<ReplayInfo> = con
        .query(
            "
        select replay.id, replay.denorm_frames, replay.denorm_time_saved, replay.denorm_game_creator
        from replay
        join map on map.chkblob = replay.chkhash
        where map.id = $1
        order by replay.denorm_frames",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|r| {
            anyhow::Ok(ReplayInfo {
                id: r.try_get("id")?,
                frames: r.try_get("denorm_frames")?,
                time_saved: r.try_get("denorm_time_saved")?,
                creator: encoding_rs::UTF_8
                    .decode(r.try_get::<_, Vec<u8>>("denorm_game_creator")?.as_slice())
                    .0
                    .to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(replays).into_response())
}
