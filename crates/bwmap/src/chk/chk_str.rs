use crate::riff::RiffChunk;
use serde::Serialize;

// Required for all versions and all game types.
// Validation: Must be at least 1 byte.

// This section contains all the strings in the map.

// u16: Number of strings in the section (Default: 1024)
// u16[Number of strings]: 1 integer for each string specifying the offset (the spot where the string starts in the section from the start of it).
// Strings: After the offsets, this is where every string in the map goes, one after another. Each one is terminated by a null character.
// This section can contain more or less then 1024 string offsests and will work in Starcraft.
// By default the first byte in Strings is a NUL character, and all unused offsets in the STR section point to this NUL character. Note that STR sections can be stacked in a smiliar fashion as MTXM. The exact mechanisms of this are uncertain.

#[derive(Debug, Serialize)]
pub struct ChkStr<'a> {
    pub number_of_strings: Option<&'a u16>,
    pub string_offsets: &'a [u16],
    pub strings: &'a [u8],
}

#[derive(Debug, Serialize)]
pub struct ChkStr2 {
    pub string_data: Vec<u8>,
}

pub(crate) fn parse_str(chunks: &[RiffChunk]) -> Result<ChkStr2, anyhow::Error> {
    let mut data = Vec::new();

    for chunk in chunks {
        if chunk.data.len() >= data.len() {
            data.resize(chunk.data.len(), 0);
        }

        data.as_mut_slice()[0..chunk.data.len()].copy_from_slice(chunk.data);
    }

    Ok(ChkStr2 { string_data: data })
}

// let
// let mut strings = Vec::new();

// for string_number in 0..number_of_strings as usize {
//     if string_number >= string_offsets.len() {
//         break; // PROTECTION: number_of_strings is larger than the number of offsets that the STR section could hold. There are no more strings after this so break out of the loop.
//     }

//     let offset = string_offsets[string_number] as usize;

//     if offset > sec.len() {
//         strings.push(vec![0]); // PROTECTION: Invalid string offset that points outside of STR section
//         continue;
//     }

//     let mut len_string = 0;
//     for &c in &sec[offset..] {
//         if c == 0u8 {
//             break;
//         }

//         len_string += 1;
//     }

//     strings.push(Vec::from(&sec[offset..offset + len_string]));
// }
