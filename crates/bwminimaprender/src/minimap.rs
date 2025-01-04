use anyhow::Result;
use image::ImageEncoder;
use lazy_static::lazy_static;
use std::collections::hash_map::HashMap;
use tracing::instrument;

lazy_static! {
    pub static ref TILESETS: HashMap<u16, std::collections::HashMap<u16, [u8; 3]>> = {
        fn parse_lst_file(src: &str) -> std::collections::HashMap<u16, [u8; 3]> {
            let mut ret = std::collections::HashMap::new();

            for line in src.split('\n') {
                if line.is_empty() {
                    continue;
                }

                let split: Vec<&str> = line.split('\t').collect();

                if split.len() != 5 {
                    continue;
                }

                let id = split[0].parse::<u16>().unwrap();
                let rgb: Vec<u8> = split[1][1..split[1].len() - 1]
                    .split(',')
                    .map(|x| x.parse::<u8>().unwrap())
                    .collect();

                ret.insert(id, [rgb[0], rgb[1], rgb[2]]);
            }

            ret
        }

        let mut m = HashMap::new();
        m.insert(0, parse_lst_file(include_str!("external/lst/remaster/0")));
        m.insert(1, parse_lst_file(include_str!("external/lst/remaster/1")));
        m.insert(2, parse_lst_file(include_str!("external/lst/remaster/2")));
        m.insert(3, parse_lst_file(include_str!("external/lst/remaster/3")));
        m.insert(4, parse_lst_file(include_str!("external/lst/remaster/4")));
        m.insert(5, parse_lst_file(include_str!("external/lst/remaster/5")));
        m.insert(6, parse_lst_file(include_str!("external/lst/remaster/6")));
        m.insert(7, parse_lst_file(include_str!("external/lst/remaster/7")));
        m
    };
}

// const fn era_as_str(era: u16) -> &'static str {
//     match era % 8 {
//         0 => "Badlands",
//         1 => "Space Platform",
//         2 => "Installation",
//         3 => "Ashworld",
//         4 => "Jungle",
//         5 => "Desert",
//         6 => "Arctic",
//         7 => "Twilight",
//         _ => unreachable!(),
//     }
// }

// #[instrument(skip(mtxm), fields(tileset = era_as_str(era)), err)]
pub fn render_minimap(mtxm: &[u16], width: usize, height: usize, era: u16) -> Result<Vec<u8>> {
    let mut png = Vec::<u8>::new();

    {
        let mut img: image::RgbImage = image::ImageBuffer::new(width as u32, height as u32);
        let tileset = era % 8;
        let tileset_map = TILESETS.get(&tileset).unwrap();

        for y in 0..height {
            for x in 0..width {
                let offset = y * width + x;
                let tile_id = if offset < mtxm.len() { mtxm[offset] } else { 0 };

                let rgb = tileset_map.get(&tile_id);

                if let Some(rgb) = rgb {
                    img.put_pixel(x as u32, y as u32, image::Rgb(*rgb));
                } else {
                    img.put_pixel(x as u32, y as u32, image::Rgb([0, 0, 0]));
                }
            }
        }

        image::codecs::png::PngEncoder::new(&mut png).write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgb8,
        )?;
    }

    Ok(png)
}

#[instrument(skip(png), err)]
pub fn calculate_perceptual_hash(png: &[u8]) -> Result<[u8; 16 * 16 / 8]> {
    use image::ImageDecoder;

    let cursor = std::io::Cursor::new(png);

    let png = image::codecs::png::PngDecoder::new(cursor)?;
    anyhow::ensure!(png.color_type() == image::ColorType::Rgb8);

    let (x, y) = png.dimensions();

    let mut image_data = vec![0; png.total_bytes() as usize];
    png.read_image(image_data.as_mut_slice())?;

    let image: image::ImageBuffer<image::Rgb<u8>, _> =
        image::ImageBuffer::from_vec(x, y, image_data).unwrap();

    let ph16x16 = image::imageops::grayscale(&image::imageops::resize(
        &image,
        16,
        16,
        image::imageops::Lanczos3,
    ));

    let ph16x16_sum = ph16x16.iter().fold(0, |acc, x| acc + (*x as usize));
    let ph16x16_avg = (ph16x16_sum / (16 * 16)) as u8;

    let ph16x16: Vec<u8> = ph16x16
        .iter()
        .map(|x| if *x < ph16x16_avg { 0 } else { 1 })
        .collect::<Vec<u8>>()
        .chunks_exact(8)
        .map(|x| x.iter().fold(0u8, |acc, x| acc << 1 | *x))
        .collect();

    anyhow::Ok(ph16x16.as_slice().try_into()?)
}
