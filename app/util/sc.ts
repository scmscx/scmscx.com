/**
 * The i18n key for a unit, not its English name.
 *
 * These feed I18nSpan, so they have to be keys. The table currently holds the
 * English text in every language as a placeholder -- Korean players do use
 * Korean unit names, so this is a backlog item rather than a decision; run
 * `make i18n` for the list of keys still awaiting a translator.
 */
const unit_id_to_name = (unitId: number) => {
  switch (unitId) {
    case 0:
      return "unit.terran_marine";
    case 1:
      return "unit.terran_ghost";
    case 2:
      return "unit.terran_vulture";
    case 3:
      return "unit.terran_goliath";
    case 4:
      return "unit.goliath_turret";
    case 5:
      return "unit.terran_siege_tank_tank_mode";
    case 6:
      return "unit.tank_turret_tank_mode";
    case 7:
      return "unit.terran_scv";
    case 8:
      return "unit.terran_wraith";
    case 9:
      return "unit.terran_science_vessel";
    case 10:
      return "unit.gui_montag_firebat";
    case 11:
      return "unit.terran_dropship";
    case 12:
      return "unit.terran_battlecruiser";
    case 13:
      return "unit.vulture_spider_mine";
    case 14:
      return "unit.nuclear_missile";
    case 15:
      return "unit.terran_civilian";
    case 16:
      return "unit.sarah_kerrigan_ghost";
    case 17:
      return "unit.alan_schezar_goliath";
    case 18:
      return "unit.alan_schezar_turret";
    case 19:
      return "unit.jim_raynor_vulture";
    case 20:
      return "unit.jim_raynor_marine";
    case 21:
      return "unit.tom_kazansky_wraith";
    case 22:
      return "unit.magellan_science_vessel";
    case 23:
      return "unit.edmund_duke_siege_tank";
    case 24:
      return "unit.edmund_duke_turret";
    case 25:
      return "unit.edmund_duke_siege_mode";
    case 26:
      return "unit.edmund_duke_turret";
    case 27:
      return "unit.arcturus_mengsk_battlecruiser";
    case 28:
      return "unit.hyperion_battlecruiser";
    case 29:
      return "unit.norad_ii_battlecruiser";
    case 30:
      return "unit.terran_siege_tank_siege_mode";
    case 31:
      return "unit.tank_turret_siege_mode";
    case 32:
      return "unit.firebat";
    case 33:
      return "unit.scanner_sweep";
    case 34:
      return "unit.terran_medic";
    case 35:
      return "unit.zerg_larva";
    case 36:
      return "unit.zerg_egg";
    case 37:
      return "unit.zerg_zergling";
    case 38:
      return "unit.zerg_hydralisk";
    case 39:
      return "unit.zerg_ultralisk";
    case 40:
      return "unit.zerg_broodling";
    case 41:
      return "unit.zerg_drone";
    case 42:
      return "unit.zerg_overlord";
    case 43:
      return "unit.zerg_mutalisk";
    case 44:
      return "unit.zerg_guardian";
    case 45:
      return "unit.zerg_queen";
    case 46:
      return "unit.zerg_defiler";
    case 47:
      return "unit.zerg_scourge";
    case 48:
      return "unit.torrarsque_ultralisk";
    case 49:
      return "unit.matriarch_queen";
    case 50:
      return "unit.infested_terran";
    case 51:
      return "unit.infested_kerrigan";
    case 52:
      return "unit.unclean_one_defiler";
    case 53:
      return "unit.hunter_killer_hydralisk";
    case 54:
      return "unit.devouring_one_zergling";
    case 55:
      return "unit.kukulza_mutalisk";
    case 56:
      return "unit.kukulza_guardian";
    case 57:
      return "unit.yggdrasill_overlord";
    case 58:
      return "unit.terran_valkyrie_frigate";
    case 59:
      return "unit.mutalisk_guardian_cocoon";
    case 60:
      return "unit.protoss_corsair";
    case 61:
      return "unit.protoss_dark_templar_unit";
    case 62:
      return "unit.zerg_devourer";
    case 63:
      return "unit.protoss_dark_archon";
    case 64:
      return "unit.protoss_probe";
    case 65:
      return "unit.protoss_zealot";
    case 66:
      return "unit.protoss_dragoon";
    case 67:
      return "unit.protoss_high_templar";
    case 68:
      return "unit.protoss_archon";
    case 69:
      return "unit.protoss_shuttle";
    case 70:
      return "unit.protoss_scout";
    case 71:
      return "unit.protoss_arbiter";
    case 72:
      return "unit.protoss_carrier";
    case 73:
      return "unit.protoss_interceptor";
    case 74:
      return "unit.dark_templar_hero";
    case 75:
      return "unit.zeratul_dark_templar";
    case 76:
      return "unit.tassadar_zeratul_archon";
    case 77:
      return "unit.fenix_zealot";
    case 78:
      return "unit.fenix_dragoon";
    case 79:
      return "unit.tassadar_templar";
    case 80:
      return "unit.mojo_scout";
    case 81:
      return "unit.warbringer_reaver";
    case 82:
      return "unit.gantrithor_carrier";
    case 83:
      return "unit.protoss_reaver";
    case 84:
      return "unit.protoss_observer";
    case 85:
      return "unit.protoss_scarab";
    case 86:
      return "unit.danimoth_arbiter";
    case 87:
      return "unit.aldaris_templar";
    case 88:
      return "unit.artanis_scout";
    case 89:
      return "unit.rhynadon_badlands_critter";
    case 90:
      return "unit.bengalaas_jungle_critter";
    case 91:
      return "unit.unused_was_cargo_ship";
    case 92:
      return "unit.unused_was_mercenary_gunship";
    case 93:
      return "unit.scantid_desert_critter";
    case 94:
      return "unit.kakaru_twilight_critter";
    case 95:
      return "unit.ragnasaur_ashworld_critter";
    case 96:
      return "unit.ursadon_ice_world_critter";
    case 97:
      return "unit.lurker_egg";
    case 98:
      return "unit.raszagal_corsair";
    case 99:
      return "unit.samir_duran_ghost";
    case 100:
      return "unit.alexei_stukov_ghost";
    case 101:
      return "unit.map_revealer";
    case 102:
      return "unit.gerard_dugalle_battlecruiser";
    case 103:
      return "unit.zerg_lurker";
    case 104:
      return "unit.infested_duran";
    case 105:
      return "unit.disruption_web";
    case 106:
      return "unit.terran_command_center";
    case 107:
      return "unit.terran_comsat_station";
    case 108:
      return "unit.terran_nuclear_silo";
    case 109:
      return "unit.terran_supply_depot";
    case 110:
      return "unit.terran_refinery";
    case 111:
      return "unit.terran_barracks";
    case 112:
      return "unit.terran_academy";
    case 113:
      return "unit.terran_factory";
    case 114:
      return "unit.terran_starport";
    case 115:
      return "unit.terran_control_tower";
    case 116:
      return "unit.terran_science_facility";
    case 117:
      return "unit.terran_covert_ops";
    case 118:
      return "unit.terran_physics_lab";
    case 119:
      return "unit.unused_was_starbase";
    case 120:
      return "unit.terran_machine_shop";
    case 121:
      return "unit.unused_was_repair_bay";
    case 122:
      return "unit.terran_engineering_bay";
    case 123:
      return "unit.terran_armory";
    case 124:
      return "unit.terran_missile_turret";
    case 125:
      return "unit.terran_bunker";
    case 126:
      return "unit.norad_ii";
    case 127:
      return "unit.ion_cannon";
    case 128:
      return "unit.uraj_crystal";
    case 129:
      return "unit.khalis_crystal";
    case 130:
      return "unit.infested_command_center";
    case 131:
      return "unit.zerg_hatchery";
    case 132:
      return "unit.zerg_lair";
    case 133:
      return "unit.zerg_hive";
    case 134:
      return "unit.zerg_nydus_canal";
    case 135:
      return "unit.zerg_hydralisk_den";
    case 136:
      return "unit.zerg_defiler_mound";
    case 137:
      return "unit.zerg_greater_spire";
    case 138:
      return "unit.zerg_queen_s_nest";
    case 139:
      return "unit.zerg_evolution_chamber";
    case 140:
      return "unit.zerg_ultralisk_cavern";
    case 141:
      return "unit.zerg_spire";
    case 142:
      return "unit.zerg_spawning_pool";
    case 143:
      return "unit.zerg_creep_colony";
    case 144:
      return "unit.zerg_spore_colony";
    case 145:
      return "unit.unused_zerg_building";
    case 146:
      return "unit.zerg_sunken_colony";
    case 147:
      return "unit.zerg_overmind_with_shell";
    case 148:
      return "unit.zerg_overmind";
    case 149:
      return "unit.zerg_extractor";
    case 150:
      return "unit.mature_chrysalis";
    case 151:
      return "unit.zerg_cerebrate";
    case 152:
      return "unit.zerg_cerebrate_daggoth";
    case 153:
      return "unit.unused_zerg_building_5";
    case 154:
      return "unit.protoss_nexus";
    case 155:
      return "unit.protoss_robotics_facility";
    case 156:
      return "unit.protoss_pylon";
    case 157:
      return "unit.protoss_assimilator";
    case 158:
      return "unit.unused_protoss_building";
    case 159:
      return "unit.protoss_observatory";
    case 160:
      return "unit.protoss_gateway";
    case 161:
      return "unit.unused_protoss_building";
    case 162:
      return "unit.protoss_photon_cannon";
    case 163:
      return "unit.protoss_citadel_of_adun";
    case 164:
      return "unit.protoss_cybernetics_core";
    case 165:
      return "unit.protoss_templar_archives";
    case 166:
      return "unit.protoss_forge";
    case 167:
      return "unit.protoss_stargate";
    case 168:
      return "unit.stasis_cell_prison";
    case 169:
      return "unit.protoss_fleet_beacon";
    case 170:
      return "unit.protoss_arbiter_tribunal";
    case 171:
      return "unit.protoss_robotics_support_bay";
    case 172:
      return "unit.protoss_shield_battery";
    case 173:
      return "unit.khaydarin_crystal_formation";
    case 174:
      return "unit.protoss_temple";
    case 175:
      return "unit.xel_naga_temple";
    case 176:
      return "unit.mineral_field_type_1";
    case 177:
      return "unit.mineral_field_type_2";
    case 178:
      return "unit.mineral_field_type_3";
    case 179:
      return "unit.cave";
    case 180:
      return "unit.cave_in";
    case 181:
      return "unit.cantina";
    case 182:
      return "unit.mining_platform";
    case 183:
      return "unit.independant_command_center";
    case 184:
      return "unit.independant_starport";
    case 185:
      return "unit.jump_gate";
    case 186:
      return "unit.ruins";
    case 187:
      return "unit.kyadarin_crystal_formation";
    case 188:
      return "unit.vespene_geyser";
    case 189:
      return "unit.warp_gate";
    case 190:
      return "unit.psi_disruptor";
    case 191:
      return "unit.zerg_marker";
    case 192:
      return "unit.terran_marker";
    case 193:
      return "unit.protoss_marker";
    case 194:
      return "unit.zerg_beacon";
    case 195:
      return "unit.terran_beacon";
    case 196:
      return "unit.protoss_beacon";
    case 197:
      return "unit.zerg_flag_beacon";
    case 198:
      return "unit.terran_flag_beacon";
    case 199:
      return "unit.protoss_flag_beacon";
    case 200:
      return "unit.power_generator";
    case 201:
      return "unit.overmind_cocoon";
    case 202:
      return "unit.dark_swarm";
    case 203:
      return "unit.floor_missile_trap";
    case 204:
      return "unit.floor_hatch";
    case 205:
      return "unit.left_upper_level_door";
    case 206:
      return "unit.right_upper_level_door";
    case 207:
      return "unit.left_pit_door";
    case 208:
      return "unit.right_pit_door";
    case 209:
      return "unit.floor_gun_trap";
    case 210:
      return "unit.left_wall_missile_trap";
    case 211:
      return "unit.left_wall_flame_trap";
    case 212:
      return "unit.right_wall_missile_trap";
    case 213:
      return "unit.right_wall_flame_trap";
    case 214:
      return "unit.start_location";
    case 215:
      return "unit.flag";
    case 216:
      return "unit.young_chrysalis";
    case 217:
      return "unit.psi_emitter";
    case 218:
      return "unit.data_disc";
    case 219:
      return "unit.khaydarin_crystal";
    case 220:
      return "unit.mineral_cluster_type_1";
    case 221:
      return "unit.mineral_cluster_type_2";
    case 222:
      return "unit.protoss_vespene_gas_orb_type_1";
    case 223:
      return "unit.protoss_vespene_gas_orb_type_2";
    case 224:
      return "unit.zerg_vespene_gas_sac_type_1";
    case 225:
      return "unit.zerg_vespene_gas_sac_type_2";
    case 226:
      return "unit.terran_vespene_gas_tank_type_1";
    case 227:
      return "unit.terran_vespene_gas_tank_type_2";

    default:
      return "unit.unknown";
  }
};

