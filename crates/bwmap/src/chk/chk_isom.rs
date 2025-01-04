use crate::{riff::RiffChunk, util::reinterpret_slice2};
use serde::Serialize;

// Not Required.

// This section is required to place isometric terrain on the map. It provides data about the nature of the isometrical "diamonds" (the editing mode of StarEdit).

// u16[ (width / 2 + 1) * (height + 1) * 4 ]: 4 integers for each map rectangle tile (plus one extra tile to the right and bottom of the map), as per the RECT structure. Each rectangle border (left, top, right, bottom) is assigned an "ISOM value." These form the "diamond" pattern of the map, and each rectangle thus gets two ISOM values associated with it. StarEdit somehow hashes these (the exact algorithm is unknown) and thus produces the tile index of the rectangle (which it stores in TILE for reasons of efficiency). The index of the rectangle's right tile is that of its left tile + 16.
// This section is the only truly unknown section of the .chk format. If you're an ex-Blizzard employee or SI, please edit this section. If you have additional research, post on the forums about it and/or edit this section.

#[derive(Debug, Serialize)]
pub struct ChkIsom {
    pub data: Vec<u16>, // PROTECTION: some map protectors make ISOM sections that are not a multiple of 2 bytes long. So, need to copy them and pad with 0.
}

pub(crate) fn parse_isom(chunks: &[RiffChunk]) -> Result<ChkIsom, anyhow::Error> {
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

    Ok(ChkIsom { data })
}
