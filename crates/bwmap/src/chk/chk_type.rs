use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Not required.

// This section specifies the type of scenario.

// u32: Specifies the type of scenario:
// 0x53574152 or RAWS - 1.04 StarCraft and above ("hybrid")
// 0x42574152 or RAWB - Brood War

#[derive(Debug, Serialize)]
pub struct ChkType<'a> {
    pub scenario_type: &'a u32,
}

pub(crate) fn parse_type<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkType<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkType {
        scenario_type: slicer.extract_ref()?,
    })
}
