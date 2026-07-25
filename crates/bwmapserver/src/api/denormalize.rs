//! Admin-only maintenance endpoints that rebuild a map's denormalized columns:
//! `/api/denormalize/{map_id}` for one map, `/api/denormalize_all` for every map
//! with a chk. Both are gated to a single account and answer 404 (not 403) to
//! anyone else, so their existence isn't advertised.
//!
//! `denormalize_all` walks the whole table, so it runs the per-map work through
//! [`process_iter_async_concurrent`] with a bounded number of in-flight futures
//! rather than spawning one per row.

use anyhow::Result;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures_util::FutureExt;
use tracing::info;

use crate::webutil::{MaybeUser, Pool, PoolExt};

/// The only account allowed to run these.
const ADMIN_USER_ID: i64 = 4;

pub(crate) async fn denormalize(
    user: MaybeUser,
    Extension(pool): Extension<Pool>,
    Path((map_id,)): Path<(String,)>,
) -> Result<Response, bwcommon::MyError> {
    let Some(session) = user.0 else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if session.id != ADMIN_USER_ID {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let map_id = crate::util::parse_map_id(&map_id)?;

    let mut con = pool.acquire().await?;
    let mut tx = con.transaction().await?;

    bwcommon::denormalize_map_tx(map_id, &mut tx).await?;

    tx.commit().await?;

    Ok(StatusCode::OK.into_response())
}

pub(crate) async fn denormalize_all(
    user: MaybeUser,
    Extension(pool): Extension<Pool>,
) -> Result<Response, bwcommon::MyError> {
    let Some(session) = user.0 else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if session.id != ADMIN_USER_ID {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let con = pool.acquire().await?;

    let map_ids = con
        .query("Select map.id from map where chkblob is not null", &[])
        .await?
        .into_iter()
        .map(|x| anyhow::Ok(x.try_get::<_, i64>(0)?))
        .collect::<Result<Vec<_>>>()?;

    process_iter_async_concurrent(
        map_ids.iter(),
        || {},
        128,
        |x, y| info!("Completed: {x}, ret: {y:?}"),
        |(): (), map_id: &i64| async {
            let mut con = pool.acquire().await?;
            let mut tx = con.transaction().await?;
            bwcommon::denormalize_map_tx(*map_id, &mut tx).await?;
            tx.commit().await?;
            anyhow::Ok(())
        },
    )
    .await;

    Ok(StatusCode::OK.into_response())
}

/// Drive `iter` through `func` with at most `max_outstanding` futures in flight,
/// calling `on_item_completed` as each finishes. `cloner` mints the per-item
/// context handed to `func`. Returns how many items were processed.
async fn process_iter_async_concurrent<I, T, F, J, R, F2, H, Z>(
    mut iter: I,
    cloner: H,
    max_outstanding: usize,
    on_item_completed: F2,
    func: F,
) -> usize
where
    I: Iterator<Item = T>,
    F: Fn(Z, T) -> R,
    R: futures_util::Future<Output = J> + Send,
    F2: Fn(usize, J),
    H: Fn() -> Z,
{
    let mut futs = Vec::new();
    let mut counter = 0;
    loop {
        while futs.len() < max_outstanding {
            if let Some(entry) = iter.next() {
                futs.push(func(cloner(), entry).boxed());
            } else {
                break;
            }
        }

        if futs.is_empty() {
            break;
        }

        let (item, _, remaining_futures) = futures_util::future::select_all(futs).await;

        futs = remaining_futures;

        counter += 1;

        on_item_completed(counter, item);
    }

    counter
}
