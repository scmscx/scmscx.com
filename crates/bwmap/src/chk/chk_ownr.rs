use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be size of 12 bytes.

// This section designates the controller of a particular player. It is exactly the same as the "IOWN" section, except there is an additional value, 0x00 for Inactive.

// u8[12]: One byte for each player, specifies the owner of the player:
// 00 - Inactive
// 01 - Computer (game)
// 02 - Occupied by Human Player
// 03 - Rescue Passive
// 04 - Unused
// 05 - Computer
// 06 - Human (Open Slot)
// 07 - Neutral
// 08 - Closed slot
// Italicized settings denote invalid map options, which may involve a buffer overflow.

#[derive(Debug, Serialize)]
pub struct ChkOwnr {
    pub player_owner: [u8; 12],
}

pub(crate) fn parse_ownr(chunks: &[RiffChunk<'_>]) -> Result<ChkOwnr, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkOwnr {
        player_owner: slicer.extract_u8_array_lax(),
    })
}
