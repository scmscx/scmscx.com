use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions. Not required for Melee.
// Validation: Must be size of 1280 bytes.

// This section is used whenever the create units with properties trigger is used. Since a slot has to be assigned to the action, this is where each slot is designated.

// There are 64 of the following structures regardless of how many are used and it cannot exceed 64.

// u16: Flag of which special properties can be applied to unit, and are valid.
// Bit 0 - Cloak bit is valid
// Bit 1 - Burrowed bit is valid
// Bit 2 - In transit bit is valid
// Bit 3 - Hallucinated bit is valid
// Bit 4 - Invincible bit is valid
// Bit 5-15 - Unknown/unused
// u16: Which elements of the unit data are valid, which properties can be changed by the map maker.
// Bit 0 - Owner player is valid (unit is not neutral)
// Bit 1 - HP is valid
// Bit 2 - Shields is valid
// Bit 3 - Energy is valid
// Bit 4 - Resource amount is valid (unit is a resource)
// Bit 5 - Amount in hanger is valid
// Bit 6 - Unknown/unused
// u8: Player number that owns unit. Will always be NULL in this section (0)
// u8: Hit point % (1-100)
// u8: Shield point % (1-100)
// u8: Energy point % (1-100)
// u32: Resource amount (for resources only)
// u16: # of units in hangar
// u16: Flags
// Bit 0 - Unit is cloaked
// Bit 1 - Unit is burrowed
// Bit 2 - Building is in transit
// Bit 3 - Unit is hallucinated
// Bit 4 - Unit is invincible
// Bit 5-15 - Unknown/unused
// u32: Unknown/unused. Padding?

#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ChkUprpIndividual {
    pub flag_of_special_properties: u16,
    pub which_elements_of_unit_data_are_valid: u16,
    pub owner: u8,
    pub hit_points_percent: u8,
    pub shield_points_percent: u8,
    pub energy_points_percent: u8,
    pub resource_amount: u32,
    pub number_of_units_in_hangar: u16,
    pub flags: u16,
    pub padding: u32,
}

#[derive(Debug, Serialize)]
pub struct ChkUprp<'a> {
    pub cuwp_slots: &'a [ChkUprpIndividual],
}

pub(crate) fn parse_uprp<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkUprp<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkUprp {
        cuwp_slots: slicer.extract_rest_as_slice_lax()?,
    })
}
