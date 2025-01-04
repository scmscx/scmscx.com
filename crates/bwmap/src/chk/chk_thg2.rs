use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be a multiple of 10 bytes.

// The map editor only writes to this section. Starcraft uses this section.

// Sprites, usually on doodads, get the following structure.

// u16: Unit/Sprite number of the sprite
// u16: X coordinate of the doodad unit
// u16: Y coordinate of the doodad unit
// u8: Player number that owns the doodad
// u8: Unused
// u16: Flags
// Bit 0-11 - Unused
// Bit 12 - Draw as sprite (Determines if it is a sprite or a unit sprite)
// Bit 13 - Unused
// Bit 14 - Unused
// Bit 15 - Disabled (Only valid if Draw as sprite is unchecked, disables the unit)
// This section can be split. Additional THG2 sections will add more sprites.

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ChkThg2Individual {
    pub sprite_number: u16,
    pub x: u16,
    pub y: u16,
    pub owner: u8,
    pub unused: u8,
    pub flags: u16,
}

#[derive(Debug, Serialize)]
pub struct ChkThg2 {
    pub sprites: Vec<ChkThg2Individual>,
}

pub(crate) fn parse_thg2(chunks: &[RiffChunk]) -> Result<ChkThg2, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut sprites: Vec<ChkThg2Individual> = Vec::new();

    for chunk in chunks {
        let mut slicer = CursorSlicer::new(chunk.data);
        sprites.extend_from_slice(slicer.extract_rest_as_slice_lax()?);
    }

    Ok(ChkThg2 { sprites })
}
