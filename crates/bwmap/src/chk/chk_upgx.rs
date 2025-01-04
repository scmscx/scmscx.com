use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Hybrid (in Expansion mode) and Brood War. Not required for Melee.
// Validation: Must be size of 794 bytes.
// In Brood War scenarios this section replaces "UPGS".

// This section the same as UPGS except section except it uses the Brood War set of 61 upgrades instead of the original 46, and there is an unused byte after the first set:

// u8[61]: 1 byte per each upgrade, in order of upgrade id.
// 00 - Upgrade uses custom settings
// 01 - Upgrade uses default settings
// u8: Unused.
// u16[61]: 1 integer per upgrade, base mineral cost for each upgrade, in order of upgrade id.
// u16[61]: 1 integer per upgrade, mineral cost factor for each upgrade, in order of upgrade id.
// u16[61]: 1 integer per upgrade, base gas cost for each upgrade, in order of upgrade id.
// u16[61]: 1 integer per upgrade, gas cost factor for each upgrade, in order of upgrade id.
// u16[61]: 1 integer per upgrade, base time for each upgrade, in order of upgrade id.
// u16[61]: 1 integer per upgrade, gas time factor for each upgrade, in order of upgrade id.

#[derive(Debug, Serialize)]
pub struct ChkUpgx<'a> {
    #[serde(skip_serializing)]
    pub upgrade_uses_default_setings: &'a [u8; 61],
    pub unused: &'a u8,
    #[serde(skip_serializing)]
    pub base_mineral_cost: &'a [u16; 61],
    #[serde(skip_serializing)]
    pub mineral_cost_factor: &'a [u16; 61],
    #[serde(skip_serializing)]
    pub base_gas_cost: &'a [u16; 61],
    #[serde(skip_serializing)]
    pub gas_cost_factor: &'a [u16; 61],
    #[serde(skip_serializing)]
    pub base_time: &'a [u16; 61],
    #[serde(skip_serializing)]
    pub time_factor: &'a [u16; 61],
}

pub(crate) fn parse_upgx<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkUpgx<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkUpgx {
        upgrade_uses_default_setings: slicer.extract_ref()?,
        unused: slicer.extract_ref()?,
        base_mineral_cost: slicer.extract_ref()?,
        mineral_cost_factor: slicer.extract_ref()?,
        base_gas_cost: slicer.extract_ref()?,
        gas_cost_factor: slicer.extract_ref()?,
        base_time: slicer.extract_ref()?,
        time_factor: slicer.extract_ref()?,
    })
}
