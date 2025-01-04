// 1.1 "TYPE" - Map Type
// 1.2 "VER " - Format Version
// 1.3 "IVER" - Map Version
// 1.4 "IVE2" - Map Version
// 1.5 "VCOD" - Verification Code
// 1.6 "IOWN" - StarEdit Player Types
// 1.7 "OWNR" - StarCraft Player Types
// 1.8 "ERA " - Tileset
// 1.9 "DIM " - Map Dimensions
// 1.10 "SIDE" - Player Races
// 1.11 "MTXM" - StarCraft Terrain
// 1.12 "PUNI" - Player Unit Restrictions
// 1.13 "UPGR" - Upgrade Restrictions
// 1.14 "PTEC" - Tech Restrictions
// 1.15 "UNIT" - Placed Units
// 1.16 "ISOM" - Isometric Terrain
// 1.17 "TILE" - StarEdit Terrain
// 1.18 "DD2 " - StarEdit Sprites (Doodads)
// 1.19 "THG2" - StarCraft Sprites
// 1.20 "MASK" - Fog of War Layer
// 1.21 "STR " - String Data
// 1.22 "UPRP" - CUWP Slots
// 1.23 "UPUS" - CUWP Slots Used
// 1.24 "MRGN" - Locations
// 1.25 "TRIG" - Triggers
// 1.26 "MBRF" - Mission Briefings
// 1.27 "SPRP" - Scenario Properties
// 1.28 "FORC" - Force Settings
// 1.29 "WAV " - WAV String Indexes
// 1.30 "UNIS" - Unit Settings
// 1.31 "UPGS" - Upgrade Settings
// 1.32 "TECS" - Tech Settings
// 1.33 "SWNM" - Switch Names
// 1.34 "COLR" - Player Colors
// 1.35 "PUPx" - BW Upgrade Restrictions
// 1.36 "PTEx" - BW Tech Restrictions
// 1.37 "UNIx" - BW Unit Settings
// 1.38 "UPGx" - BW Upgrade Settings
// 1.39 "TECx" - BW Tech Settings

use serde::Serialize;

use crate::{
    chk::{
        chk_mbrf::{ChkMbrfAction, ChkMbrfCondition},
        chk_trig::{ChkTrigAction, ChkTrigCondition},
    },
    util::reinterpret_as_slice,
    ParsedChk,
};
use tracing::instrument;

