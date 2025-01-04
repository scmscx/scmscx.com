use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be equal to size of the VCOD data in the EXE resource. Must result in valid checksum.

// This section has a verification code to make sure this is actually a CHK file.

// u32[256]: Seed values.
// u8[16]: Operation codes to produce the resulting hash:
// 00 - XOR's the current hash with the total values of OWNR, SIDE * 256, FORC * 65536
// 01 - Adds to the current hash with the total values of OWNR, SIDE * 256, FORC * 65536
// 02 - Subtracts from the current hash with the total values of OWNR, SIDE * 256, FORC * 65536
// 03 - XOR's the current hash with the VCOD table at index OWNR, SIDE, FORC, and 0
// 04 - same as 0x03
// 05 - same as 0x03
// 06 - Complex function consisting of ORs and shifts
// 07 - Inverted version of 0x06, switching direction of all shifts
// 08+ - Do nothing
// Opcodes and seeds do not have to match those in EXE resource, as long as the same value is produced.
// The only indexes in the seed table that are referenced are those that correspond individual values of OWNR, SIDE, FORC player assignments, and index 0.

#[derive(Debug, Serialize)]
pub struct ChkVcod<'a> {
    #[serde(skip_serializing)]
    pub seed_values: &'a [u32; 256],
    pub hash: &'a [u8; 16],
}

pub(crate) fn parse_vcod<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkVcod<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkVcod {
        seed_values: slicer.extract_ref()?,
        hash: slicer.extract_ref()?,
    })
}
