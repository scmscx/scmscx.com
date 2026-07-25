//! Replay blob downloads: `/api/replays/{replay_id}`.

use axum::extract::{Extension, Path};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use bwcommon::MyError;

use crate::webutil::{Pool, PoolExt};

pub async fn get_replay(
    Extension(pool): Extension<Pool>,
    Path((replay_id,)): Path<(i64,)>,
) -> Result<Response, MyError> {
    let replay_blob =
        pool.acquire().await?
        .query_one("select replayblob.data from replay join replayblob on replayblob.hash = replay.hash where replay.id = $1", &[&replay_id])
        .await?.try_get::<_, Vec<u8>>(0)?;

    Ok(IntoResponse::into_response((
        [(header::CONTENT_TYPE, "application/octet-stream")],
        replay_blob,
    )))
}