static AI_SCRIPT_MAP: phf::Map<&'static [u8], &'static str> = phf::phf_map! {
    b"TMCu" => "Terran Custom Level",
    b"ZMCu" => "Zerg Custom Level",
    b"PMCu" => "Protoss Custom Level",
    b"TMCx" => "Terran Expansion Custom Level",
    b"ZMCx" => "Zerg Expansion Custom Level",
    b"PMCx" => "Protoss Expansion Custom Level",
    b"TLOf" => "Terran Campaign Easy",
    b"TMED" => "Terran Campaign Medium",
    b"THIf" => "Terran Campaign Difficult",
    b"TSUP" => "Terran Campaign Insane",
    b"TARE" => "Terran Campaign Area Town",
    b"ZLOf" => "Zerg Campaign Easy",
    b"ZMED" => "Zerg Campaign Medium",
    b"ZHIf" => "Zerg Campaign Difficult",
    b"ZSUP" => "Zerg Campaign Insane",
    b"ZARE" => "Zerg Campaign Area Town",
    b"PLOf" => "Protoss Campaign Easy",
    b"PMED" => "Protoss Campaign Medium",
    b"PHIf" => "Protoss Campaign Difficult",
    b"PSUP" => "Protoss Campaign Insane",
    b"PARE" => "Protoss Campaign Area Town",
    b"TLOx" => "Expansion Terran Campaign Easy",
    b"TMEx" => "Expansion Terran Campaign Medium",
    b"THIx" => "Expansion Terran Campaign Difficult",
    b"TSUx" => "Expansion Terran Campaign Insane",
    b"TARx" => "Expansion Terran Campaign Area Town",
    b"ZLOx" => "Expansion Zerg Campaign Easy",
    b"ZMEx" => "Expansion Zerg Campaign Medium",
    b"ZHIx" => "Expansion Zerg Campaign Difficult",
    b"ZSUx" => "Expansion Zerg Campaign Insane",
    b"ZARx" => "Expansion Zerg Campaign Area Town",
    b"PLOx" => "Expansion Protoss Campaign Easy",
    b"PMEx" => "Expansion Protoss Campaign Medium",
    b"PHIx" => "Expansion Protoss Campaign Difficult",
    b"PSUx" => "Expansion Protoss Campaign Insane",
    b"PARx" => "Expansion Protoss Campaign Area Town",
    b"Suic" => "Send All Units on Strategic Suicide Missions",
    b"SuiR" => "Send All Units on Random Suicide Missions",
    b"Rscu" => "Switch Computer Player to Rescue Passive",
    b"+Vi0" => "Turn ON Shared Vision for Player 1",
    b"+Vi1" => "Turn ON Shared Vision for Player 2",
    b"+Vi2" => "Turn ON Shared Vision for Player 3",
    b"+Vi3" => "Turn ON Shared Vision for Player 4",
    b"+Vi4" => "Turn ON Shared Vision for Player 5",
    b"+Vi5" => "Turn ON Shared Vision for Player 6",
    b"+Vi6" => "Turn ON Shared Vision for Player 7",
    b"+Vi7" => "Turn ON Shared Vision for Player 8",
    b"-Vi0" => "Turn OFF Shared Vision for Player 1",
    b"-Vi1" => "Turn OFF Shared Vision for Player 2",
    b"-Vi2" => "Turn OFF Shared Vision for Player 3",
    b"-Vi3" => "Turn OFF Shared Vision for Player 4",
    b"-Vi4" => "Turn OFF Shared Vision for Player 5",
    b"-Vi5" => "Turn OFF Shared Vision for Player 6",
    b"-Vi6" => "Turn OFF Shared Vision for Player 7",
    b"-Vi7" => "Turn OFF Shared Vision for Player 8",
    b"MvTe" => "Move Dark Templars to Region",
    b"ClrC" => "Clear Previous Combat Data",
    b"Enmy" => "Set Player to Enemy",
    b"y   " => "Set Player to Ally",
    b"VluA" => "Value This Area Higher",
    b"EnBk" => "Enter Closest Bunker",
    b"StTg" => "Set Generic Command Target",
    b"StPt" => "Make These Units Patrol",
    b"EnTr" => "Enter Transport",
    b"ExTr" => "Exit Transport",
    b"NuHe" => "AI Nuke Here",
    b"HaHe" => "AI Harass Here",
    b"JYDg" => "Set Unit Order To:Junk Yard Dog",
    b"DWHe" => "Disruption Web Here",
    b"ReHe" => "Recall Here",
    b"Ter3" => "Terran 3 - Zerg Town",
    b"Ter5" => "Terran 5 - Terran Main Town",
    b"Te5H" => "Terran 5 - Terran Harvest Town",
    b"Ter6" => "Terran 6 - Air Attack Zerg",
    b"Te6b" => "Terran 6 - Ground Attack Zerg",
    b"Te6c" => "Terran 6 - Zerg Support Town",
    b"Ter7" => "Terran 7 - Bottom Zerg Town",
    b"Te7s" => "Terran 7 - Right Zerg Town",
    b"Te7m" => "Terran 7 - Middle Zerg Town",
    b"Ter8" => "Terran 8 - Confederate Town",
    b"Tr9L" => "Terran 9 - Light Attack",
    b"Tr9H" => "Terran 9 - Heavy Attack",
    b"Te10" => "Terran 10 - Confederate Towns",
    b"T11z" => "Terran 11 - Zerg Town",
    b"T11a" => "Terran 11 - Lower Protoss Town",
    b"T11b" => "Terran 11 - Upper Protoss Town",
    b"T12N" => "Terran 12 - Nuke Town",
    b"T12P" => "Terran 12 - Phoenix Town",
    b"T12T" => "Terran 12 - Tank Town",
    b"TED1" => "Terran 1 - Electronic Distribution",
    b"TED2" => "Terran 2 - Electronic Distribution",
    b"TED3" => "Terran 3 - Electronic Distribution",
    b"TSW1" => "Terran 1 - Shareware",
    b"TSW2" => "Terran 2 - Shareware",
    b"TSW3" => "Terran 3 - Shareware",
    b"TSW4" => "Terran 4 - Shareware",
    b"TSW5" => "Terran 5 - Shareware",
    b"Zer1" => "Zerg 1 - Terran Town",
    b"Zer2" => "Zerg 2 - Protoss Town",
    b"Zer3" => "Zerg 3 - Terran Town",
    b"Zer4" => "Zerg 4 - Right Terran Town",
    b"Ze4S" => "Zerg 4 - Lower Terran Town",
    b"Zer6" => "Zerg 6 - Protoss Town",
    b"Zr7a" => "Zerg 7 - Air Town",
    b"Zr7g" => "Zerg 7 - Ground Town",
    b"Zr7s" => "Zerg 7 - Support Town",
    b"Zer8" => "Zerg 8 - Scout Town",
    b"Ze8T" => "Zerg 8 - Templar Town",
    b"Zer9" => "Zerg 9 - Teal Protoss",
    b"Z9ly" => "Zerg 9 - Left Yellow Protoss",
    b"Z9ry" => "Zerg 9 - Right Yellow Protoss",
    b"Z9lo" => "Zerg 9 - Left Orange Protoss",
    b"Z9ro" => "Zerg 9 - Right Orange Protoss",
    b"Z10a" => "Zerg 10 - Left Teal (Attack",
    b"Z10b" => "Zerg 10 - Right Teal (Support",
    b"Z10c" => "Zerg 10 - Left Yellow (Support",
    b"Z10d" => "Zerg 10 - Right Yellow (Attack",
    b"Z10e" => "Zerg 10 - Red Protoss",
    b"Pro1" => "Protoss 1 - Zerg Town",
    b"Pro2" => "Protoss 2 - Zerg Town",
    b"Pr3R" => "Protoss 3 - Air Zerg Town",
    b"Pr3G" => "Protoss 3 - Ground Zerg Town",
    b"Pro4" => "Protoss 4 - Zerg Town",
    b"Pr5I" => "Protoss 5 - Zerg Town Island",
    b"Pr5B" => "Protoss 5 - Zerg Town Base",
    b"Pro7" => "Protoss 7 - Left Protoss Town",
    b"Pr7B" => "Protoss 7 - Right Protoss Town",
    b"Pr7S" => "Protoss 7 - Shrine Protoss",
    b"Pro8" => "Protoss 8 - Left Protoss Town",
    b"Pr8B" => "Protoss 8 - Right Protoss Town",
    b"Pr8D" => "Protoss 8 - Protoss Defenders",
    b"Pro9" => "Protoss 9 - Ground Zerg",
    b"Pr9W" => "Protoss 9 - Air Zerg",
    b"Pr9Y" => "Protoss 9 - Spell Zerg",
    b"Pr10" => "Protoss 10 - Mini-Towns",
    b"P10C" => "Protoss 10 - Mini-Town Master",
    b"P10o" => "Protoss 10 - Overmind Defenders",
    b"PB1A" => "Brood Wars Protoss 1 - Town A",
    b"PB1B" => "Brood Wars Protoss 1 - Town B",
    b"PB1C" => "Brood Wars Protoss 1 - Town C",
    b"PB1D" => "Brood Wars Protoss 1 - Town D",
    b"PB1E" => "Brood Wars Protoss 1 - Town E",
    b"PB1F" => "Brood Wars Protoss 1 - Town F",
    b"PB2A" => "Brood Wars Protoss 2 - Town A",
    b"PB2B" => "Brood Wars Protoss 2 - Town B",
    b"PB2C" => "Brood Wars Protoss 2 - Town C",
    b"PB2D" => "Brood Wars Protoss 2 - Town D",
    b"PB2E" => "Brood Wars Protoss 2 - Town E",
    b"PB2F" => "Brood Wars Protoss 2 - Town F",
    b"PB3A" => "Brood Wars Protoss 3 - Town A",
    b"PB3B" => "Brood Wars Protoss 3 - Town B",
    b"PB3C" => "Brood Wars Protoss 3 - Town C",
    b"PB3D" => "Brood Wars Protoss 3 - Town D",
    b"PB3E" => "Brood Wars Protoss 3 - Town E",
    b"PB3F" => "Brood Wars Protoss 3 - Town F",
    b"PB4A" => "Brood Wars Protoss 4 - Town A",
    b"PB4B" => "Brood Wars Protoss 4 - Town B",
    b"PB4C" => "Brood Wars Protoss 4 - Town C",
    b"PB4D" => "Brood Wars Protoss 4 - Town D",
    b"PB4E" => "Brood Wars Protoss 4 - Town E",
    b"PB4F" => "Brood Wars Protoss 4 - Town F",
    b"PB5A" => "Brood Wars Protoss 5 - Town A",
    b"PB5B" => "Brood Wars Protoss 5 - Town B",
    b"PB5C" => "Brood Wars Protoss 5 - Town C",
    b"PB5D" => "Brood Wars Protoss 5 - Town D",
    b"PB5E" => "Brood Wars Protoss 5 - Town E",
    b"PB5F" => "Brood Wars Protoss 5 - Town F",
    b"PB6A" => "Brood Wars Protoss 6 - Town A",
    b"PB6B" => "Brood Wars Protoss 6 - Town B",
    b"PB6C" => "Brood Wars Protoss 6 - Town C",
    b"PB6D" => "Brood Wars Protoss 6 - Town D",
    b"PB6E" => "Brood Wars Protoss 6 - Town E",
    b"PB6F" => "Brood Wars Protoss 6 - Town F",
    b"PB7A" => "Brood Wars Protoss 7 - Town A",
    b"PB7B" => "Brood Wars Protoss 7 - Town B",
    b"PB7C" => "Brood Wars Protoss 7 - Town C",
    b"PB7D" => "Brood Wars Protoss 7 - Town D",
    b"PB7E" => "Brood Wars Protoss 7 - Town E",
    b"PB7F" => "Brood Wars Protoss 7 - Town F",
    b"PB8A" => "Brood Wars Protoss 8 - Town A",
    b"PB8B" => "Brood Wars Protoss 8 - Town B",
    b"PB8C" => "Brood Wars Protoss 8 - Town C",
    b"PB8D" => "Brood Wars Protoss 8 - Town D",
    b"PB8E" => "Brood Wars Protoss 8 - Town E",
    b"PB8F" => "Brood Wars Protoss 8 - Town F",
    b"TB1A" => "Brood Wars Terran 1 - Town A",
    b"TB1B" => "Brood Wars Terran 1 - Town B",
    b"TB1C" => "Brood Wars Terran 1 - Town C",
    b"TB1D" => "Brood Wars Terran 1 - Town D",
    b"TB1E" => "Brood Wars Terran 1 - Town E",
    b"TB1F" => "Brood Wars Terran 1 - Town F",
    b"TB2A" => "Brood Wars Terran 2 - Town A",
    b"TB2B" => "Brood Wars Terran 2 - Town B",
    b"TB2C" => "Brood Wars Terran 2 - Town C",
    b"TB2D" => "Brood Wars Terran 2 - Town D",
    b"TB2E" => "Brood Wars Terran 2 - Town E",
    b"TB2F" => "Brood Wars Terran 2 - Town F",
    b"TB3A" => "Brood Wars Terran 3 - Town A",
    b"TB3B" => "Brood Wars Terran 3 - Town B",
    b"TB3C" => "Brood Wars Terran 3 - Town C",
    b"TB3D" => "Brood Wars Terran 3 - Town D",
    b"TB3E" => "Brood Wars Terran 3 - Town E",
    b"TB3F" => "Brood Wars Terran 3 - Town F",
    b"TB4A" => "Brood Wars Terran 4 - Town A",
    b"TB4B" => "Brood Wars Terran 4 - Town B",
    b"TB4C" => "Brood Wars Terran 4 - Town C",
    b"TB4D" => "Brood Wars Terran 4 - Town D",
    b"TB4E" => "Brood Wars Terran 4 - Town E",
    b"TB4F" => "Brood Wars Terran 4 - Town F",
    b"TB5A" => "Brood Wars Terran 5 - Town A",
    b"TB5B" => "Brood Wars Terran 5 - Town B",
    b"TB5C" => "Brood Wars Terran 5 - Town C",
    b"TB5D" => "Brood Wars Terran 5 - Town D",
    b"TB5E" => "Brood Wars Terran 5 - Town E",
    b"TB5F" => "Brood Wars Terran 5 - Town F",
    b"TB6A" => "Brood Wars Terran 6 - Town A",
    b"TB6B" => "Brood Wars Terran 6 - Town B",
    b"TB6C" => "Brood Wars Terran 6 - Town C",
    b"TB6D" => "Brood Wars Terran 6 - Town D",
    b"TB6E" => "Brood Wars Terran 6 - Town E",
    b"TB6F" => "Brood Wars Terran 6 - Town F",
    b"TB7A" => "Brood Wars Terran 7 - Town A",
    b"TB7B" => "Brood Wars Terran 7 - Town B",
    b"TB7C" => "Brood Wars Terran 7 - Town C",
    b"TB7D" => "Brood Wars Terran 7 - Town D",
    b"TB7E" => "Brood Wars Terran 7 - Town E",
    b"TB7F" => "Brood Wars Terran 7 - Town F",
    b"TB8A" => "Brood Wars Terran 8 - Town A",
    b"TB8B" => "Brood Wars Terran 8 - Town B",
    b"TB8C" => "Brood Wars Terran 8 - Town C",
    b"TB8D" => "Brood Wars Terran 8 - Town D",
    b"TB8E" => "Brood Wars Terran 8 - Town E",
    b"TB8F" => "Brood Wars Terran 8 - Town F",
    b"ZB1A" => "Brood Wars Zerg 1 - Town A",
    b"ZB1B" => "Brood Wars Zerg 1 - Town B",
    b"ZB1C" => "Brood Wars Zerg 1 - Town C",
    b"ZB1D" => "Brood Wars Zerg 1 - Town D",
    b"ZB1E" => "Brood Wars Zerg 1 - Town E",
    b"ZB1F" => "Brood Wars Zerg 1 - Town F",
    b"ZB2A" => "Brood Wars Zerg 2 - Town A",
    b"ZB2B" => "Brood Wars Zerg 2 - Town B",
    b"ZB2C" => "Brood Wars Zerg 2 - Town C",
    b"ZB2D" => "Brood Wars Zerg 2 - Town D",
    b"ZB2E" => "Brood Wars Zerg 2 - Town E",
    b"ZB2F" => "Brood Wars Zerg 2 - Town F",
    b"ZB3A" => "Brood Wars Zerg 3 - Town A",
    b"ZB3B" => "Brood Wars Zerg 3 - Town B",
    b"ZB3C" => "Brood Wars Zerg 3 - Town C",
    b"ZB3D" => "Brood Wars Zerg 3 - Town D",
    b"ZB3E" => "Brood Wars Zerg 3 - Town E",
    b"ZB3F" => "Brood Wars Zerg 3 - Town F",
    b"ZB4A" => "Brood Wars Zerg 4 - Town A",
    b"ZB4B" => "Brood Wars Zerg 4 - Town B",
    b"ZB4C" => "Brood Wars Zerg 4 - Town C",
    b"ZB4D" => "Brood Wars Zerg 4 - Town D",
    b"ZB4E" => "Brood Wars Zerg 4 - Town E",
    b"ZB4F" => "Brood Wars Zerg 4 - Town F",
    b"ZB5A" => "Brood Wars Zerg 5 - Town A",
    b"ZB5B" => "Brood Wars Zerg 5 - Town B",
    b"ZB5C" => "Brood Wars Zerg 5 - Town C",
    b"ZB5D" => "Brood Wars Zerg 5 - Town D",
    b"ZB5E" => "Brood Wars Zerg 5 - Town E",
    b"ZB5F" => "Brood Wars Zerg 5 - Town F",
    b"ZB6A" => "Brood Wars Zerg 6 - Town A",
    b"ZB6B" => "Brood Wars Zerg 6 - Town B",
    b"ZB6C" => "Brood Wars Zerg 6 - Town C",
    b"ZB6D" => "Brood Wars Zerg 6 - Town D",
    b"ZB6E" => "Brood Wars Zerg 6 - Town E",
    b"ZB6F" => "Brood Wars Zerg 6 - Town F",
    b"ZB7A" => "Brood Wars Zerg 7 - Town A",
    b"ZB7B" => "Brood Wars Zerg 7 - Town B",
    b"ZB7C" => "Brood Wars Zerg 7 - Town C",
    b"ZB7D" => "Brood Wars Zerg 7 - Town D",
    b"ZB7E" => "Brood Wars Zerg 7 - Town E",
    b"ZB7F" => "Brood Wars Zerg 7 - Town F",
    b"ZB8A" => "Brood Wars Zerg 8 - Town A",
    b"ZB8B" => "Brood Wars Zerg 8 - Town B",
    b"ZB8C" => "Brood Wars Zerg 8 - Town C",
    b"ZB8D" => "Brood Wars Zerg 8 - Town D",
    b"ZB8E" => "Brood Wars Zerg 8 - Town E",
    b"ZB8F" => "Brood Wars Zerg 8 - Town F",
    b"ZB9A" => "Brood Wars Zerg 9 - Town A",
    b"ZB9B" => "Brood Wars Zerg 9 - Town B",
    b"ZB9C" => "Brood Wars Zerg 9 - Town C",
    b"ZB9D" => "Brood Wars Zerg 9 - Town D",
    b"ZB9E" => "Brood Wars Zerg 9 - Town E",
    b"ZB9F" => "Brood Wars Zerg 9 - Town F",
    b"ZB0A" => "Brood Wars Zerg 10 - Town A",
    b"ZB0B" => "Brood Wars Zerg 10 - Town B",
    b"ZB0C" => "Brood Wars Zerg 10 - Town C",
    b"ZB0D" => "Brood Wars Zerg 10 - Town D",
    b"ZB0E" => "Brood Wars Zerg 10 - Town E",
    b"ZB0F" => "Brood Wars Zerg 10 - Town F",
};

