use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Not required.

// This section "additionally identifies" the map version.

// u16: "Additional file format version:"
// 11 - 1.04 StarCraft and above ("hybrid") or Brood War.
// This section does not "replace" IVER in hybrid/Brood War scenarios: both seem to be written by StarEdit but not read by StarCraft.

#[derive(Debug, Serialize)]
pub struct ChkIve2<'a> {
    pub additional_file_format_version: &'a u16,
}

pub(crate) fn parse_ive2<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkIve2<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkIve2 {
        additional_file_format_version: slicer.extract_ref()?,
    })
}
