use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions. Not required for Melee.
// Validation: This section will always validate.

// This section contains the data on fog of war for each player. This is whether at the start of the game that levels of black space that is available.

// u8[ map width * height ]: One byte for each map tile. The bits indicate for each player the fog of war.
// Bit 0 - Player 1's Fog of War. If on, the tile is covered with fog. if off, the tile is visible.
// Bit 1 - Player 2's Fog of War. If on, the tile is covered with fog. if off, the tile is visible.
// Bit 2 - Player 3's Fog of War. If on, the tile is covered with fog. if off, the tile is visible.
// Bit 3 - Player 4's Fog of War. If on, the tile is covered with fog. if off, the tile is visible.
// Bit 4 - Player 5's Fog of War. If on, the tile is covered with fog. if off, the tile is visible.
// Bit 5 - Player 6's Fog of War. If on, the tile is covered with fog. if off, the tile is visible.
// Bit 6 - Player 7's Fog of War. If on, the tile is covered with fog. if off, the tile is visible.
// Bit 7 - Player 8's Fog of War. If on, the tile is covered with fog. if off, the tile is visible.
// Any size greater than width*height will be ignored. Any size less will default missing tiles to 0xFF

#[derive(Debug, Serialize)]
pub struct ChkMask<'a> {
    pub fog: &'a [u8],
}

pub(crate) fn parse_mask<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkMask<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkMask {
        fog: slicer.extract_rest_as_slice()?,
    })
}
