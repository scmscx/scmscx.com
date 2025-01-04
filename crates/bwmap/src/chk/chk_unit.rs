use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be a multiple of 36 bytes.

// This section contains all the pre-placed units on the map and their properties. Each unit on the map gets the following structure:

// u32: The unit's class instance (sort of a "serial number")
// U16: X coordinate of unit
// U16: Y coordinate of unit
// u16: Unit ID
// u16: Type of relation to another building (i.e. add-on, nydus link)
// Bit 9 - Nydus Link
// Bit 10 - Addon Link
// u16: Flags of special properties which can be applied to the unit and are valid:
// Bit 0 - Cloak is valid
// Bit 1 - Burrow is valid
// Bit 2 - In transit is valid
// Bit 3 - Hallucinated is valid
// Bit 4 - Invincible is valid
// Bit 5-15 - Unused
// u16: Out of the elements of the unit data, the properties which can be changed by the map maker:
// Bit 0 - Owner player is valid (the unit is not a critter, start location, etc.; not a neutral unit)
// Bit 1 - HP is valid
// Bit 2 - Shields is valid
// Bit 3 - Energy is valid (unit is a wraith, etc.)
// Bit 4 - Resource amount is valid (unit is a mineral patch, vespene geyser, etc.)
// Bit 5 - Amount in hangar is valid (unit is a reaver, carrier, etc.)
// Bit 6-15 - Unused
// u8: Player number of owner (0-based)
// u8: Hit points % (1-100)
// u8: Shield points % (1-100)
// u8: Energy points % (1-100)
// u32: Resource amount
// u16: Number of units in hangar
// u16: Unit state flags
// Bit 0 - Unit is cloaked
// Bit 1 - Unit is burrowed
// Bit 2 - Building is in transit
// Bit 3 - Unit is hallucinated
// Bit 4 - Unit is invincible
// Bit 5-15 - Unused
// u32: Unused
// u32: Class instance of the unit to which this unit is related to (i.e. via an add-on, nydus link, etc.). It is "0" if the unit is not linked to any other unit.
// Notes about UNIT:

// The X/Y coordinates are the center of the sprite of the unit (in pixels).
// Please edit if you could confirm the bit values to be correct/if you know more bit values.
// Default values will apply if bit values are unchecked. Defaults: 100% HP, 100% SP, 100% EP, 0 resources, 0 hangar count.
// This section can be split. Additional UNIT sections will add more units.

#[derive(Debug, Serialize, Eq, PartialEq, Copy, Clone)]
#[repr(C, packed)]
pub struct ChkUnitIndividual {
    pub class_instance: u32,
    pub x: u16,
    pub y: u16,
    pub unit_id: u16,
    pub type_of_relation_to_other_building: u16,
    pub properties_that_can_be_applied: u16,
    pub properties_that_can_be_changed: u16,
    pub owner: u8,
    pub hit_points_percent: u8,
    pub shield_points_percent: u8,
    pub energy_points_percent: u8,
    pub resource_amount: u32,
    pub number_of_units_in_hangar: u16,
    pub unit_state_flags: u16,
    pub unused: u32,
    pub class_instance_related_to: u32,
}

#[derive(Debug, Serialize)]
pub struct ChkUnit {
    pub units: Vec<ChkUnitIndividual>,
}

pub(crate) fn parse_unit(chunks: &[RiffChunk]) -> Result<ChkUnit, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut units: Vec<ChkUnitIndividual> = Vec::new();

    for chunk in chunks {
        let mut slicer = CursorSlicer::new(chunk.data);
        units.extend_from_slice(slicer.extract_rest_as_slice_lax()?);
    }

    Ok(ChkUnit { units })
}