#[derive(Debug, Serialize)]
pub enum ResourceType {
    Unknown(i64),
    Ore,
    Gas,
    OreAndGas,
}

#[instrument(level = "trace")]
fn parse_resource_type(resource_type: u16) -> ResourceType {
    match resource_type {
        0 => ResourceType::Ore,
        1 => ResourceType::Gas,
        10 => ResourceType::OreAndGas,
        _ => ResourceType::Unknown(resource_type as i64),
    }
}

#[derive(Debug, Serialize)]
pub enum NumericComparison {
    Unknown(i64),
    AtLeast,
    AtMost,
    Exactly,
}

#[instrument(level = "trace")]
fn parse_numeric_comparison(numeric_comparison: u8) -> NumericComparison {
    match numeric_comparison {
        0 => NumericComparison::AtLeast,
        1 => NumericComparison::AtMost,
        10 => NumericComparison::Exactly,
        _ => NumericComparison::Unknown(numeric_comparison as i64),
    }
}

#[derive(Debug, Serialize)]
pub enum NumberModifier {
    Unknown(i64),
    SetTo,
    Add,
    Subtract,
}

#[instrument(level = "trace")]
fn parse_number_modifier(modifier: u8) -> NumberModifier {
    match modifier {
        7 => NumberModifier::SetTo,
        8 => NumberModifier::Add,
        9 => NumberModifier::Subtract,
        _ => NumberModifier::Unknown(modifier as i64),
    }
}

#[derive(Debug, Serialize)]
pub enum ScoreType {
    Unknown(i64),
    Total,
    Units,
    Buildings,
    UnitsAndBuildings,
    Kills,
    Razings,
    KillsAndRazings,
    Custom,
}

fn parse_score_type(score_type: u16) -> ScoreType {
    match score_type {
        0 => ScoreType::Total,
        1 => ScoreType::Units,
        2 => ScoreType::Buildings,
        3 => ScoreType::UnitsAndBuildings,
        4 => ScoreType::Kills,
        5 => ScoreType::Razings,
        6 => ScoreType::KillsAndRazings,
        7 => ScoreType::Custom,
        _ => ScoreType::Unknown(score_type as i64),
    }
}

#[derive(Debug, Serialize)]
pub enum SwitchState {
    Unknown(i64),
    Set,
    Cleared,
}

#[instrument(level = "trace")]
fn parse_switch_state(switch_state: u8) -> SwitchState {
    match switch_state {
        2 => SwitchState::Set,
        3 => SwitchState::Cleared,
        _ => SwitchState::Unknown(switch_state as i64),
    }
}

#[derive(Debug, Serialize)]
pub enum Order {
    Unknown(i64),
    Move,
    Patrol,
    Attack,
}

#[instrument(level = "trace")]
fn parse_order(order: u8) -> Order {
    match order {
        0 => Order::Move,
        1 => Order::Patrol,
        2 => Order::Attack,
        _ => Order::Unknown(order as i64),
    }
}

#[derive(Debug, Serialize)]
pub enum ActionState {
    Unknown(i64),
    EnabledOrSet,
    DisabledOrClear,
    ToggleOrToggle,
    RandomizeSwitch,
}

#[instrument(level = "trace")]
fn parse_action_state(action_state: u8) -> ActionState {
    match action_state {
        4 => ActionState::EnabledOrSet,
        5 => ActionState::DisabledOrClear,
        6 => ActionState::ToggleOrToggle,
        11 => ActionState::RandomizeSwitch,
        _ => ActionState::Unknown(action_state as i64),
    }
}

#[instrument(level = "trace")]
fn parse_ai_script(aiscript_id: u32) -> &'static str {
    if let Ok(x) = reinterpret_as_slice(&aiscript_id) {
        AI_SCRIPT_MAP
            .get(x)
            .cloned()
            .unwrap_or("Failed to find ai script")
    } else {
        ""
    }
}

//AI_SCRIPT_MAP

#[derive(Debug, Serialize)]
pub enum AllianceStatus {
    Unknown(i64),
    Enemy,
    Allied,
    AlliedVictory,
}

fn parse_alliance_status(alliance_status: u16) -> AllianceStatus {
    match alliance_status {
        0 => AllianceStatus::Enemy,
        1 => AllianceStatus::Allied,
        2 => AllianceStatus::AlliedVictory,
        _ => AllianceStatus::Unknown(alliance_status as i64),
    }
}

