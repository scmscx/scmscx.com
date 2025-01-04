use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for Vanilla and Hybrid (in Original mode). Not required for Melee.
// Validation: Must be size of 1748 bytes.
// In Brood War scenarios, this section is replaced by "PUPx".

// This section contains player upgrade restrictions: it indicates the starting/maximum levels at/to which a player can perform a particular upgrade.

// u8[46][12]: 1 byte for the maximum level a player can upgrade to, in order of its upgrade id, then each player
// u8[46][12]: 1 byte for the level of an upgrade a player starts off with, in order of its upgrade id, then each player
// u8[46]: 1 byte for the global default maximum level of each upgrade, in order of its upgrade id.
// u8[46]: 1 byte for the global default starting level of each upgrade, in order of its upgrade id.
// u8[46][12]: 1 byte for each upgrade in order of its upgrade id, then each player, indicating whether a player uses the global upgrade defaults:
// 00 - Player does not use global upgrade defaults for this upgrade
// 01 - Player uses upgrade defaults for this upgrade
// See #List of Upgrade IDs.

#[derive(Debug, Serialize)]
pub struct ChkUpgr<'a> {
    #[serde(skip_serializing)]
    pub max_upgrade_level: &'a [[u8; 46]; 12],
    #[serde(skip_serializing)]
    pub starting_upgrade_level: &'a [[u8; 46]; 12],
    #[serde(skip)]
    pub global_default_maximum_upgrade_level: &'a [u8; 46],
    #[serde(skip)]
    pub global_default_starting_upgrade_level: &'a [u8; 46],
    #[serde(skip_serializing)]
    pub player_uses_upgrade_defaults: &'a [[u8; 46]; 12],
}

pub(crate) fn parse_upgr<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkUpgr<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkUpgr {
        max_upgrade_level: slicer.extract_ref()?,
        starting_upgrade_level: slicer.extract_ref()?,
        global_default_maximum_upgrade_level: slicer.extract_ref()?,
        global_default_starting_upgrade_level: slicer.extract_ref()?,
        player_uses_upgrade_defaults: slicer.extract_ref()?,
    })
}
