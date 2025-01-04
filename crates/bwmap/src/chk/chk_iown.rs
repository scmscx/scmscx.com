use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Not Required.

// This section specifies the owner of each player.

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
// This section is separate from OWNR as a staredit value. Staredit does not display "inactive" as a valid option. Italicized settings denote invalid map options, which may involve a buffer overflow.

#[derive(Debug, Serialize)]
pub struct ChkIown {
    pub player_owner: [u8; 12],
}

pub(crate) fn parse_iown(chunks: &[RiffChunk<'_>]) -> Result<ChkIown, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkIown {
        player_owner: slicer.extract_u8_array_lax(),
    })
}