#[derive(Debug, Serialize)]
pub enum UnitType {
    Unknown(i64),
    TerranMarine,
    TerranGhost,
    TerranVulture,
    TerranGoliath,
    GoliathTurret,
    TerranSiegeTankTankMode,
    TankTurretTankMode,
    TerranSCV,
    TerranWraith,
    TerranScienceVessel,
    GuiMontangFirebat,
    TerranDropship,
    TerranBattlecruiser,
    VultureSpiderMine,
    NuclearMissile,
    TerranCivilian,
    SarahKerriganGhost,
    AlanSchezarGoliath,
    AlanSchezarTurret,
    JimRaynorVulture,
    JimRaynorMarine,
    TomKazanskyWraith,
    MagellanScienceVessel,
    EdmundDukeSiegeTank,
    EdmundDukeTurretTankMode,
    EdmundDukeSiegeMode,
    EdmundDukeTurretSiegeMode,
    ArcturusMengskBattlecruiser,
    HyperionBattlecruiser,
    NoradIIBattlecruiser,
    TerranSiegeTankSiegeMode,
    TankTurretSiegeMode,
    Firebat,
    ScannerSweep,
    TerranMedic,
    ZergLarva,
    ZergEgg,
    ZergZergling,
    ZergHydralisk,
    ZergUltralisk,
    ZergBroodling,
    ZergDrone,
    ZergOverlord,
    ZergMutalisk,
    ZergGuardian,
    ZergQueen,
    ZergDefiler,
    ZergScourge,
    TorrarsqueUltralisk,
    MatriarchQueen,
    InfestedTerran,
    InfestedKerrigan,
    UncleanOneDefiler,
    HunterKillerHydralisk,
    DevouringOneZergling,
    KukulzaMutalisk,
    KukulzaGuardian,
    YggdrasillOverlord,
    TerranValkyrie,
    MutaliskOrGuardianCocoon,
    ProtossCorsair,
    ProtossDarkTemplar,
    ZergDevourer,
    ProtossDarkArchon,
    ProtossProbe,
    ProtossZealot,
    ProtossDragoon,
    ProtossHighTemplar,
    ProtossArchon,
    ProtossShuttle,
    ProtossScout,
    ProtossArbiter,
    ProtossCarrier,
    ProtossInterceptor,
    DarkTemplarHero,
    ZeratulDarkTemplar,
    TassadarAndZeratulArchon,
    FenixZealot,
    FenixDragoon,
    TassadarTemplar,
    MojoScout,
    WarbringerReaver,
    GantrithorCarrier,
    ProtossReaver,
    ProtossObserver,
    ProtossScarab,
    DanimothArbiter,
    AldarisTemplar,
    ArtanisScout,
    RhynadonBadlandsCritter,
    BengalaasJungleCritter,
    UnusedWasCargoShip,
    UnusedWasMercenaryGunship,
    ScantidDesertCritter,
    KakaruTwilightCritter,
    RagnasaurAshworldCritter,
    UrsadonIceWorldCritter,
    LurkerEgg,
    Raszagal,
    SamirDuranGhost,
    AlexeiStukovGhost,
    MapRevealer,
    GerardDuGalle,
    ZergLurker,
    InfestedDuran,
    DisruptionWeb,
    TerranCommandCenter,
    TerranComsatStation,
    TerranNuclearSilo,
    TerranSupplyDepot,
    TerranRefinery,
    TerranBarracks,
    TerranAcademy,
    TerranFactory,
    TerranStarport,
    TerranControlTower,
    TerranScienceFacility,
    TerranCovertOps,
    TerranPhysicsLab,
    UnusedWasStarbase,
    TerranMachineShop,
    UnusedWasRepairBay,
    TerranEngineeringBay,
    TerranArmory,
    TerranMissileTurret,
    TerranBunker,
    NoradII,
    IonCannon,
    UrajCrystal,
    KhalisCrystal,
    InfestedCommandCenter,
    ZergHatchery,
    ZergLair,
    ZergHive,
    ZergNydusCanal,
    ZergHydraliskDen,
    ZergDefilerMound,
    ZergGreaterSpire,
    ZergQueensNest,
    ZergEvolutionChamber,
    ZergUltraliskCavern,
    ZergSpire,
    ZergSpawningPool,
    ZergCreepColony,
    ZergSporeColony,
    UnusedZergBuilding,
    ZergSunkenColony,
    ZergOvermindWithShell,
    ZergOvermind,
    ZergExtractor,
    MatureChrysalis,
    ZergCerebrate,
    ZergCerebrateDaggoth,
    UnusedZergBuilding5,
    ProtossNexus,
    ProtossRoboticsFacility,
    ProtossPylon,
    ProtossAssimilator,
    UnusedProtossBuilding1,
    ProtossObservatory,
    ProtossGateway,
    UnusedProtossBuilding2,
    ProtossPhotonCannon,
    ProtossCitadelofAdun,
    ProtossCyberneticsCore,
    ProtossTemplarArchives,
    ProtossForge,
    ProtossStargate,
    StasisCellPrison,
    ProtossFleetBeacon,
    ProtossArbiterTribunal,
    ProtossRoboticsSupportBay,
    ProtossShieldBattery,
    KhaydarinCrystalFormation,
    ProtossTemple,
    XelNagaTemple,
    MineralFieldType1,
    MineralFieldType2,
    MineralFieldType3,
    Cave,
    Cavein,
    Cantina,
    MiningPlatform,
    IndependantCommandCenter,
    IndependantStarport,
    IndependantJumpGate,
    Ruins,
    KyadarinCrystalFormation,
    VespeneGeyser,
    WarpGate,
    PSIDisruptor,
    ZergMarker,
    TerranMarker,
    ProtossMarker,
    ZergBeacon,
    TerranBeacon,
    ProtossBeacon,
    ZergFlagBeacon,
    TerranFlagBeacon,
    ProtossFlagBeacon,
    PowerGenerator,
    OvermindCocoon,
    DarkSwarm,
    FloorMissileTrap,
    FloorHatch,
    LeftUpperLevelDoor,
    RightUpperLevelDoor,
    LeftPitDoor,
    RightPitDoor,
    FloorGunTrap,
    LeftWallMissileTrap,
    LeftWallFlameTrap,
    RightWallMissileTrap,
    RightWallFlameTrap,
    StartLocation,
    Flag,
    YoungChrysalis,
    PsiEmitter,
    DataDisc,
    KhaydarinCrystal,
    MineralClusterType1,
    MineralClusterType2,
    ProtossVespeneGasOrbType1,
    ProtossVespeneGasOrbType2,
    ZergVespeneGasSacType1,
    ZergVespeneGasSacType2,
    TerranVespeneGasTankType1,
    TerranVespeneGasTankType2,
    None,
    AnyUnit,
    Men,
    Buildings,
    Factories,
}

