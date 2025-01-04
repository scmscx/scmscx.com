use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Requirement unknown, but can be in any map type including non-remaster maps.
// Validation: Must be size of 32 bytes.
// u8[8][3]: 3 bytes for each player corresponding to an RGB color value
// u8: R
// u8: G
// u8: B
// u8[8]: Indicates how to select the color for each player
// 0 - Random Predefined
// 1 - Player's Choice
// 2 - Custom RGB Color (RGB value for the corresponding player defined above)
// 3 - Use "COLR" selection

#[derive(Debug, Serialize)]
pub struct ChkCrgb {
    pub rgb: [[u8; 3]; 8],
    pub player_color_option: [u8; 8],
}

pub(crate) fn parse_crgb(chunks: &[RiffChunk<'_>]) -> Result<ChkCrgb, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkCrgb {
        rgb: slicer.extract()?,
        player_color_option: slicer.extract()?,
    })
}
