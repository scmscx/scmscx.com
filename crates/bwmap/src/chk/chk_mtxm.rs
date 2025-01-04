use crate::{riff::RiffChunk, util::reinterpret_slice2};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be less than or equal to 131072 (0x20000) bytes (256 * 256 * 2).

// Terrain section that contains a map of the level's appearance. StarEdit disregards this section and instead uses TILE; it is only used in Starcraft.

// u16[map width * height]: one integer for each tile.
// Notes about MTXM:

// The Width/Height of the map is measured in the number of square 32x32p tiles.
// Tiles in this section are listed from left to right, top to bottom.
// The values for each integer are their respective "MegaTile" values in the scenario's tileset. If the size of this section is greater than width*height*2, the data following is ignored. If the size of this section is less, the resulting tiles that have not been defined will be null tiles.
// This section includes doodads as terrain; TILE, which is otherwise identical, doesn't. Out of the terrain sections (TILE, ISOM, and MTXM), SC only reads MTXM for the sake of not having to generate this data on-the-fly: it contains the exact representation of the level's appearance, including doodads. TILE, on the other hand, is directly tied via a tile lookup function to ISOM, and exists for the sake of not having to generate tiles from ISOM on-the-fly in StarEdit.
// If MTXM section is smaller than (map width*height), then the remaining tiles will be filled with null tiles or tiles specified by previous MTXM sections.

#[derive(Debug, Serialize)]
pub struct ChkMtxm {
    pub data: Vec<u16>, // PROTECTION: some map protectors make MTXM sections that are not a multiple of 2 bytes long. So, need to copy them and pad with 0.
}

pub(crate) fn parse_mtxm(chunks: &[RiffChunk]) -> Result<ChkMtxm, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut ret = Vec::new();

    for chunk in chunks {
        let sec = chunk.data;

        let data = if sec.len() % 2 == 0 {
            Vec::from(reinterpret_slice2::<u16>(sec)?)
        } else {
            let mut ret = if sec.len() == 1 {
                Vec::new()
            } else {
                Vec::from(reinterpret_slice2::<u16>(&sec[0..sec.len() - 1])?)
            };

            ret.push(sec[sec.len() - 1] as u16);
            ret
        };

        if data.len() > ret.len() {
            ret = data;
        } else {
            ret[..data.len()].copy_from_slice(&data[..]);
        }
    }

    Ok(ChkMtxm { data: ret })
}
