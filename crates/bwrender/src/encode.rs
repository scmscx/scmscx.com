use anyhow::Result;
use image::imageops::FilterType;
use image::RgbImage;
use webp::Encoder;

const MAX_DIMENSION: u32 = 8192;

/// Downscale an RGB image so that neither dimension exceeds `MAX_DIMENSION`,
/// preserving the aspect ratio.  Uses Lanczos3 resampling for high quality.
/// Returns the (possibly unchanged) image data, width, and height.
fn downscale_if_needed(rgb_data: &[u8], width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    if width <= MAX_DIMENSION && height <= MAX_DIMENSION {
        return (rgb_data.to_vec(), width, height);
    }

    let scale = f64::min(
        MAX_DIMENSION as f64 / width as f64,
        MAX_DIMENSION as f64 / height as f64,
    );
    let new_width = (width as f64 * scale).round() as u32;
    let new_height = (height as f64 * scale).round() as u32;

    let img = RgbImage::from_raw(width, height, rgb_data.to_vec())
        .expect("RGB data length must match width * height * 3");

    let resized = image::imageops::resize(&img, new_width, new_height, FilterType::Lanczos3);
    let out = resized.into_raw();
    (out, new_width, new_height)
}

/// Encode raw RGB pixel data to WebP format.
///
/// Images larger than 4096px in either dimension are downscaled with Lanczos3
/// resampling before encoding.
///
/// - `quality > 0`: lossy encoding at the given quality (0-100)
/// - `quality <= 0`: lossless encoding
pub fn encode_rgb_to_webp(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    quality: f32,
) -> Result<Vec<u8>> {
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        let (data, w, h) = downscale_if_needed(rgb_data, width, height);
        encode_webp(&data, w, h, quality)
    } else {
        encode_webp(rgb_data, width, height, quality)
    }
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

fn encode_webp(rgb_data: &[u8], width: u32, height: u32, quality: f32) -> Result<Vec<u8>> {
    let encoder = Encoder::from_rgb(rgb_data, width, height);

    let encoded = if quality <= 0.0 {
        encoder.encode_lossless()
    } else {
        encoder.encode(quality)
    };

    Ok(encoded.to_vec())
}
