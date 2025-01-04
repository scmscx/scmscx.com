const unit_id_to_name = (unitId: number) => {
    switch (unitId) {
      case 0:
        return "Terran Marine";
      case 1:
        return "Terran Ghost";
      case 2:
        return "Terran Vulture";
      case 3:
        return "Terran Goliath";
      case 4:
        return "Goliath Turret";
      case 5:
        return "Terran Siege Tank (Tank Mode)";
      case 6:
        return "Tank Turret(Tank Mode)";
      case 7:
        return "Terran SCV";
      case 8:
        return "Terran Wraith";
      case 9:
        return "Terran Science Vessel";
      case 10:
        return "Gui Montag (Firebat)";
      case 11:
        return "Terran Dropship";
      case 12:
        return "Terran Battlecruiser";
      case 13:
        return "Vulture Spider Mine";
      case 14:
        return "Nuclear Missile";
      case 15:
        return "Terran Civilian";
      case 16:
        return "Sarah Kerrigan (Ghost)";
      case 17:
        return "Alan Schezar (Goliath)";
      case 18:
        return "Alan Schezar Turret";
      case 19:
        return "Jim Raynor (Vulture)";
      case 20:
        return "Jim Raynor (Marine)";
      case 21:
        return "Tom Kazansky (Wraith)";
      case 22:
        return "Magellan (Science Vessel)";
      case 23:
        return "Edmund Duke (Siege Tank)";
      case 24:
        return "Edmund Duke Turret";
      case 25:
        return "Edmund Duke (Siege Mode)";
      case 26:
        return "Edmund Duke Turret";
      case 27:
        return "Arcturus Mengsk (Battlecruiser)";
      case 28:
        return "Hyperion (Battlecruiser)";
      case 29:
        return "Norad II (Battlecruiser)";
      case 30:
        return "Terran Siege Tank (Siege Mode)";
      case 31:
        return "Tank Turret (Siege Mode)";
      case 32:
        return "Firebat";
      case 33:
        return "Scanner Sweep";
      case 34:
        return "Terran Medic";
      case 35:
        return "Zerg Larva";
      case 36:
        return "Zerg Egg";
      case 37:
        return "Zerg Zergling";
      case 38:
        return "Zerg Hydralisk";
      case 39:
        return "Zerg Ultralisk";
      case 40:
        return "Zerg Broodling";
      case 41:
        return "Zerg Drone";
      case 42:
        return "Zerg Overlord";
      case 43:
        return "Zerg Mutalisk";
      case 44:
        return "Zerg Guardian";
      case 45:
        return "Zerg Queen";
      case 46:
        return "Zerg Defiler";
      case 47:
        return "Zerg Scourge";
      case 48:
        return "Torrarsque (Ultralisk)";
      case 49:
        return "Matriarch (Queen)";
      case 50:
        return "Infested Terran";
      case 51:
        return "Infested Kerrigan";
      case 52:
        return "Unclean One (Defiler)";
      case 53:
        return "Hunter Killer (Hydralisk)";
      case 54:
        return "Devouring One (Zergling)";
      case 55:
        return "Kukulza (Mutalisk)";
      case 56:
        return "Kukulza (Guardian)";
      case 57:
        return "Yggdrasill (Overlord)";
      case 58:
        return "Terran Valkyrie Frigate";
      case 59:
        return "Mutalisk/Guardian Cocoon";
      case 60:
        return "Protoss Corsair";
      case 61:
        return "Protoss Dark Templar(Unit)";
      case 62:
        return "Zerg Devourer";
      case 63:
        return "Protoss Dark Archon";
      case 64:
        return "Protoss Probe";
      case 65:
        return "Protoss Zealot";
      case 66:
        return "Protoss Dragoon";
      case 67:
        return "Protoss High Templar";
      case 68:
        return "Protoss Archon";
      case 69:
        return "Protoss Shuttle";
      case 70:
        return "Protoss Scout";
      case 71:
        return "Protoss Arbiter";
      case 72:
        return "Protoss Carrier";
      case 73:
        return "Protoss Interceptor";
      case 74:
        return "Dark Templar(Hero)";
      case 75:
        return "Zeratul (Dark Templar)";
      case 76:
        return "Tassadar/Zeratul (Archon)";
      case 77:
        return "Fenix (Zealot)";
      case 78:
        return "Fenix (Dragoon)";
      case 79:
        return "Tassadar (Templar)";
      case 80:
        return "Mojo (Scout)";
      case 81:
        return "Warbringer (Reaver)";
      case 82:
        return "Gantrithor (Carrier)";
      case 83:
        return "Protoss Reaver";
      case 84:
        return "Protoss Observer";
      case 85:
        return "Protoss Scarab";
      case 86:
        return "Danimoth (Arbiter)";
      case 87:
        return "Aldaris (Templar)";
      case 88:
        return "Artanis (Scout)";
      case 89:
        return "Rhynadon (Badlands Critter)";
      case 90:
        return "Bengalaas (Jungle Critter)";
      case 91:
        return "Unused - Was Cargo Ship";
      case 92:
        return "Unused - Was Mercenary Gunship";
      case 93:
        return "Scantid (Desert Critter)";
      case 94:
        return "Kakaru (Twilight Critter)";
      case 95:
        return "Ragnasaur (Ashworld Critter)";
      case 96:
        return "Ursadon (Ice World Critter)";
      case 97:
        return "Lurker Egg";
      case 98:
        return "Raszagal (Corsair)";
      case 99:
        return "Samir Duran (Ghost)";
      case 100:
        return "Alexei Stukov (Ghost)";
      case 101:
        return "Map Revealer";
      case 102:
        return "Gerard DuGalle (Battlecruiser)";
      case 103:
        return "Zerg Lurker";
      case 104:
        return "Infested Duran";
      case 105:
        return "Disruption Web";
      case 106:
        return "Terran Command Center";
      case 107:
        return "Terran Comsat Station";
      case 108:
        return "Terran Nuclear Silo";
      case 109:
        return "Terran Supply Depot";
      case 110:
        return "Terran Refinery";
      case 111:
        return "Terran Barracks";
      case 112:
        return "Terran Academy";
      case 113:
        return "Terran Factory";
      case 114:
        return "Terran Starport";
      case 115:
        return "Terran Control Tower";
      case 116:
        return "Terran Science Facility";
      case 117:
        return "Terran Covert Ops";
      case 118:
        return "Terran Physics Lab";
      case 119:
        return "Unused - Was Starbase?";
      case 120:
        return "Terran Machine Shop";
      case 121:
        return "Unused - Was Repair Bay?";
      case 122:
        return "Terran Engineering Bay";
      case 123:
        return "Terran Armory";
      case 124:
        return "Terran Missile Turret";
      case 125:
        return "Terran Bunker";
      case 126:
        return "Norad II";
      case 127:
        return "Ion Cannon";
      case 128:
        return "Uraj Crystal";
      case 129:
        return "Khalis Crystal";
      case 130:
        return "Infested Command Center";
      case 131:
        return "Zerg Hatchery";
      case 132:
        return "Zerg Lair";
      case 133:
        return "Zerg Hive";
      case 134:
        return "Zerg Nydus Canal";
      case 135:
        return "Zerg Hydralisk Den";
      case 136:
        return "Zerg Defiler Mound";
      case 137:
        return "Zerg Greater Spire";
      case 138:
        return "Zerg Queen's Nest";
      case 139:
        return "Zerg Evolution Chamber";
      case 140:
        return "Zerg Ultralisk Cavern";
      case 141:
        return "Zerg Spire";
      case 142:
        return "Zerg Spawning Pool";
      case 143:
        return "Zerg Creep Colony";
      case 144:
        return "Zerg Spore Colony";
      case 145:
        return "Unused Zerg Building";
      case 146:
        return "Zerg Sunken Colony";
      case 147:
        return "Zerg Overmind (With Shell)";
      case 148:
        return "Zerg Overmind";
      case 149:
        return "Zerg Extractor";
      case 150:
        return "Mature Chrysalis";
      case 151:
        return "Zerg Cerebrate";
      case 152:
        return "Zerg Cerebrate Daggoth";
      case 153:
        return "Unused Zerg Building 5";
      case 154:
        return "Protoss Nexus";
      case 155:
        return "Protoss Robotics Facility";
      case 156:
        return "Protoss Pylon";
      case 157:
        return "Protoss Assimilator";
      case 158:
        return "Unused Protoss Building";
      case 159:
        return "Protoss Observatory";
      case 160:
        return "Protoss Gateway";
      case 161:
        return "Unused Protoss Building";
      case 162:
        return "Protoss Photon Cannon";
      case 163:
        return "Protoss Citadel of Adun";
      case 164:
        return "Protoss Cybernetics Core";
      case 165:
        return "Protoss Templar Archives";
      case 166:
        return "Protoss Forge";
      case 167:
        return "Protoss Stargate";
      case 168:
        return "Stasis Cell/Prison";
      case 169:
        return "Protoss Fleet Beacon";
      case 170:
        return "Protoss Arbiter Tribunal";
      case 171:
        return "Protoss Robotics Support Bay";
      case 172:
        return "Protoss Shield Battery";
      case 173:
        return "Khaydarin Crystal Formation";
      case 174:
        return "Protoss Temple";
      case 175:
        return "Xel'Naga Temple";
      case 176:
        return "Mineral Field (Type 1)";
      case 177:
        return "Mineral Field (Type 2)";
      case 178:
        return "Mineral Field (Type 3)";
      case 179:
        return "Cave";
      case 180:
        return "Cave-in";
      case 181:
        return "Cantina";
      case 182:
        return "Mining Platform";
      case 183:
        return "Independant Command Center";
      case 184:
        return "Independant Starport";
      case 185:
        return "Jump Gate";
      case 186:
        return "Ruins";
      case 187:
        return "Kyadarin Crystal Formation";
      case 188:
        return "Vespene Geyser";
      case 189:
        return "Warp Gate";
      case 190:
        return "PSI Disruptor";
      case 191:
        return "Zerg Marker";
      case 192:
        return "Terran Marker";
      case 193:
        return "Protoss Marker";
      case 194:
        return "Zerg Beacon";
      case 195:
        return "Terran Beacon";
      case 196:
        return "Protoss Beacon";
      case 197:
        return "Zerg Flag Beacon";
      case 198:
        return "Terran Flag Beacon";
      case 199:
        return "Protoss Flag Beacon";
      case 200:
        return "Power Generator";
      case 201:
        return "Overmind Cocoon";
      case 202:
        return "Dark Swarm";
      case 203:
        return "Floor Missile Trap";
      case 204:
        return "Floor Hatch";
      case 205:
        return "Left Upper Level Door";
      case 206:
        return "Right Upper Level Door";
      case 207:
        return "Left Pit Door";
      case 208:
        return "Right Pit Door";
      case 209:
        return "Floor Gun Trap";
      case 210:
        return "Left Wall Missile Trap";
      case 211:
        return "Left Wall Flame Trap";
      case 212:
        return "Right Wall Missile Trap";
      case 213:
        return "Right Wall Flame Trap";
      case 214:
        return "Start Location";
      case 215:
        return "Flag";
      case 216:
        return "Young Chrysalis";
      case 217:
        return "Psi Emitter";
      case 218:
        return "Data Disc";
      case 219:
        return "Khaydarin Crystal";
      case 220:
        return "Mineral Cluster Type 1";
      case 221:
        return "Mineral Cluster Type 2";
      case 222:
        return "Protoss Vespene Gas Orb Type 1";
      case 223:
        return "Protoss Vespene Gas Orb Type 2";
      case 224:
        return "Zerg Vespene Gas Sac Type 1";
      case 225:
        return "Zerg Vespene Gas Sac Type 2";
      case 226:
        return "Terran Vespene Gas Tank Type 1";
      case 227:
        return "Terran Vespene Gas Tank Type 2";
  
      default:
        return "Unknown unit ID";
    }
  };

  const map_era_to_tileset = (era: number) => {
    switch (era) {
      case 0:
        return "Badlands";
      case 1:
        return "Space Platform";
      case 2:
        return "Installation";
      case 3:
        return "Ashworld";
      case 4:
        return "Jungle";
      case 5:
        return "Desert";
      case 6:
        return "Arctic";
      case 7:
        return "Twilight";
      default:
        return "<InvalidEra>";
    }
  };
  
  const map_ver_to_string = (ver: number) => {
    switch (ver) {
      case 206:
        return "Remastered 1.21";
      case 205:
        return "Broodwar 1.04+";
      case 64:
        return "Starcraft Remastered 1.21 hybrid";
      case 63:
        return "Starcraft 1.04+ hybrid";
  
      case 59:
        return "Starcraft 1.00";
  
      case 61:
      case 75:
      case 201:
      case 203:
        return "BroodWar Internal";
  
      case 47:
        return "StarCraft Beta";
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
        return "Warcraft II";
  
      default:
        return "Unknown Version";
    }
  };
  
  const map_player_owners_to_strings = (ownr: number) => {
    switch (ownr) {
      case 0:
        return "Inactive";
      case 1:
        return "Computer (game)";
      case 2:
        return "Occupied by Human Player";
      case 3:
        return "Rescue Passive";
      case 4:
        return "Unused";
      case 5:
        return "Computer";
      case 6:
        return "Open";
      case 7:
        return "Neutral";
      case 8:
        return "Closed slot";
      default:
        return "<InvalidPlayerOwner>";
    }
  };
  
  const map_player_side_to_strings = (ownr: number) => {
    switch (ownr) {
      case 0:
        return "Zerg";
      case 1:
        return "Terran";
      case 2:
        return "Protoss";
      case 3:
        return "Invalid (Independent)";
      case 4:
        return "Invalid (Neutral)";
      case 5:
        return "User Select";
      case 6:
        return "Random";
      case 7:
        return "Inactive";
      default:
        return "<InvalidPlayerSide>";
    }
  };


export {unit_id_to_name, map_era_to_tileset, map_ver_to_string, map_player_owners_to_strings, map_player_side_to_strings};
