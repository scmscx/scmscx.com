use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Vanilla and Hybrid (in Original mode). Not required for Melee.
// Validation: Must be size of 598 bytes.
// In Brood War scenarios, this section is replaced by "UPGx".

// This section contains upgrade settings:

// u8[46]: 1 byte per each upgrade, in order of upgrade id.
// 00 - Upgrade uses custom settings
// 01 - Upgrade uses default settings
// u16[46]: 1 integer per upgrade, base mineral cost for each upgrade, in order of upgrade id.
// u16[46]: 1 integer per upgrade, mineral cost factor for each upgrade, in order of upgrade id.
// u16[46]: 1 integer per upgrade, base gas cost for each upgrade, in order of upgrade id.
// u16[46]: 1 integer per upgrade, gas cost factor for each upgrade, in order of upgrade id.
// u16[46]: 1 integer per upgrade, base time for each upgrade, in order of upgrade id.
// u16[46]: 1 integer per upgrade, gas time factor for each upgrade, in order of upgrade id.
// See #List of Upgrade IDs for upgrade ids.

#[derive(Debug, Serialize)]
pub struct ChkUpgs<'a> {
    #[serde(skip_serializing)]
    pub upgrade_uses_default_setings: &'a [u8; 46],
    #[serde(skip_serializing)]
    pub base_mineral_cost: &'a [u16; 46],
    #[serde(skip_serializing)]
    pub mineral_cost_factor: &'a [u16; 46],
    #[serde(skip_serializing)]
    pub base_gas_cost: &'a [u16; 46],
    #[serde(skip_serializing)]
    pub gas_cost_factor: &'a [u16; 46],
    #[serde(skip_serializing)]
    pub base_time: &'a [u16; 46],
    #[serde(skip_serializing)]
    pub time_factor: &'a [u16; 46],
}

pub(crate) fn parse_upgs<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkUpgs<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkUpgs {
        upgrade_uses_default_setings: slicer.extract_ref()?,
        base_mineral_cost: slicer.extract_ref()?,
        mineral_cost_factor: slicer.extract_ref()?,
        base_gas_cost: slicer.extract_ref()?,
        gas_cost_factor: slicer.extract_ref()?,
        base_time: slicer.extract_ref()?,
        time_factor: slicer.extract_ref()?,
    })
}
