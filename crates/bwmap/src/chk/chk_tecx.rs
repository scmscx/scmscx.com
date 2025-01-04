use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Hybrid (in Expansion mode) and Brood War. Not required for Melee.
// Validation: Must be size of 396 bytes.
// In Brood War scenarios this section replaces "TECS".

// This section is indentical to UNIS section except it uses the Brood War set of 44 technologies instead of the original 24.

#[derive(Debug, Serialize)]
pub struct ChkTecx<'a> {
    #[serde(skip_serializing)]
    pub technology_uses_default_settings: &'a [u8; 44],
    #[serde(skip_serializing)]
    pub mineral_cost: &'a [u16; 44],
    #[serde(skip_serializing)]
    pub gas_cost: &'a [u16; 44],
    #[serde(skip_serializing)]
    pub time: &'a [u16; 44],
    #[serde(skip_serializing)]
    pub energy_cost_to_cast: &'a [u16; 44],
}

pub(crate) fn parse_tecx<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkTecx<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkTecx {
        technology_uses_default_settings: slicer.extract_ref()?,
        mineral_cost: slicer.extract_ref()?,
        gas_cost: slicer.extract_ref()?,
        time: slicer.extract_ref()?,
        energy_cost_to_cast: slicer.extract_ref()?,
    })
}
