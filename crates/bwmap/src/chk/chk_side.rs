use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be size of 12 bytes.

// This section contains the species/race of each player.

// u8[12]: 1 byte per player the species of that player:
// 00 - Zerg
// 01 - Terran
// 02 - Protoss
// 03 - Invalid (Independent), shown as "Unknown" in SC:R lobby
// 04 - Invalid (Neutral), shown as "Unknown"
// 05 - User Select
// 06 - Random (Forced; Acts as a selected race)
// 07 - Inactive, shown as locked "Select Race" option in SC:R lobby
// Italicized settings denote invalid map options. Note Players 9-11 are defaultly Inactive and Player 12 is defaultly Neutral.

#[derive(Debug, Serialize)]
pub struct ChkSide {
    pub player_side: [u8; 12],
}

pub(crate) fn parse_side(chunks: &[RiffChunk<'_>]) -> Result<ChkSide, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkSide {
        player_side: slicer.extract_u8_array_lax(),
    })
}
