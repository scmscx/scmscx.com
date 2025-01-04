use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Vanilla and Hybrid (in Original mode). Not required for Melee.
// Validation: Must be size of 912 bytes.
// In Brood War scenarios, this section is replaced by "PTEx".

// This section contains player technology availability restrictions: it indicates whether a player starts off with a particular technology or can research a particular technology.

// u8[24][12]: 1 byte for each technology id, then each player, for player availability:
// 00 - Technology is not available for a player
// 01 - Technology is available for a player
// u8[24][12]: 1 byte for each technology id, then each player, for "already researched" status:
// 00 - Technology is not already researched
// 01 - Technology is already researched
// u8[24]: 1 byte for each technology for global availability defaults:
// 00 - Technology is not available by default
// 01 - Technology is available by default
// u8[24]: 1 byte for each technology for global "already researched" defaults:
// 00 - Technology is not already researched by default
// 01 - Technology is already researched by default
// u8[24][12]: 1 byte for each technology in order of its technology id, then each player, indicating whether a player uses the global defaults:
// 00 - Technology overrides defaults for player
// 01 - Technology uses default settings for player
// See #List of Technology IDs.

#[derive(Debug, Serialize)]
pub struct ChkPtec<'a> {
    pub player_availability: &'a [[u8; 24]; 12],
    pub already_researched: &'a [[u8; 24]; 12],
    pub global_availability_defaults: &'a [u8; 24],
    pub global_already_researched_defaults: &'a [u8; 24],
    pub player_uses_default: &'a [[u8; 24]; 12],
}

pub(crate) fn parse_ptec<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkPtec<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkPtec {
        player_availability: slicer.extract_ref()?,
        already_researched: slicer.extract_ref()?,
        global_availability_defaults: slicer.extract_ref()?,
        global_already_researched_defaults: slicer.extract_ref()?,
        player_uses_default: slicer.extract_ref()?,
    })
}
