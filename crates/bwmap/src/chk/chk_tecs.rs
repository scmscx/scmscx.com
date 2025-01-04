use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Vanilla and Hybrid (in Original mode). Not required for Melee.
// Validation: Must be size of 216 bytes.
// In Brood War scenarios, this section is replaced by "TECx".

// This section contains technology/special abilities settings:

// u8[24]: 1 byte per each technology, specifies if the tech overrides the default settings. In order of technology id
// 00 - Technology uses custom settings
// 01 - Technology uses default settings
// u16[24]: Mineral cost to develop technology. In order of technology id.
// u16[24]: Gas cost to develop technology. In order of technology id.
// u16[24]: Time required to develop technology. In order of technology id.
// u16[24]: Energy cost to cast technology/special ability. In order of technology id.
// See #List of Technology IDs for technology ids.

#[derive(Debug, Serialize)]
pub struct ChkTecs<'a> {
    pub technology_uses_default_settings: &'a [u8; 24],
    pub mineral_cost: &'a [u16; 24],
    pub gas_cost: &'a [u16; 24],
    pub time: &'a [u16; 24],
    pub energy_cost_to_cast: &'a [u16; 24],
}

pub(crate) fn parse_tecs<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkTecs<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkTecs {
        technology_uses_default_settings: slicer.extract_ref()?,
        mineral_cost: slicer.extract_ref()?,
        gas_cost: slicer.extract_ref()?,
        time: slicer.extract_ref()?,
        energy_cost_to_cast: slicer.extract_ref()?,
    })
}
