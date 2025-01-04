use crate::{riff::RiffChunk, util::CursorSlicer};
use serde::Serialize;

// Not Required.

// This section contains the strings used for each switch. There are 256 switches, and can't be any more or any less.

// u32[256]: One long for each switch, specifies the string number for the name of each switch. Unnamed switches will have an index of 0, and have a default switch name.

#[derive(Debug, Serialize)]
pub struct ChkSwnm<'a> {
    #[serde(skip_serializing)]
    pub switch_name_string_number: &'a [u32; 256],
}

pub(crate) fn parse_swnm<'a>(chunks: &[RiffChunk<'a>]) -> Result<ChkSwnm<'a>, anyhow::Error> {
    anyhow::ensure!(chunks.len() > 0);

    let mut slicer = CursorSlicer::new(chunks[chunks.len() - 1].data);

    Ok(ChkSwnm {
        switch_name_string_number: slicer.extract_ref()?,
    })
}
