use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// u8[228]: 1 byte for each unit, in order of Unit ID
// 00 - Unit does not use default settings
// 01 - Unit does use default settings
// u32[228]: Hit points for unit (Note the displayed value is this value / 256, with the low byte being a fractional HP value)
// u16[228]: Shield points, in order of Unit ID
// u8[228]: Armor points, in order of Unit ID
// u16[228]: Build time (1/60 seconds), in order of Unit ID
// u16[228]: Mineral cost, in order of Unit ID
// u16[228]: Gas cost, in order of Unit ID
// u16[228]: String number, in order of Unit ID
// u16[100]: Base weapon damage the weapon does, in weapon ID order (#List of Unit Weapon IDs)
// u16[100]: Upgrade bonus weapon damage, in weapon ID order

#[derive(Debug, Serialize)]
pub struct ChkUnis<'a> {
    #[serde(skip_serializing)]
    pub config: &'a [u8; 228],
    #[serde(skip_serializing)]
    pub hit_points: &'a [u32; 228],
    #[serde(skip_serializing)]
    pub shield_points: &'a [u16; 228],
    #[serde(skip_serializing)]
    pub armor_points: &'a [u8; 228],
    #[serde(skip_serializing)]
    pub build_time: &'a [u16; 228],
    #[serde(skip_serializing)]
    pub mineral_cost: &'a [u16; 228],
    #[serde(skip_serializing)]
    pub gas_cost: &'a [u16; 228],
    #[serde(skip_serializing)]
    pub string_number: &'a [u16; 228],
    #[serde(skip_serializing)]
    pub base_weapon_damage: &'a [u16; 100],
    #[serde(skip_serializing)]
    pub upgrade_bonus_weapon_damage: &'a [u16; 100],
}

pub(crate) fn parse_unis<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkUnis<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkUnis {
        config: slicer.extract_ref()?,
        hit_points: slicer.extract_ref()?,
        shield_points: slicer.extract_ref()?,
        armor_points: slicer.extract_ref()?,
        build_time: slicer.extract_ref()?,
        mineral_cost: slicer.extract_ref()?,
        gas_cost: slicer.extract_ref()?,
        string_number: slicer.extract_ref()?,
        base_weapon_damage: slicer.extract_ref()?,
        upgrade_bonus_weapon_damage: slicer.extract_ref()?,
    })
}
