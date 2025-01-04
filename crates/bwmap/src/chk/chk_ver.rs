use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be size of 2 bytes.

// This section identifies the file format version.

// u16: File format version:
// 59 - Starcraft 1.00 (original retail release)
// 63 - Starcraft 1.04 (hybrid)
// 64 - Starcraft Remastered (1.21) (hybrid)
// 205 - Brood War 1.00 (1.04)
// 206 - Starcraft Remastered (1.21) (broodwar)
// This is the only version code section to actually be read by StarCraft (of TYPE, VER , IVER, and IVE2). Any other value is invalid in retail StarCraft and is usually a beta version.

// Other unsupported versions include:

// 0-19 - Warcraft II versions (.PUD)
// 47 - Starcraft Beta
// 61 - Brood War internal (map version 61)
// 75 - Brood War internal (map version 75)
// 201 - Brood War internal (map version 201)
// 203 - Brood War internal (map version 203)

#[derive(Debug, Serialize)]
pub struct ChkVer<'a> {
    pub file_format_version: &'a u16,
}

pub(crate) fn parse_ver<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkVer<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkVer {
        file_format_version: slicer.extract_ref()?,
    })
}
