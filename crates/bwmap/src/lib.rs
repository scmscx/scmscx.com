mod chk;
mod chunk_name;
mod parsed_chk;
mod riff;
mod util;

#[cfg(feature = "full")]
mod trig;

#[cfg(test)]
mod test;

pub use parsed_chk::ParsedChk;
pub use riff::parse_riff;

#[cfg(feature = "full")]
pub use trig::*;
