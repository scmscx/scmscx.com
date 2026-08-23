use anyhow::{anyhow, Result};
use image::imageops::FilterType;
use image::RgbImage;
use webp::{Encoder, WebPConfig};

const MAX_DIMENSION: u32 = 8192;

/// Halve an RGB image exactly, averaging each 2x2 block into one pixel.
///
/// Both dimensions must be even. This is the ideal reduction for a 2:1 step —
/// every output pixel is the mean of exactly the four inputs it covers — and it
/// is roughly 175x faster than asking a general resampler to do the same job,
/// because there are no filter weights to evaluate.
fn halve(src: &[u8], width: u32, height: u32) -> Vec<u8> {
    let (out_w, out_h) = (width as usize / 2, height as usize / 2);
    let row = width as usize * 3;
    let mut out = vec![0u8; out_w * out_h * 3];

    for oy in 0..out_h {
        let top = oy * 2 * row;
        let bottom = top + row;
        for ox in 0..out_w {
            let a = top + ox * 6;
            let b = bottom + ox * 6;
            let o = (oy * out_w + ox) * 3;
            for c in 0..3 {
                out[o + c] = ((u32::from(src[a + c])
                    + u32::from(src[a + 3 + c])
                    + u32::from(src[b + c])
                    + u32::from(src[b + 3 + c]))
                    / 4) as u8;
            }
        }
    }

    out
}

/// Downscale an RGB image so that neither dimension exceeds `MAX_DIMENSION`,
/// preserving the aspect ratio.
///
/// Halves the image with a box filter for as long as it is at least twice the
/// target, then resamples the remainder with Lanczos3. Map images are the size
/// they are because of the tile grid, so the largest of them — a 256x256 map
/// renders to 16384x16384 — land on an exact 2:1 ratio and are served entirely
/// by the halving step: 21s of Lanczos3 becomes 0.12s, and the result is *better*,
/// since box-averaging a 2:1 step neither rings nor aliases. Sizes that are not a
/// power-of-two multiple of the cap (a 192x192 map, 12288px) still take the
/// resampler, now fed a correctly prefiltered image.
///
/// Takes `rgb_data` **by value** on purpose: a full-size buffer is 768MiB, so it
/// is moved rather than copied, and each intermediate is freed as soon as the
/// next one exists. Passing it by reference cost two extra live copies of the
/// full-size image per in-flight encode and OOM-killed the renderer.
pub fn downscale_to_cap(
    rgb_data: Vec<u8>,
    width: u32,
    height: u32,
    filter: FilterType,
) -> (Vec<u8>, u32, u32) {
    if width <= MAX_DIMENSION && height <= MAX_DIMENSION {
        return (rgb_data, width, height);
    }

    let scale = f64::min(
        MAX_DIMENSION as f64 / width as f64,
        MAX_DIMENSION as f64 / height as f64,
    );
    let new_width = (width as f64 * scale).round() as u32;
    let new_height = (height as f64 * scale).round() as u32;

    // Cheap exact halving first. The guard keeps every step lossless-by-averaging
    // and never overshoots the target in either axis.
    let (mut data, mut w, mut h) = (rgb_data, width, height);
    while w >= new_width * 2 && h >= new_height * 2 && w % 2 == 0 && h % 2 == 0 {
        data = halve(&data, w, h);
        w /= 2;
        h /= 2;
    }

    if (w, h) == (new_width, new_height) {
        return (data, w, h);
    }

    let img =
        RgbImage::from_raw(w, h, data).expect("RGB data length must match width * height * 3");
    let resized = image::imageops::resize(&img, new_width, new_height, filter);
    (resized.into_raw(), new_width, new_height)
}

/// Encode raw RGB pixel data to WebP.
///
/// Does **not** resize: run [`downscale_to_cap`] first, so the caller can time
/// and report the two phases separately. They have wildly different costs and
/// respond to different knobs, and rolling them into one number hides which one
/// is actually slow.
///
/// - `quality > 0`: lossy encoding at the given quality (0-100)
/// - `quality <= 0`: lossless encoding
/// - `method`: libwebp effort, 0 (fastest) to 6 (smallest)
pub fn encode_rgb_to_webp(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    quality: f32,
    method: i32,
) -> Result<Vec<u8>> {
    encode_webp(rgb_data, width, height, quality, method)
}

