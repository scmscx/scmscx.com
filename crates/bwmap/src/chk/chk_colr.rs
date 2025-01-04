use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Brood War only and all game types.
// Can be in non-Brood War maps in Remaster, if CRGB is also present.
// Validation: Must be size of 8 bytes.

// This section indicates what color each player is, but only has effect on Brood War.

// u8[8]: 1 byte for each player, indicates the color of the player
// 00 - Red
// 01 - Blue
// 02 - Teal
// 03 - Purple
// 04 - Orange
// 05 - Brown
// 06 - White
// 07 - Yellow
// 08 - Green
// 09 - Pale yellow
// 10 - Tan
// 11 - Azure (Neutral color)
// Other values can be used but may have different results depending on the tileset. Any color value above 11 is an overflow. As of patch 1.18.6 values above 11 result in default player color.

// SC:R Only colors: Selecting these in a non-SC:R map seems to just use default colors, unless the CRGB section is also present regardless of version (including non-BW versions).

// 12 - Pale Green
// 13 - Blueish Grey
// 14 - Pale Yellow
// 15 - Cyan
// 16 - Pink
// 17 - Olive
// 18 - Lime
// 19 - Navy
// 20 - Magenta
// 21 - Grey
// 22 - Black

#[derive(Debug, Serialize)]
pub struct ChkColr {
    pub player_color: [u8; 8],
}

pub(crate) fn parse_colr(chunks: &[RiffChunk<'_>]) -> Result<ChkColr, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkColr {
        player_color: slicer.extract()?,
    })
}