/** The i18n key for a map's version. See `unit_id_to_name`. */
const map_ver_to_string = (ver: number) => {
  switch (ver) {
    case 206:
      return "map.version_remastered_1_21";
    case 205:
      return "map.version_broodwar_1_04";
    case 64:
      return "map.version_starcraft_remastered_1_21_hybrid";
    case 63:
      return "map.version_starcraft_1_04_hybrid";

    case 59:
      return "map.version_starcraft_1_00";

    case 61:
    case 75:
    case 201:
    case 203:
      return "map.version_broodwar_internal";

    case 47:
      return "map.version_starcraft_beta";
    case 1:
    case 2:
    case 3:
    case 4:
    case 5:
    case 6:
    case 7:
    case 8:
    case 9:
    case 10:
    case 11:
    case 12:
    case 13:
    case 14:
    case 15:
    case 16:
    case 17:
    case 18:
    case 19:
      return "map.version_warcraft_ii";

    default:
      return "map.version_unknown";
  }
};

/**
 * The i18n key for a player slot's owner, not its English name.
 *
 * These are looked up in the string table, so they must be keys rather than
 * display text -- returning English here silently rendered untranslated and
 * logged a missing key per player slot.
 */
const map_player_owners_to_strings = (ownr: number) => {
  switch (ownr) {
    case 0:
      return "map.player_owner_inactive";
    case 1:
      return "map.player_owner_computer_game";
    case 2:
      return "map.player_owner_human";
    case 3:
      return "map.player_owner_rescue_passive";
    case 4:
      return "map.player_owner_unused";
    case 5:
      return "map.player_owner_computer";
    case 6:
      return "map.player_owner_open";
    case 7:
      return "map.player_owner_neutral";
    case 8:
      return "map.player_owner_closed";
    default:
      return "map.player_owner_invalid";
  }
};

