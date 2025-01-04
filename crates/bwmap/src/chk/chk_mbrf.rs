use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions. Not required for Melee.
// Validation: Must be a multiple of 2400 bytes.

// This section contains all of the mission briefings shown by the players.

// See "TRIG" section for format, as it is exactly the same except for the conditions. In this section the 16 conditions are still there, they are all null except for the very first condition, which only has a condition byte of 13. See #Mission Briefing Actions List for the action bytes for the mission briefing actions.

// This section can be split. Additional MBRF sections will add more briefing triggers.

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ChkMbrfCondition {
    pub location: u32,
    pub group: u32,
    pub qualified_number: u32,
    pub unit_id: u16,
    pub numeric_comparison_or_switch_state: u8,
    pub condition: u8,
    pub resource_type_or_score_type_or_switch_number: u8,
    pub flags: u8,
    pub mask_flag: u16,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ChkMbrfAction {
    pub location: u32,
    pub string_number: u32,
    pub wav_string_number: u32,
    pub seconds_or_milliseconds: u32,
    pub first_or_only_group_or_player_affected: u32,
    pub second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number:
        u32,
    pub unit_type_or_score_type_or_resource_type_or_alliance_status: u16,
    pub action: u8,
    pub number_of_units_or_action_state_or_unit_order_or_number_modifier: u8,
    pub flags: u8,
    pub padding: u8,
    pub mask_flag: u16,
}

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ChkMbrfIndividual {
    pub conditions: [ChkMbrfCondition; 16],
    #[serde(skip)]
    pub actions: [ChkMbrfAction; 64],
    pub execution_flags: u32,
    pub executed_for_player: [u8; 27],
    pub current_action: u8,
}

#[derive(Debug, Serialize)]
pub struct ChkMbrf {
    pub triggers: Vec<ChkMbrfIndividual>,
}

pub(crate) fn parse_mbrf(chunks: &[RiffChunk]) -> Result<ChkMbrf, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut triggers: Vec<ChkMbrfIndividual> = Vec::new();

    for chunk in chunks {
        let mut slicer = CursorSlicer::new(chunk.data);
        triggers.extend_from_slice(slicer.extract_rest_as_slice_lax()?);
    }

    Ok(ChkMbrf { triggers })
}
