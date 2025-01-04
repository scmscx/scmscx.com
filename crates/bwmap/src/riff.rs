use crate::{chunk_name::parse_chunk_name, chunk_name::ChunkName, util::parse_slice};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiffChunk<'a> {
    pub chunk_name: ChunkName,
    pub size: u32,
    pub offset: usize,
    #[serde(skip)]
    pub data: &'a [u8],
}

#[instrument(level = "trace", skip_all)]
pub fn parse_riff(chk: &[u8]) -> Vec<RiffChunk> {
    let mut offset = 0;
    let mut ret = Vec::new();

    while offset + 8 < chk.len() {
        let chunk_name: ChunkName = parse_chunk_name(&chk[offset..offset + 4]);
        let size: u32 = u32::from_le_bytes(parse_slice(&chk[offset + 4..offset + 8]));

        let chunk_data_start_offset = offset + 8;
        let chunk_data_end_offset =
            ((chunk_data_start_offset as u64 + size as u64) % (u32::MAX as u64 + 1)) as usize;
        offset = chunk_data_end_offset;

        if chunk_data_end_offset < chunk_data_start_offset {
            // negative sized chunks are ignored.
            continue;
        }

        if chunk_data_end_offset <= chk.len() {
            let data = &chk[chunk_data_start_offset..chunk_data_end_offset];
            ret.push(RiffChunk {
                chunk_name,
                size,
                offset,
                data,
            });
        }
    }

    ret
}

#[instrument(level = "trace", skip_all)]
pub fn validate_and_group_riff_chunks<'a>(
    chk: &[RiffChunk<'a>],
) -> HashMap<ChunkName, Vec<RiffChunk<'a>>> {
    let validating_iterator = chk.iter().filter(|chunk| match chunk.chunk_name {
        ChunkName::VER => chunk.size == 2,
        ChunkName::VCOD => true, // Not sure how to validate this exactly but vcod isn't really read anyway.
        ChunkName::OWNR => chunk.size == 12,
        ChunkName::ERA => chunk.size == 2,
        ChunkName::DIM => chunk.size == 4,
        ChunkName::SIDE => chunk.size == 12,
        ChunkName::MTXM => chunk.size <= (256 * 256 * 2),
        ChunkName::PUNI => chunk.size == 5700,
        ChunkName::UPGR => chunk.size == 1748,
        ChunkName::PTEC => chunk.size == 912,
        ChunkName::UNIT => chunk.size % 36 == 0,
        ChunkName::THG2 => chunk.size % 10 == 0,
        ChunkName::MASK => true, // this section will always validate?
        ChunkName::STR => chunk.size >= 1,
        ChunkName::STRx => chunk.size >= 1, // assumed
        ChunkName::UPRP => chunk.size >= 1280,
        ChunkName::MRGN => chunk.size == 1280 || chunk.size == 5100,
        ChunkName::TRIG => chunk.size % 2400 == 0,
        ChunkName::MBRF => chunk.size % 2400 == 0,
        ChunkName::SPRP => chunk.size == 4,
        ChunkName::FORC => chunk.size <= 20,
        ChunkName::UNIS => chunk.size == 4048,
        ChunkName::UPGS => chunk.size == 598,
        ChunkName::TECS => chunk.size == 216,
        ChunkName::COLR => chunk.size == 8,
        ChunkName::CRGB => chunk.size == 32,
        ChunkName::PUPx => chunk.size == 2318,
        ChunkName::PTEx => chunk.size == 1672,
        ChunkName::UNIx => chunk.size == 4168,
        ChunkName::UPGx => chunk.size == 794,
        ChunkName::TECx => chunk.size == 396,
        ChunkName::UNKNOWN(_) => false,
        _ => true,
    });

    let mut ret = HashMap::new();

    for riff_chunk in validating_iterator {
        ret.entry(riff_chunk.chunk_name.clone())
            .or_insert(Vec::new())
            .push(riff_chunk.clone());
    }

    ret
}

#[cfg(test)]
mod test {
    use crate::{chunk_name::ChunkName, riff::parse_riff, test::get_all_test_chks};
    use futures::{pin_mut, TryStreamExt};

    #[tokio::test]
    async fn test_parse_riff() {
        let stream = get_all_test_chks();

        pin_mut!(stream);

        while let Some(chk) = stream.try_next().await.unwrap() {
            let riff_chunks = parse_riff(&chk);

            assert!(riff_chunks.len() > 0);
            assert!(riff_chunks
                .iter()
                .any(|x| std::mem::discriminant(&x.chunk_name)
                    == std::mem::discriminant(&ChunkName::VER)));
        }
    }
}
