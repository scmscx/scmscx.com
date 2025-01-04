use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Hybrid (in Expansion mode) and Brood War. Not required for Melee.
// Validation: Must be size of 1672 bytes.
// In Brood War scenarios this section replaces "PTEC".

// This section is identical to PTEC section except it uses the Brood War set of 44 technologies instead of the original 24.

#[derive(Debug, Serialize)]
pub struct ChkPtex<'a> {
    #[serde(skip_serializing)]
    pub player_availability: &'a [[u8; 44]; 12],
    #[serde(skip_serializing)]
    pub already_researched: &'a [[u8; 44]; 12],
    #[serde(skip_serializing)]
    pub global_availability_defaults: &'a [u8; 44],
    #[serde(skip_serializing)]
    pub global_already_researched_defaults: &'a [u8; 44],
    #[serde(skip_serializing)]
    pub player_uses_default: &'a [[u8; 44]; 12],
}

pub(crate) fn parse_ptex<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkPtex<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkPtex {
        player_availability: slicer.extract_ref()?,
        already_researched: slicer.extract_ref()?,
        global_availability_defaults: slicer.extract_ref()?,
        global_already_researched_defaults: slicer.extract_ref()?,
        player_uses_default: slicer.extract_ref()?,
    })
}
