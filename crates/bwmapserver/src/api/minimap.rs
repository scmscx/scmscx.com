//! Minimap images keyed by chk hash: `/api/minimap/{chk_id}` and
//! `/api/minimap_resized/{chk_id}`.
//!
//! Both gate on [`access::check_chk_access`], since a chk is reachable through
//! any map that references it and inherits the most restrictive flag among them.

use axum::extract::{Extension, Path};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use bwcommon::MyError;

use crate::access;
use crate::db;
use crate::webutil::{minimap_cache_control, MaybeUser, Pool};

pub async fn get_minimap(
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    Path((chk_id,)): Path<(String,)>,
) -> Result<Response, MyError> {
    let restricted = match access::check_chk_access(&pool, &chk_id, user.session())
        .await?
        .restricted_or_refusal()
    {
        Ok(restricted) => restricted,
        Err(status) => {
            return Ok((status, [(header::CACHE_CONTROL, "no-cache")]).into_response());
        }
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
    let restricted = match access::check_chk_access(&pool, &chk_id, user.session())
        .await?
        .restricted_or_refusal()
    {
        Ok(restricted) => restricted,
        Err(status) => {
            return Ok((status, [(header::CACHE_CONTROL, "no-cache")]).into_response());
        }
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
