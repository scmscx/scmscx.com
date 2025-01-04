use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Not Required.

// This section goes along with the "UPRP" section. This section just indicates which of the 64 unit properties slot are used.

// u8[64]: 1 byte for each trigger unit properties slot
// 00 - Properties slot is unused
// 01 - Properties slot is used

#[derive(Debug, Serialize)]
pub struct ChkUpus<'a> {
    pub cuwp_slot_is_used: &'a [u8],
}

pub(crate) fn parse_upus<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkUpus<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkUpus {
        cuwp_slot_is_used: slicer.extract_rest_as_slice_lax()?,
    })
}
