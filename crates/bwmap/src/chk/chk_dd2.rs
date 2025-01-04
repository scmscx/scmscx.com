use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Not Required.

// This section contains the doodad map of the level. There are several parts to doodads. The graphical portion is stored in the MTXM section. The second part of the doodad is stored in this section. This section is only used by the map editor.

// Each doodad in the map gets the following structure:

// u16: Number of the doodad. Size of the doodad is dependent on this. Doodads are different for each tileset.
// u16: X coordinate of the doodad unit
// u16: Y coordinate of the doodad unit
// u8: Player number that owns the doodad
// u8: Enabled flag
// 00 - Doodad is enabled (trap can attack, door is closed, etc)
// 01 - Doodad is disabled

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ChkDd2Individual {
    pub doodad_number: u16,
    pub x: u16,
    pub y: u16,
    pub owner: u8,
    pub disabled: u8,
}

#[derive(Debug, Serialize)]
pub struct ChkDd2 {
    pub doodads: Vec<ChkDd2Individual>,
}

pub(crate) fn parse_dd2(chunks: &[RiffChunk]) -> Result<ChkDd2, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkDd2 {
        doodads: slicer.extract_rest_as_slice_lax()?.to_vec(),
    })
}
