use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Hybrid (in Expansion mode) and Brood War. Not required for Melee.
// Validation: Must be size of 2318 bytes.
// In Brood War scenarios this section replaces "UPGR".

// This section is identical to UPGR section except it uses the Brood War set of 61 upgrades instead of the original 46.

#[derive(Debug, Serialize)]
pub struct ChkPupx<'a> {
    #[serde(skip_serializing)]
    pub max_upgrade_level: &'a [[u8; 61]; 12],
    #[serde(skip_serializing)]
    pub starting_upgrade_level: &'a [[u8; 61]; 12],
    #[serde(skip_serializing)]
    pub global_default_maximum_upgrade_level: &'a [u8; 61],
    #[serde(skip_serializing)]
    pub global_default_starting_upgrade_level: &'a [u8; 61],
    #[serde(skip_serializing)]
    pub player_uses_upgrade_defaults: &'a [[u8; 61]; 12],
}

pub(crate) fn parse_pupx<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkPupx<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkPupx {
        max_upgrade_level: slicer.extract_ref()?,
        starting_upgrade_level: slicer.extract_ref()?,
        global_default_maximum_upgrade_level: slicer.extract_ref()?,
        global_default_starting_upgrade_level: slicer.extract_ref()?,
        player_uses_upgrade_defaults: slicer.extract_ref()?,
    })
}
