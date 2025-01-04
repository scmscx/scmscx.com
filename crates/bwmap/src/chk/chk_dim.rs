use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be size of 4 bytes.

// This section contains the dimensions of the map.

// u16: Width of the map
// u16: Height of the map
// The Width/Height of the map is measured in the number of square 32x32p tiles.
// Standard Dimensions are 64, 96, 128, 192, and 256.

#[derive(Debug, Serialize)]
pub struct ChkDim<'a> {
    pub width: &'a u16,
    pub height: &'a u16,
}

pub(crate) fn parse_dim<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkDim<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkDim {
        width: slicer.extract_ref()?,
        height: slicer.extract_ref()?,
    })
}