/** The i18n key for a player's race. See `map_player_owners_to_strings`. */
const map_player_side_to_strings = (ownr: number) => {
  switch (ownr) {
    case 0:
      return "map.player_side_zerg";
    case 1:
      return "map.player_side_terran";
    case 2:
      return "map.player_side_protoss";
    case 3:
      return "map.player_side_invalid_independent";
    case 4:
      return "map.player_side_invalid_neutral";
    case 5:
      return "map.player_side_user_select";
    case 6:
      return "map.player_side_random";
    case 7:
      return "map.player_side_inactive";
    default:
      return "map.player_side_invalid";
  }
};

/**
 * The i18n key for a tileset, rather than its English name.
 *
 * `map_era_to_tileset` returns display text, which used to double as a
 * translation key back when the table was keyed by English. It no longer is, so
 * feeding its output to I18nSpan silently rendered English and logged a missing
 * key per row. Keys are looked up from the era directly, which also fixes two
 * that never matched even then: the function says "Space Platform" and "Arctic"
 * where the table said "Space" and "Ice".
 */
const map_era_to_tileset_key = (era: number) => {
  switch (era) {
    case 0:
      return "common.tileset_badlands";
    case 1:
      return "common.tileset_space";
    case 2:
      return "common.tileset_installation";
    case 3:
      return "common.tileset_ashworld";
    case 4:
      return "common.tileset_jungle";
    case 5:
      return "common.tileset_desert";
    case 6:
      return "common.tileset_ice";
    case 7:
      return "common.tileset_twilight";
    default:
      return "common.tileset_badlands";
  }
};

export {
  unit_id_to_name,
  map_era_to_tileset_key,
  map_ver_to_string,
  map_player_owners_to_strings,
  map_player_side_to_strings,
};
