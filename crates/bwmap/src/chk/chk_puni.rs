use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions. Not required for Melee.
// Validation: Must be size of 5700 bytes.

// This section contains player unit restrictions: it indicates whether the player can or cannot build a particular unit.

// u8[228][12]: 1 byte for each unit in order of its unit id, then each player, for player availability:
// 00 - That unit is not available for production if the player has 'override defaults' on
// 01 - That unit is available for production if the player has 'override defaults' on
// u8[228]: 1 byte for each unit in order of it's unit id, for global availability defaults (for all players):
// 00 - That unit is not available for production
// 01 - That unit is available for production
// u8[228][12]: 1 byte for each unit in order of its unit id, then each player, indicating whether a player uses the global availability defaults.
// 00 - Player overrides defaults for this unit
// 01 - Player uses defaults for this unit

#[derive(Debug, Serialize)]
pub struct ChkPuni<'a> {
    #[serde(skip_serializing)]
    pub unit_player_availability: &'a [[u8; 228]; 12],
    #[serde(skip_serializing)]
    pub unit_global_availability: &'a [u8; 228],
    #[serde(skip_serializing)]
    pub unit_player_uses_defaults: &'a [[u8; 228]; 12],
}

pub(crate) fn parse_puni<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkPuni<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkPuni {
        unit_player_availability: slicer.extract_ref()?,
        unit_global_availability: slicer.extract_ref()?,
        unit_player_uses_defaults: slicer.extract_ref()?,
    })
}
