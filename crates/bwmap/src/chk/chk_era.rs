use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be size of 2 bytes.

// This section indicates the tileset of the scenario.

// u16: Designates tileset:
// 00 - Badlands
// 01 - Space Platform
// 02 - Installation
// 03 - Ashworld
// 04 - Jungle
// 05 - Desert
// 06 - Arctic
// 07 - Twilight
// StarCraft masks the tileset indicator's bit value, so bits after the third place (anything after the value "7") are removed. Thus, 9 (1001 in binary) is interpreted as 1 (0001), 10 (1010) as 2 (0010), etc.
// Desert, Arctic, and Twilight are Brood War-only tilesets.

#[derive(Debug, Serialize)]
pub struct ChkEra<'a> {
    pub tileset: &'a u16,
}

pub(crate) fn parse_era<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkEra<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkEra {
        tileset: slicer.extract_ref()?,
    })
}
