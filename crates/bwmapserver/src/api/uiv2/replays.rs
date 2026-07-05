use axum::extract::{Extension, Path};
use axum::response::Response;
use axum::Json;
use bwcommon::with_logging_info;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;

use crate::webutil::Pool;

pub async fn replays(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let pool = pool.clone();
    let con = pool.get().await?;

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

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(with_logging_info(info, Json(replays)))
}
