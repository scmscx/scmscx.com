use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Hybrid (in Expansion mode) and Brood War. Not required for Melee.
// Validation: Must be size of 4168 bytes.
// In Brood War scenarios this section replaces "UNIS".

// This section is indentical to UNIS section except it uses the Brood War set of 130 weapons instead of the original 100.

#[derive(Debug, Serialize)]
pub struct ChkUnix {
    #[serde(skip_serializing)]
    pub config: [u8; 228],
    #[serde(skip_serializing)]
    pub hit_points: [u32; 228],
    #[serde(skip_serializing)]
    pub shield_points: [u16; 228],
    #[serde(skip_serializing)]
    pub armor_points: [u8; 228],
    #[serde(skip_serializing)]
    pub build_time: [u16; 228],
    #[serde(skip_serializing)]
    pub mineral_cost: [u16; 228],
    #[serde(skip_serializing)]
    pub gas_cost: [u16; 228],
    #[serde(skip_serializing)]
    pub string_number: [u16; 228],
    #[serde(skip_serializing)]
    pub base_weapon_damage: [u16; 130],
    #[serde(skip_serializing)]
    pub upgrade_bonus_weapon_damage: [u16; 130],
}

pub(crate) fn parse_unix(chunks: &[RiffChunk]) -> Result<ChkUnix, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkUnix {
        config: slicer.extract()?,
        hit_points: slicer.extract()?,
        shield_points: slicer.extract()?,
        armor_points: slicer.extract()?,
        build_time: slicer.extract()?,
        mineral_cost: slicer.extract()?,
        gas_cost: slicer.extract()?,
        string_number: slicer.extract()?,
        base_weapon_damage: slicer.extract()?,
        upgrade_bonus_weapon_damage: slicer.extract()?,
    })
}
