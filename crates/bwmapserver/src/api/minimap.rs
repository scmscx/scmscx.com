//! Minimap images keyed by chk hash: `/api/minimap/{chk_id}` and
//! `/api/minimap_resized/{chk_id}`.
//!
//! Both gate on [`check_chk_access`], since a chk is reachable through any map
//! that references it and inherits the most restrictive flag among them.

use axum::extract::{Extension, Path};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use bwcommon::MyError;

use crate::db;
use crate::webutil::{minimap_cache_control, MaybeUser, Pool, PoolExt};

enum ChkAccess {
    /// Access granted. `restricted` is true when any map referencing this chk is
    /// NSFW or blackholed — i.e. the response was only served because the caller
    /// is authorized. Callers use it to keep such content out of shared caches.
    Allowed {
        restricted: bool,
    },
    NotFound,
    Unauthorized,
}

async fn check_chk_access(
    pool: &Pool,
    chk_id: &str,
    user_id: Option<i64>,
) -> Result<ChkAccess, anyhow::Error> {
    // A chk inherits the most-restrictive flag of any map that references
    // it: if even one map is blackholed, treat the whole chk as blackholed;
    // if any is NSFW, treat the whole chk as NSFW.
    let row = pool
        .acquire()
        .await?
        .query_one(
            "select
                count(*) > 0 as exists_any,
                coalesce(bool_or(blackholed), false) as any_blackholed,
                coalesce(bool_or(nsfw), false) as any_nsfw
             from map
             where chkblob = $1",
            &[&chk_id],
        )
        .await?;

    let exists_any: bool = row.try_get("exists_any")?;
    let any_blackholed: bool = row.try_get("any_blackholed")?;
    let any_nsfw: bool = row.try_get("any_nsfw")?;

    let is_admin = user_id == Some(4);

    if !exists_any {
        return Ok(ChkAccess::NotFound);
    }
    if any_blackholed && !is_admin {
        return Ok(ChkAccess::NotFound);
    }
    if any_nsfw && user_id.is_none() {
        return Ok(ChkAccess::Unauthorized);
    }

    Ok(ChkAccess::Allowed {
        restricted: any_nsfw || any_blackholed,
    })
}

pub async fn get_minimap(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((chk_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let user_id = user.id();

    let restricted = match check_chk_access(&pool, &chk_id, user_id).await? {
        ChkAccess::NotFound => {
            return Ok(
                (StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-cache")]).into_response(),
            );
        }
        ChkAccess::Unauthorized => {
            return Ok((
                StatusCode::UNAUTHORIZED,
                [(header::CACHE_CONTROL, "no-cache")],
            )
                .into_response());
        }
        ChkAccess::Allowed { restricted } => restricted,
    };

    let minimap = db::get_minimap(chk_id, pool.clone()).await?.2;

    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, minimap_cache_control(restricted)),
        ],
        minimap,
    )
        .into_response())
}

pub async fn get_minimap_resized(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((chk_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let user_id = user.id();

    let restricted = match check_chk_access(&pool, &chk_id, user_id).await? {
        ChkAccess::NotFound => {
            return Ok(
                (StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-cache")]).into_response(),
            );
        }
        ChkAccess::Unauthorized => {
            return Ok((
                StatusCode::UNAUTHORIZED,
                [(header::CACHE_CONTROL, "no-cache")],
            )
                .into_response());
        }
        ChkAccess::Allowed { restricted } => restricted,
    };

    use image::ImageDecoder;

    let minimap = db::get_minimap(chk_id.clone(), pool.clone()).await?.2;

    let cursor = std::io::Cursor::new(minimap.as_slice());
    let png = image::codecs::png::PngDecoder::new(cursor)?;
    let (x, y) = png.dimensions();

    let mut image_data = vec![0; png.total_bytes() as usize];

    (|| {
        anyhow::ensure!(png.color_type() == image::ColorType::Rgb8);
        anyhow::Ok(())
    })()?;

    png.read_image(image_data.as_mut_slice())?;

    let image: image::ImageBuffer<image::Rgb<u8>, _> =
        image::ImageBuffer::from_vec(x, y, image_data).unwrap();

    let scaling_factor = std::cmp::min(512 / x, 512 / y);

    let image = image::imageops::resize(
        &image,
        x * scaling_factor,
        y * scaling_factor,
        image::imageops::Nearest,
    );

    let mut png = Vec::<u8>::new();
    use image::ImageEncoder;
    image::codecs::png::PngEncoder::new(&mut png).write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgb8,
    )?;

    Ok(IntoResponse::into_response((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, minimap_cache_control(restricted)),
        ],
        png,
    )))
}