fn parse_unit_type(unit_type: u16) -> UnitType {
    match unit_type {
        0 => UnitType::TerranMarine,
        1 => UnitType::TerranGhost,
        2 => UnitType::TerranVulture,
        3 => UnitType::TerranGoliath,
        4 => UnitType::GoliathTurret,
        5 => UnitType::TerranSiegeTankTankMode,
        6 => UnitType::TankTurretTankMode,
        7 => UnitType::TerranSCV,
        8 => UnitType::TerranWraith,
        9 => UnitType::TerranScienceVessel,
        10 => UnitType::GuiMontangFirebat,
        11 => UnitType::TerranDropship,
        12 => UnitType::TerranBattlecruiser,
        13 => UnitType::VultureSpiderMine,
        14 => UnitType::NuclearMissile,
        15 => UnitType::TerranCivilian,
        16 => UnitType::SarahKerriganGhost,
        17 => UnitType::AlanSchezarGoliath,
        18 => UnitType::AlanSchezarTurret,
        19 => UnitType::JimRaynorVulture,
        20 => UnitType::JimRaynorMarine,
        21 => UnitType::TomKazanskyWraith,
        22 => UnitType::MagellanScienceVessel,
        23 => UnitType::EdmundDukeSiegeTank,
        24 => UnitType::EdmundDukeTurretTankMode,
        25 => UnitType::EdmundDukeSiegeMode,
        26 => UnitType::EdmundDukeTurretSiegeMode,
        27 => UnitType::ArcturusMengskBattlecruiser,
        28 => UnitType::HyperionBattlecruiser,
        29 => UnitType::NoradIIBattlecruiser,
        30 => UnitType::TerranSiegeTankSiegeMode,
        31 => UnitType::TankTurretSiegeMode,
        32 => UnitType::Firebat,
        33 => UnitType::ScannerSweep,
        34 => UnitType::TerranMedic,
        35 => UnitType::ZergLarva,
        36 => UnitType::ZergEgg,
        37 => UnitType::ZergZergling,
        38 => UnitType::ZergHydralisk,
        39 => UnitType::ZergUltralisk,
        40 => UnitType::ZergBroodling,
        41 => UnitType::ZergDrone,
        42 => UnitType::ZergOverlord,
        43 => UnitType::ZergMutalisk,
        44 => UnitType::ZergGuardian,
        45 => UnitType::ZergQueen,
        46 => UnitType::ZergDefiler,
        47 => UnitType::ZergScourge,
        48 => UnitType::TorrarsqueUltralisk,
        49 => UnitType::MatriarchQueen,
        50 => UnitType::InfestedTerran,
        51 => UnitType::InfestedKerrigan,
        52 => UnitType::UncleanOneDefiler,
        53 => UnitType::HunterKillerHydralisk,
        54 => UnitType::DevouringOneZergling,
        55 => UnitType::KukulzaMutalisk,
        56 => UnitType::KukulzaGuardian,
        57 => UnitType::YggdrasillOverlord,
        58 => UnitType::TerranValkyrie,
        59 => UnitType::MutaliskOrGuardianCocoon,
        60 => UnitType::ProtossCorsair,
        61 => UnitType::ProtossDarkTemplar,
        62 => UnitType::ZergDevourer,
        63 => UnitType::ProtossDarkArchon,
        64 => UnitType::ProtossProbe,
        65 => UnitType::ProtossZealot,
        66 => UnitType::ProtossDragoon,
        67 => UnitType::ProtossHighTemplar,
        68 => UnitType::ProtossArchon,
        69 => UnitType::ProtossShuttle,
        70 => UnitType::ProtossScout,
        71 => UnitType::ProtossArbiter,
        72 => UnitType::ProtossCarrier,
        73 => UnitType::ProtossInterceptor,
        74 => UnitType::DarkTemplarHero,
        75 => UnitType::ZeratulDarkTemplar,
        76 => UnitType::TassadarAndZeratulArchon,
        77 => UnitType::FenixZealot,
        78 => UnitType::FenixDragoon,
        79 => UnitType::TassadarTemplar,
        80 => UnitType::MojoScout,
        81 => UnitType::WarbringerReaver,
        82 => UnitType::GantrithorCarrier,
        83 => UnitType::ProtossReaver,
        84 => UnitType::ProtossObserver,
        85 => UnitType::ProtossScarab,
        86 => UnitType::DanimothArbiter,
        87 => UnitType::AldarisTemplar,
        88 => UnitType::ArtanisScout,
        89 => UnitType::RhynadonBadlandsCritter,
        90 => UnitType::BengalaasJungleCritter,
        91 => UnitType::UnusedWasCargoShip,
        92 => UnitType::UnusedWasMercenaryGunship,
        93 => UnitType::ScantidDesertCritter,
        94 => UnitType::KakaruTwilightCritter,
        95 => UnitType::RagnasaurAshworldCritter,
        96 => UnitType::UrsadonIceWorldCritter,
        97 => UnitType::LurkerEgg,
        98 => UnitType::Raszagal,
        99 => UnitType::SamirDuranGhost,
        100 => UnitType::AlexeiStukovGhost,
        101 => UnitType::MapRevealer,
        102 => UnitType::GerardDuGalle,
        103 => UnitType::ZergLurker,
        104 => UnitType::InfestedDuran,
        105 => UnitType::DisruptionWeb,
        106 => UnitType::TerranCommandCenter,
        107 => UnitType::TerranComsatStation,
        108 => UnitType::TerranNuclearSilo,
        109 => UnitType::TerranSupplyDepot,
        110 => UnitType::TerranRefinery,
        111 => UnitType::TerranBarracks,
        112 => UnitType::TerranAcademy,
        113 => UnitType::TerranFactory,
        114 => UnitType::TerranStarport,
        115 => UnitType::TerranControlTower,
        116 => UnitType::TerranScienceFacility,
        117 => UnitType::TerranCovertOps,
        118 => UnitType::TerranPhysicsLab,
        119 => UnitType::UnusedWasStarbase,
        120 => UnitType::TerranMachineShop,
        121 => UnitType::UnusedWasRepairBay,
        122 => UnitType::TerranEngineeringBay,
        123 => UnitType::TerranArmory,
        124 => UnitType::TerranMissileTurret,
        125 => UnitType::TerranBunker,
        126 => UnitType::NoradII,
        127 => UnitType::IonCannon,
        128 => UnitType::UrajCrystal,
        129 => UnitType::KhalisCrystal,
        130 => UnitType::InfestedCommandCenter,
        131 => UnitType::ZergHatchery,
        132 => UnitType::ZergLair,
        133 => UnitType::ZergHive,
        134 => UnitType::ZergNydusCanal,
        135 => UnitType::ZergHydraliskDen,
        136 => UnitType::ZergDefilerMound,
        137 => UnitType::ZergGreaterSpire,
        138 => UnitType::ZergQueensNest,
        139 => UnitType::ZergEvolutionChamber,
        140 => UnitType::ZergUltraliskCavern,
        141 => UnitType::ZergSpire,
        142 => UnitType::ZergSpawningPool,
        143 => UnitType::ZergCreepColony,
        144 => UnitType::ZergSporeColony,
        145 => UnitType::UnusedZergBuilding,
        146 => UnitType::ZergSunkenColony,
        147 => UnitType::ZergOvermindWithShell,
        148 => UnitType::ZergOvermind,
        149 => UnitType::ZergExtractor,
        150 => UnitType::MatureChrysalis,
        151 => UnitType::ZergCerebrate,
        152 => UnitType::ZergCerebrateDaggoth,
        153 => UnitType::UnusedZergBuilding5,
        154 => UnitType::ProtossNexus,
        155 => UnitType::ProtossRoboticsFacility,
        156 => UnitType::ProtossPylon,
        157 => UnitType::ProtossAssimilator,
        158 => UnitType::UnusedProtossBuilding1,
        159 => UnitType::ProtossObservatory,
        160 => UnitType::ProtossGateway,
        161 => UnitType::UnusedProtossBuilding2,
        162 => UnitType::ProtossPhotonCannon,
        163 => UnitType::ProtossCitadelofAdun,
        164 => UnitType::ProtossCyberneticsCore,
        165 => UnitType::ProtossTemplarArchives,
        166 => UnitType::ProtossForge,
        167 => UnitType::ProtossStargate,
        168 => UnitType::StasisCellPrison,
        169 => UnitType::ProtossFleetBeacon,
        170 => UnitType::ProtossArbiterTribunal,
        171 => UnitType::ProtossRoboticsSupportBay,
        172 => UnitType::ProtossShieldBattery,
        173 => UnitType::KhaydarinCrystalFormation,
        174 => UnitType::ProtossTemple,
        175 => UnitType::XelNagaTemple,
        176 => UnitType::MineralFieldType1,
        177 => UnitType::MineralFieldType2,
        178 => UnitType::MineralFieldType3,
        179 => UnitType::Cave,
        180 => UnitType::Cavein,
        181 => UnitType::Cantina,
        182 => UnitType::MiningPlatform,
        183 => UnitType::IndependantCommandCenter,
        184 => UnitType::IndependantStarport,
        185 => UnitType::IndependantJumpGate,
        186 => UnitType::Ruins,
        187 => UnitType::KyadarinCrystalFormation,
        188 => UnitType::VespeneGeyser,
        189 => UnitType::WarpGate,
        190 => UnitType::PSIDisruptor,
        191 => UnitType::ZergMarker,
        192 => UnitType::TerranMarker,
        193 => UnitType::ProtossMarker,
        194 => UnitType::ZergBeacon,
        195 => UnitType::TerranBeacon,
        196 => UnitType::ProtossBeacon,
        197 => UnitType::ZergFlagBeacon,
        198 => UnitType::TerranFlagBeacon,
        199 => UnitType::ProtossFlagBeacon,
        200 => UnitType::PowerGenerator,
        201 => UnitType::OvermindCocoon,
        202 => UnitType::DarkSwarm,
        203 => UnitType::FloorMissileTrap,
        204 => UnitType::FloorHatch,
        205 => UnitType::LeftUpperLevelDoor,
        206 => UnitType::RightUpperLevelDoor,
        207 => UnitType::LeftPitDoor,
        208 => UnitType::RightPitDoor,
        209 => UnitType::FloorGunTrap,
        210 => UnitType::LeftWallMissileTrap,
        211 => UnitType::LeftWallFlameTrap,
        212 => UnitType::RightWallMissileTrap,
        213 => UnitType::RightWallFlameTrap,
        214 => UnitType::StartLocation,
        215 => UnitType::Flag,
        216 => UnitType::YoungChrysalis,
        217 => UnitType::PsiEmitter,
        218 => UnitType::DataDisc,
        219 => UnitType::KhaydarinCrystal,
        220 => UnitType::MineralClusterType1,
        221 => UnitType::MineralClusterType2,
        222 => UnitType::ProtossVespeneGasOrbType1,
        223 => UnitType::ProtossVespeneGasOrbType2,
        224 => UnitType::ZergVespeneGasSacType1,
        225 => UnitType::ZergVespeneGasSacType2,
        226 => UnitType::TerranVespeneGasTankType1,
        227 => UnitType::TerranVespeneGasTankType2,
        228 => UnitType::None,
        229 => UnitType::AnyUnit,
        230 => UnitType::Men,
        231 => UnitType::Buildings,
        232 => UnitType::Factories,
        _ => UnitType::Unknown(unit_type as i64),
    }
}

#[derive(Debug, Serialize)]
pub enum Group {
    Unknown(i64),
    Player1,
    Player2,
    Player3,
    Player4,
    Player5,
    Player6,
    Player7,
    Player8,
    Player9,
    Player10,
    Player11,
    Player12,
    None,
    CurrentPlayer,
    Foes,
    Allies,
    NeutralPlayers,
    AllPlayers,
    Force1,
    Force2,
    Force3,
    Force4,
    Unused1,
    Unused2,
    Unused3,
    Unused4,
    NonAlliedVictoryPlayers,
}