/// Encode raw RGB pixel data to PNG format.
pub fn encode_rgb_to_png(rgb_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    image::ImageEncoder::write_image(
        encoder,
        rgb_data,
        width,
        height,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(buf)
}

fn encode_webp(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    quality: f32,
    method: i32,
) -> Result<Vec<u8>> {
    let mut config =
        WebPConfig::new().map_err(|()| anyhow!("failed to initialize libwebp config"))?;
    config.method = method;

    if quality <= 0.0 {
        config.lossless = 1;
        config.alpha_compression = 0;
        config.quality = 75.0;
    } else {
        config.lossless = 0;
        config.alpha_compression = 1;
        config.quality = quality;
    }

    let encoded = Encoder::from_rgb(rgb_data, width, height)
        .encode_advanced(&config)
        .map_err(|e| anyhow!("libwebp encoding failed: {e:?}"))?;

    Ok(encoded.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(width: u32, height: u32) -> Vec<u8> {
        (0..(width as usize * height as usize * 3))
            .map(|i| (i % 251) as u8)
            .collect()
    }

    #[test]
    fn small_images_pass_through_unchanged() {
        let data = rgb(4, 4);
        let (out, w, h) = downscale_to_cap(data.clone(), 4, 4, FilterType::Lanczos3);
        assert_eq!((w, h), (4, 4));
        assert_eq!(out, data, "an image under the cap must not be resampled");
    }

    #[test]
    fn oversized_images_are_clamped_to_max_dimension() {
        // Deliberately thin so the test stays cheap: the interesting part is the
        // scale math, not the pixel count.
        let (out, w, h) = downscale_to_cap(
            rgb(MAX_DIMENSION * 2, 6),
            MAX_DIMENSION * 2,
            6,
            FilterType::Lanczos3,
        );
        assert_eq!((w, h), (MAX_DIMENSION, 3), "aspect ratio must be preserved");
        assert_eq!(out.len(), w as usize * h as usize * 3);
    }

    #[test]
    fn oversized_height_is_clamped_too() {
        let (out, w, h) = downscale_to_cap(
            rgb(6, MAX_DIMENSION * 2),
            6,
            MAX_DIMENSION * 2,
            FilterType::Lanczos3,
        );
        assert_eq!((w, h), (3, MAX_DIMENSION));
        assert_eq!(out.len(), w as usize * h as usize * 3);
    }

    #[test]
    fn halve_averages_each_2x2_block() {
        // One 2x2 block per channel: values 0, 10, 20, 30 -> mean 15.
        let src = vec![
            0, 0, 0, 10, 10, 10, // row 0
            20, 20, 20, 30, 30, 30, // row 1
        ];
        assert_eq!(halve(&src, 2, 2), vec![15, 15, 15]);
    }

    #[test]
    fn exact_power_of_two_ratios_use_only_halving() {
        // 2x the cap in both axes -> one halve lands exactly on the target, so the
        // resampler must not run at all. Verified by matching halve() bit for bit.
        let (w, h) = (MAX_DIMENSION * 2, 4);
        let src = rgb(w, h);
        let (out, ow, oh) = downscale_to_cap(src.clone(), w, h, FilterType::Lanczos3);
        assert_eq!((ow, oh), (MAX_DIMENSION, 2));
        assert_eq!(out, halve(&src, w, h), "should be pure box halving");
    }

    #[test]
    fn non_power_of_two_ratios_still_hit_the_cap() {
        // 1.5x the cap: no halving step is possible, so this goes through Lanczos3.
        let (w, h) = (MAX_DIMENSION + MAX_DIMENSION / 2, 6);
        let (out, ow, oh) = downscale_to_cap(rgb(w, h), w, h, FilterType::Lanczos3);
        assert_eq!(ow, MAX_DIMENSION);
        assert_eq!(out.len(), ow as usize * oh as usize * 3);
    }

    #[test]
    fn halving_never_overshoots_the_target() {
        // 4x the cap -> two halves, exactly on target, no resample.
        let (w, h) = (MAX_DIMENSION * 4, 8);
        let (_, ow, oh) = downscale_to_cap(rgb(w, h), w, h, FilterType::Lanczos3);
        assert_eq!((ow, oh), (MAX_DIMENSION, 2));
    }

    #[test]
    fn webp_encode_round_trips_dimensions() {
        let webp = encode_rgb_to_webp(&rgb(64, 32), 64, 32, 50.0, 4).expect("encode failed");
        let decoded = webp::Decoder::new(&webp).decode().expect("decode failed");
        assert_eq!((decoded.width(), decoded.height()), (64, 32));
    }

    #[test]
    fn webp_encode_downscales_oversized_input() {
        let w = MAX_DIMENSION * 2;
        let (data, dw, dh) = downscale_to_cap(rgb(w, 6), w, 6, FilterType::Lanczos3);
        let webp = encode_rgb_to_webp(&data, dw, dh, 50.0, 4).expect("encode failed");
        let decoded = webp::Decoder::new(&webp).decode().expect("decode failed");
        assert_eq!((decoded.width(), decoded.height()), (MAX_DIMENSION, 3));
    }
}
