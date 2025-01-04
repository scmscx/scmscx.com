use crate::{riff::RiffChunk, util::reinterpret_slice2};
use serde::Serialize;

// Not Required.

// This section will only be different from the MTXM section in tiles where doodads are present.

// u16[ map width * height ]: 1 integer for each map tile. Moves horizontally across the map.
// The values in TILE are normally directly generated from the ISOM section (see "'ISOM' section" above), and thus do not match that of MTXM on doodad tiles.

#[derive(Debug, Serialize)]
pub struct ChkTile {
    pub data: Vec<u16>, // PROTECTION: some map protectors make TILE sections that are not a multiple of 2 bytes long. So, need to copy them and pad with 0.
}

pub(crate) fn parse_tile(chunks: &[RiffChunk<'_>]) -> Result<ChkTile, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let sec = chunks[chunks.len() - 1].data;

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

    Ok(ChkTile { data })
}