#[instrument(level = "trace")]
fn parse_group(group: u32) -> Group {
    match group {
        0 => Group::Player1,
        1 => Group::Player2,
        2 => Group::Player3,
        3 => Group::Player4,
        4 => Group::Player5,
        5 => Group::Player6,
        6 => Group::Player7,
        7 => Group::Player8,
        8 => Group::Player9,
        9 => Group::Player10,
        10 => Group::Player11,
        11 => Group::Player12,
        12 => Group::None,
        13 => Group::CurrentPlayer,
        14 => Group::Foes,
        15 => Group::Allies,
        16 => Group::NeutralPlayers,
        17 => Group::AllPlayers,
        18 => Group::Force1,
        19 => Group::Force2,
        20 => Group::Force3,
        21 => Group::Force4,
        22 => Group::Unused1,
        23 => Group::Unused2,
        24 => Group::Unused3,
        25 => Group::Unused4,
        26 => Group::NonAlliedVictoryPlayers,
        _ => Group::Unknown(group as i64),
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Condition {
    Unknown {
        id: i64,
        raw: ChkTrigCondition,
    },
    // 0
    NoCondition,
    // 1
    CountdownTimer {
        comparison: NumericComparison,
        number: i64,
    },
    // 2
    Command {
        player: Group,
        comparison: NumericComparison,
        unit_type: UnitType,
        number: i64,
    },
    // 3
    Bring {
        player: Group,
        comparison: NumericComparison,
        unit_type: UnitType,
        location: String,
        number: i64,
    },
    // 4
    Accumulate {
        player: Group,
        comparison: NumericComparison,
        unit_type: UnitType,
        number: i64,
        resource_type: ResourceType,
    },
    // 5
    Kill {
        player: Group,
        comparison: NumericComparison,
        unit_type: UnitType,
        number: i64,
    },
    // 6
    CommandsTheMost {
        unit_type: UnitType,
    },
    // 7
    CommandsTheMostAt {
        unit_type: UnitType,
        location: String,
    },
    // 8
    MostKills {
        unit_type: UnitType,
    },
    // 9
    HighestScore {
        score_type: ScoreType,
    },
    // 10
    MostResources {
        resource_type: ResourceType,
    },
    // 11
    Switch {
        switch: i64,
        switch_state: SwitchState,
    },
    // 12
    ElapsedTime {
        comparison: NumericComparison,
        number: i64,
    },
    // 13
    DataIsAMissionBriefing,
    // 14
    Opponents {
        player: Group,
        comparison: NumericComparison,
        number: i64,
    },
    // 15
    Deaths {
        player: Group,
        comparison: NumericComparison,
        unit_type: UnitType,
        number: i64,
        eud_offset: u32,
    },
    // 16
    CommandsTheLeast {
        unit_type: UnitType,
    },
    // 17
    CommandsTheLeastAt {
        unit_type: UnitType,
        location: String,
    },
    // 18
    LeastKills {
        unit_type: UnitType,
    },
    // 19
    LowestScore {
        score_type: ScoreType,
    },
    // 20
    LeastResources {
        resource_type: ResourceType,
    },
    // 21
    Score {
        player: Group,
        comparison: NumericComparison,
        score_type: ScoreType,
        number: i64,
    },
    // 22
    Always,
    // 23
    Never,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Action {
    Unknown {
        id: i64,
        raw: ChkTrigAction,
    },
    // 0
    NoAction,
    // 1
    Victory,
    // 2
    Defeat,
    // 3
    PreserveTrigger,
    // 4
    Wait {
        time: i64,
    },
    // 5
    PauseGame,
    // 6
    UnpauseGame,
    // 7
    Transmission {
        text: String,
        unit_type: UnitType,
        location: String,
        time: i64,
        modifier: NumberModifier,
        wave: String,
        wave_time: i64,
    },
    // 8
    PlayWav {
        wave: String,
        wave_time: i64,
    },
    // 9
    DisplayTextMessage {
        text: String,
    },
    // 10
    CenterView {
        location: String,
    },
    // 11
    CreateUnitWithProperties {
        player: Group,
        unit_type: UnitType,
        number: i64,
        location: String,
        unit_prop: i64,
    },
    // 12
    SetMissionObjectives {
        text: String,
    },
    // 13
    SetSwitch {
        switch: i64,
        switch_action: ActionState,
    },
    // 14
    SetCountdownTimer {
        time: i64,
        modifier: NumberModifier,
    },
    // 15
    RunAIScript {
        script: &'static str,
    },
    // 16
    RunAIScriptAtLocation {
        script: &'static str,
        location: String,
    },
    // 17
    LeaderBoardControl {
        text: String,
        unit_type: UnitType,
    },
    // 18
    LeaderBoardControlAtLocation {
        text: String,
        unit_type: UnitType,
        location: String,
    },
    // 19
    LeaderBoardResources {
        text: String,
        resource_type: ResourceType,
    },
    // 20
    LeaderBoardKills {
        text: String,
        unit_type: UnitType,
    },
    // 21
    LeaderBoardPoints {
        text: String,
        score_type: ScoreType,
    },
    // 22
    KillAllUnits {
        player: Group,
        unit_type: UnitType,
    },
    // 23
    KillUnitAtLocation {
        player: Group,
        unit_type: UnitType,
        number: i64,
        location: String,
    },
    // 24
    RemoveAllUnits {
        player: Group,
        unit_type: UnitType,
    },
    // 25
    RemoveUnitAtLocation {
        player: Group,
        unit_type: UnitType,
        number: i64,
        location: String,
    },
    // 26
    SetResources {
        player: Group,
        number: i64,
        modifier: NumberModifier,
        resource_type: ResourceType,
    },
    // 27
    SetScore {
        player: Group,
        number: i64,
        modifier: NumberModifier,
        score_type: ScoreType,
    },
    // 28
    MinimapPing {
        location: String,
    },
    // 29
    TalkingPortrait {
        unit_type: UnitType,
        time: i64,
    },
    // 30
    MuteUnitSpeech,
    // 31
    UnmuteUnitSpeech,
    // 32
    LeaderBoardComputerPlayers {
        action: ActionState,
    },
    // 33
    LeaderBoardGoalControl {
        text: String,
        unit_type: UnitType,
    },
    // 34
    LeaderBoardGoalControlAtLocation {
        text: String,
        unit_type: UnitType,
        location: String,
    },
    // 35
    LeaderBoardGoalResources {
        text: String,
        resource_type: ResourceType,
    },
    // 36
    LeaderBoardGoalKills {
        text: String,
        unit_type: UnitType,
    },
    // 37
    LeaderBoardGoalPoints {
        text: String,
        score_type: ScoreType,
    },
    // 38
    MoveLocation {
        player: Group,
        unit_type: UnitType,
        source_location: String,
        destination_location: String,
    },
    // 39
    MoveUnit {
        player: Group,
        unit_type: UnitType,
        number: i64,
        source_location: String,
        destination_location: String,
    },
    // 40
    LeaderboardGreed {
        number: i64,
    },
    // 41
    SetNextScenario {
        text: String,
    },
    // 42
    SetDoodadState {
        player: Group,
        unit_type: UnitType,
        location: String,
        state: ActionState,
    },
    // 43
    SetInvincibility {
        player: Group,
        unit_type: UnitType,
        location: String,
        state: ActionState,
    },
    // 44
    CreateUnit {
        player: Group,
        unit_type: UnitType,
        number: i64,
        location: String,
    },
    // 45
    SetDeaths {
        player: Group,
        unit_type: UnitType,
        number: i64,
        modifier: NumberModifier,
        eud_offset: u32,
    },
    // 46
    Order {
        player: Group,
        unit_type: UnitType,
        source_location: String,
        destination_location: String,
        order: Order,
    },
    // 47
    Comment {
        text: String,
    },
    // 48
    GiveUnitsToPlayer {
        source_player: Group,
        destination_player: Group,
        unit_type: UnitType,
        number: i64,
        location: String,
    },
    // 49
    ModifyUnitHitPoints {
        player: Group,
        unit_type: UnitType,
        number: i64,
        mod_ammount: i64,
        location: String,
    },
    // 50
    ModifyUnitEnergy {
        player: Group,
        unit_type: UnitType,
        number: i64,
        mod_ammount: i64,
        location: String,
    },
    // 51
    ModifyUnitShieldPoints {
        player: Group,
        unit_type: UnitType,
        number: i64,
        mod_ammount: i64,
        location: String,
    },
    // 52
    ModifyUnitResource {
        player: Group,
        unit_type: UnitType,
        number: i64,
        mod_ammount: i64,
        location: String,
    },
    // 53
    ModifyUnitHangarCount {
        player: Group,
        unit_type: UnitType,
        number: i64,
        mod_ammount: i64,
        location: String,
    },
    // 54
    PauseTimer,
    // 55
    UnpauseTimer,
    // 56
    Draw,
    // 57
    SetAllianceStatus {
        player: Group,
        alliance_status: AllianceStatus,
    },
    // 58
    DisableDebugMode,
    // 59
    EnableDebugMode,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum MissionBriefingCondition {
    Unknown { id: i64, raw: ChkMbrfCondition },
    // 0
    NoCondition,
    // 13
    DataIsAMissionBriefing,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum MissionBriefingAction {
    Unknown {
        id: i64,
        raw: ChkMbrfAction,
    },
    // 0
    NoAction,
    // 1
    Wait {
        time: i64,
    },
    // 2
    PlayWav {
        wave: String,
        wave_time: i64,
    },
    // 3
    DisplayTextMessage {
        text: String,
    },
    // 4
    SetMissionObjectives {
        text: String,
    },
    // 5
    ShowPortrait {
        unit_type: UnitType,
        slot: i64,
    },
    // 6
    HidePortrait {
        slot: i64,
    },
    // 7
    DisplaySpeakingPortrait {
        unit_type: UnitType,
        slot: i64,
    },
    // 8
    DisplayTransmission {
        text: String,
        slot: i64,
        time: i64,
        modifier: NumberModifier,
        wave: String,
        wave_time: i64,
    },
    // 9
    SkipTutorialEnabled,
}

#[derive(Debug, Serialize)]
pub struct MissionBriefing {
    pub conditions: Vec<MissionBriefingCondition>,
    pub actions: Vec<MissionBriefingAction>,
    pub execution_flags: u32,
    pub activated_for_players: [u8; 27],
    pub index_of_current_action: u8,
}

#[instrument(skip_all)]
pub fn parse_mission_briefing<'a>(parsed_chk: &ParsedChk<'a>) -> Vec<MissionBriefing> {
    let mut ret = Vec::new();

    if let Ok(trig) = &parsed_chk.mbrf {
        for trigger in &trig.triggers {
            let mut conditions = Vec::new();
            for condition in trigger.conditions {
                match condition.condition {
                    0 => {
                        //conditions.push(MissionBriefingCondition::NoCondition);
                    }
                    13 => {
                        conditions.push(MissionBriefingCondition::DataIsAMissionBriefing);
                    }
                    _ => {
                        conditions.push(MissionBriefingCondition::Unknown {
                            id: condition.condition as i64,
                            raw: condition,
                        });
                    }
                }
            }
            let mut actions = Vec::new();
            for action in trigger.actions {
                match action.action {
                    0 => {
                        //actions.push(MissionBriefingAction::NoAction);
                    }
                    1 => {
                        actions.push(MissionBriefingAction::Wait {
                            time: action.seconds_or_milliseconds as i64,
                        });
                    }
                    2 => {
                        actions.push(MissionBriefingAction::PlayWav {
                            wave: parsed_chk
                                .get_string(action.wav_string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            wave_time: action.seconds_or_milliseconds as i64,
                        });
                    }
                    3 => {
                        actions.push(MissionBriefingAction::DisplayTextMessage {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    4 => {
                        actions.push(MissionBriefingAction::SetMissionObjectives {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    5 => {
                        actions.push(MissionBriefingAction::ShowPortrait {
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            slot: action.first_or_only_group_or_player_affected as i64,
                        });
                    }
                    6 => {
                        actions.push(MissionBriefingAction::HidePortrait {
                            slot: action.first_or_only_group_or_player_affected as i64,
                        });
                    }
                    7 => {
                        actions.push(MissionBriefingAction::DisplaySpeakingPortrait {
                            slot: action.first_or_only_group_or_player_affected as i64,
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    8 => {
                        actions.push(MissionBriefingAction::DisplayTransmission {
                            slot: action.first_or_only_group_or_player_affected as i64,
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            time: action.seconds_or_milliseconds as i64,
                            modifier: parse_number_modifier(
                                action.number_of_units_or_action_state_or_unit_order_or_number_modifier,
                            ),
                            wave: parsed_chk
                                .get_string(action.wav_string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            wave_time: action.seconds_or_milliseconds as i64,
                        });
                    }
                    9 => {
                        actions.push(MissionBriefingAction::SkipTutorialEnabled);
                    }
                    _ => {
                        actions.push(MissionBriefingAction::Unknown {
                            id: action.action as i64,
                            raw: action,
                        });
                    }
                }
            }

            ret.push(MissionBriefing {
                conditions,
                actions,
                execution_flags: trigger.execution_flags,
                activated_for_players: trigger.executed_for_player,
                index_of_current_action: trigger.current_action,
            });
        }
    }

    ret
}

#[derive(Debug, Serialize)]
pub struct Trigger {
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub execution_flags: u32,
    pub activated_for_players: [u8; 27],
    pub index_of_current_action: u8,
}

#[instrument(skip_all)]
pub fn parse_triggers<'a>(parsed_chk: &ParsedChk<'a>) -> Vec<Trigger> {
    let mut ret = Vec::new();

    if let Ok(trig) = &parsed_chk.trig {
        for trigger in &trig.triggers {
            let mut conditions = Vec::new();
            for condition in trigger.conditions {
                match condition.condition {
                    0 => {
                        //conditions.push(Condition::NoCondition)
                    }
                    1 => {
                        conditions.push(Condition::CountdownTimer {
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            number: condition.qualified_number as i64,
                        });
                    }
                    2 => {
                        conditions.push(Condition::Command {
                            player: parse_group(condition.group),
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            unit_type: parse_unit_type(condition.unit_id),
                            number: condition.qualified_number as i64,
                        });
                    }
                    3 => {
                        conditions.push(Condition::Bring {
                            player: parse_group(condition.group),
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            unit_type: parse_unit_type(condition.unit_id),
                            location: parsed_chk
                                .get_location_name(condition.location as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            number: condition.qualified_number as i64,
                        });
                    }
                    4 => {
                        conditions.push(Condition::Accumulate {
                            player: parse_group(condition.group),
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            unit_type: parse_unit_type(condition.unit_id),
                            number: condition.qualified_number as i64,
                            resource_type: parse_resource_type(
                                condition.resource_type_or_score_type_or_switch_number as u16,
                            ),
                        });
                    }
                    5 => {
                        conditions.push(Condition::Kill {
                            player: parse_group(condition.group),
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            unit_type: parse_unit_type(condition.unit_id),
                            number: condition.qualified_number as i64,
                        });
                    }
                    6 => {
                        conditions.push(Condition::CommandsTheMost {
                            unit_type: parse_unit_type(condition.unit_id),
                        });
                    }
                    7 => {
                        conditions.push(Condition::CommandsTheMostAt {
                            unit_type: parse_unit_type(condition.unit_id),
                            location: parsed_chk
                                .get_location_name(condition.location as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    8 => {
                        conditions.push(Condition::MostKills {
                            unit_type: parse_unit_type(condition.unit_id),
                        });
                    }
                    9 => {
                        conditions.push(Condition::HighestScore {
                            score_type: parse_score_type(
                                condition.resource_type_or_score_type_or_switch_number as u16,
                            ),
                        });
                    }
                    10 => {
                        conditions.push(Condition::MostResources {
                            resource_type: parse_resource_type(
                                condition.resource_type_or_score_type_or_switch_number as u16,
                            ),
                        });
                    }
                    11 => {
                        conditions.push(Condition::Switch {
                            switch: condition.resource_type_or_score_type_or_switch_number as i64,
                            switch_state: parse_switch_state(
                                condition.numeric_comparison_or_switch_state,
                            ),
                        });
                    }
                    12 => {
                        conditions.push(Condition::ElapsedTime {
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            number: condition.qualified_number as i64,
                        });
                    }
                    13 => {
                        conditions.push(Condition::DataIsAMissionBriefing);
                    }
                    14 => {
                        conditions.push(Condition::Opponents {
                            player: parse_group(condition.group),
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            number: condition.qualified_number as i64,
                        });
                    }
                    15 => {
                        conditions.push(Condition::Deaths {
                            player: parse_group(condition.group),
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            unit_type: parse_unit_type(condition.unit_id),
                            number: condition.qualified_number as i64,
                            eud_offset: (condition.unit_id as u32)
                                .wrapping_mul(12)
                                .wrapping_add(condition.group)
                                .wrapping_mul(4)
                                .wrapping_add(0x58A364),
                        });
                    }
                    16 => {
                        conditions.push(Condition::CommandsTheLeast {
                            unit_type: parse_unit_type(condition.unit_id),
                        });
                    }
                    17 => {
                        conditions.push(Condition::CommandsTheLeastAt {
                            unit_type: parse_unit_type(condition.unit_id),
                            location: parsed_chk
                                .get_location_name(condition.location as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    18 => {
                        conditions.push(Condition::LeastKills {
                            unit_type: parse_unit_type(condition.unit_id),
                        });
                    }
                    19 => {
                        conditions.push(Condition::LowestScore {
                            score_type: parse_score_type(
                                condition.resource_type_or_score_type_or_switch_number as u16,
                            ),
                        });
                    }
                    20 => {
                        conditions.push(Condition::LeastResources {
                            resource_type: parse_resource_type(
                                condition.resource_type_or_score_type_or_switch_number as u16,
                            ),
                        });
                    }
                    21 => {
                        conditions.push(Condition::Score {
                            player: parse_group(condition.group),
                            comparison: parse_numeric_comparison(
                                condition.numeric_comparison_or_switch_state,
                            ),
                            score_type: parse_score_type(
                                condition.resource_type_or_score_type_or_switch_number as u16,
                            ),
                            number: condition.qualified_number as i64,
                        });
                    }
                    22 => {
                        conditions.push(Condition::Always);
                    }
                    23 => {
                        conditions.push(Condition::Never);
                    }
                    _ => {
                        conditions.push(Condition::Unknown {
                            id: condition.condition as i64,
                            raw: condition,
                        });
                    }
                }
            }

            let mut actions = Vec::new();

            for action in trigger.actions {
                match action.action {
                    0 => {}
                    1 => {
                        actions.push(Action::Victory);
                    }
                    2 => {
                        actions.push(Action::Defeat);
                    }
                    3 => {
                        actions.push(Action::PreserveTrigger);
                    }
                    4 => {
                        actions.push(Action::Wait {
                            time: action.seconds_or_milliseconds as i64,
                        });
                    }
                    5 => {
                        actions.push(Action::PauseGame);
                    }
                    6 => {
                        actions.push(Action::UnpauseGame);
                    }
                    7 => {
                        actions.push(Action::Transmission {
                            text: parsed_chk.get_string(action.string_number as usize).unwrap_or("couldn't get string".to_owned()),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                            time: action.seconds_or_milliseconds as i64,
                            modifier: parse_number_modifier(
                                action.number_of_units_or_action_state_or_unit_order_or_number_modifier,
                            ),
                            wave: parsed_chk.get_string(action.wav_string_number as usize).unwrap_or("couldn't get string".to_owned()),
                            // TODO: not sure what this is supposed to be.
                            wave_time: 0,
                        });
                    }
                    8 => {
                        actions.push(Action::PlayWav {
                            wave: parsed_chk
                                .get_string(action.wav_string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            // TODO: not sure what this is supposed to be.
                            wave_time: 0,
                        });
                    }
                    9 => {
                        actions.push(Action::DisplayTextMessage {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    10 => {
                        actions.push(Action::CenterView {
                            location: parsed_chk
                                .get_location_name(action.location as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    11 => {
                        actions.push(Action::CreateUnitWithProperties {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                            unit_prop: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                        });
                    }
                    12 => {
                        actions.push(Action::SetMissionObjectives {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    13 => {
                        actions.push(Action::SetSwitch {
                            switch: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            switch_action: parse_action_state(action.number_of_units_or_action_state_or_unit_order_or_number_modifier),
                        });
                    }
                    14 => {
                        actions.push(Action::SetCountdownTimer {
                        time: action.seconds_or_milliseconds as i64,
                        modifier: parse_number_modifier(
                            action.number_of_units_or_action_state_or_unit_order_or_number_modifier,
                        ),
                    });
                    }
                    15 => {
                        actions.push(Action::RunAIScript {
                        script: parse_ai_script(action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number),
                    });
                    }
                    16 => {
                        actions.push(Action::RunAIScriptAtLocation {
                            script: parse_ai_script(action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number),
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    17 => {
                        actions.push(Action::LeaderBoardControl {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    18 => {
                        actions.push(Action::LeaderBoardControlAtLocation {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            location: parsed_chk
                                .get_location_name(action.location as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    19 => {
                        actions.push(Action::LeaderBoardResources {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            resource_type: parse_resource_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    20 => {
                        actions.push(Action::LeaderBoardKills {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    21 => {
                        actions.push(Action::LeaderBoardPoints {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            score_type: parse_score_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    22 => {
                        actions.push(Action::KillAllUnits {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    23 => {
                        actions.push(Action::KillUnitAtLocation {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    24 => {
                        actions.push(Action::RemoveAllUnits {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    25 => {
                        actions.push(Action::RemoveUnitAtLocation {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    26 => {
                        actions.push(Action::SetResources {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            modifier: parse_number_modifier(action.number_of_units_or_action_state_or_unit_order_or_number_modifier),
                            resource_type: parse_resource_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    27 => {
                        actions.push(Action::SetScore {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            modifier: parse_number_modifier(action.number_of_units_or_action_state_or_unit_order_or_number_modifier),
                            score_type: parse_score_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    28 => {
                        actions.push(Action::MinimapPing {
                            location: parsed_chk
                                .get_location_name(action.location as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    29 => {
                        actions.push(Action::TalkingPortrait {
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            time: action.seconds_or_milliseconds as i64,
                        });
                    }
                    30 => {
                        actions.push(Action::MuteUnitSpeech);
                    }
                    31 => {
                        actions.push(Action::UnmuteUnitSpeech);
                    }
                    32 => {
                        actions.push(Action::LeaderBoardComputerPlayers {
                            action: parse_action_state(action.number_of_units_or_action_state_or_unit_order_or_number_modifier),
                        });
                    }
                    33 => {
                        actions.push(Action::LeaderBoardGoalControl {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    34 => {
                        actions.push(Action::LeaderBoardGoalControlAtLocation {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            location: parsed_chk
                                .get_location_name(action.location as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    35 => {
                        actions.push(Action::LeaderBoardGoalResources {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            resource_type: parse_resource_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    36 => {
                        actions.push(Action::LeaderBoardGoalKills {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    37 => {
                        actions.push(Action::LeaderBoardGoalPoints {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                            score_type: parse_score_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    38 => {
                        actions.push(Action::MoveLocation {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            source_location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                            destination_location: parsed_chk.get_location_name(action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    39 => {
                        actions.push(Action::MoveUnit {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            source_location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                            destination_location: parsed_chk.get_location_name(action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as usize).unwrap_or("couldn't get string".to_owned()),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                        });
                    }
                    40 => {
                        actions.push(Action::LeaderboardGreed {
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                        });
                    }
                    41 => {
                        actions.push(Action::SetNextScenario {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    42 => {
                        actions.push(Action::SetDoodadState {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                            state: parse_action_state(action.number_of_units_or_action_state_or_unit_order_or_number_modifier), // TODO: split actionstate enum into doodad state + switch state + computer-player state.
                        });
                    }
                    43 => {
                        actions.push(Action::SetInvincibility {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                            state: parse_action_state(action.number_of_units_or_action_state_or_unit_order_or_number_modifier), // TODO: split actionstate enum into doodad state + switch state + computer-player state.
                        });
                    }
                    44 => {
                        actions.push(Action::CreateUnit {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    45 => {
                        actions.push(Action::SetDeaths {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            modifier: parse_number_modifier(action.number_of_units_or_action_state_or_unit_order_or_number_modifier),
                            eud_offset:
                            (action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status as u32).wrapping_mul(12).wrapping_add(action.first_or_only_group_or_player_affected).wrapping_mul(4).wrapping_add(0x58A364)
                        });
                    }
                    46 => {
                        actions.push(Action::Order {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            source_location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                            destination_location: parsed_chk.get_location_name(action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as usize).unwrap_or("couldn't get string".to_owned()),
                            order: parse_order(action.number_of_units_or_action_state_or_unit_order_or_number_modifier)
                        });
                    }
                    47 => {
                        actions.push(Action::Comment {
                            text: parsed_chk
                                .get_string(action.string_number as usize)
                                .unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    48 => {
                        actions.push(Action::GiveUnitsToPlayer {
                            source_player: parse_group(action.first_or_only_group_or_player_affected),
                            destination_player: parse_group(action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    49 => {
                        actions.push(Action::ModifyUnitHitPoints {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            mod_ammount: 0, // TODO: what is this.
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    50 => {
                        actions.push(Action::ModifyUnitEnergy {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            mod_ammount: 0, // TODO: what is this.
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    51 => {
                        actions.push(Action::ModifyUnitShieldPoints {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            mod_ammount: 0, // TODO: what is this.
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    52 => {
                        actions.push(Action::ModifyUnitResource { // TOOD: ModifyUnitResourceAmount
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            mod_ammount: 0, // TODO: what is this.
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    53 => {
                        actions.push(Action::ModifyUnitHangarCount {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            unit_type: parse_unit_type(action
                                .unit_type_or_score_type_or_resource_type_or_alliance_status),
                            number: action.second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number as i64,
                            mod_ammount: 0, // TODO: what is this.
                            location: parsed_chk.get_location_name(action.location as usize).unwrap_or("couldn't get string".to_owned()),
                        });
                    }
                    54 => {
                        actions.push(Action::PauseTimer);
                    }
                    55 => {
                        actions.push(Action::UnpauseTimer);
                    }
                    56 => {
                        actions.push(Action::Draw);
                    }
                    57 => {
                        actions.push(Action::SetAllianceStatus {
                            player: parse_group(action.first_or_only_group_or_player_affected),
                            alliance_status: parse_alliance_status(
                                action.unit_type_or_score_type_or_resource_type_or_alliance_status,
                            ),
                        });
                    }
                    58 => {
                        actions.push(Action::DisableDebugMode);
                    }
                    59 => {
                        actions.push(Action::EnableDebugMode);
                    }
                    _ => {
                        actions.push(Action::Unknown {
                            id: action.action as i64,
                            raw: action,
                        });
                    }
                }
            }

            ret.push(Trigger {
                conditions,
                actions,
                execution_flags: trigger.execution_flags,
                activated_for_players: trigger.executed_for_player,
                index_of_current_action: trigger.current_action,
            });
        }
    }

    ret
}
