use crate::{
    chk::{
        chk_colr::{parse_colr, ChkColr},
        chk_crgb::{parse_crgb, ChkCrgb},
        chk_dd2::{parse_dd2, ChkDd2},
        chk_dim::{parse_dim, ChkDim},
        chk_era::{parse_era, ChkEra},
        chk_forc::{parse_forc, ChkForc},
        chk_iown::{parse_iown, ChkIown},
        chk_isom::{parse_isom, ChkIsom},
        chk_ive2::{parse_ive2, ChkIve2},
        chk_iver::{parse_iver, ChkIver},
        chk_mask::{parse_mask, ChkMask},
        chk_mbrf::{parse_mbrf, ChkMbrf},
        chk_mrgn::{parse_mrgn, ChkMrgn},
        chk_mtxm::{parse_mtxm, ChkMtxm},
        chk_ownr::{parse_ownr, ChkOwnr},
        chk_ptec::{parse_ptec, ChkPtec},
        chk_ptex::{parse_ptex, ChkPtex},
        chk_puni::{parse_puni, ChkPuni},
        chk_pupx::{parse_pupx, ChkPupx},
        chk_side::{parse_side, ChkSide},
        chk_sprp::{parse_sprp, ChkSprp},
        chk_str::{parse_str, ChkStr2},
        chk_strx::{parse_strx, ChkStrx2},
        chk_swnm::{parse_swnm, ChkSwnm},
        chk_tecs::{parse_tecs, ChkTecs},
        chk_tecx::{parse_tecx, ChkTecx},
        chk_thg2::{parse_thg2, ChkThg2},
        chk_tile::{parse_tile, ChkTile},
        chk_trig::{parse_trig, ChkTrig},
        chk_type::{parse_type, ChkType},
        chk_unis::{parse_unis, ChkUnis},
        chk_unit::{parse_unit, ChkUnit},
        chk_unix::{parse_unix, ChkUnix},
        chk_upgr::{parse_upgr, ChkUpgr},
        chk_upgs::{parse_upgs, ChkUpgs},
        chk_upgx::{parse_upgx, ChkUpgx},
        chk_uprp::{parse_uprp, ChkUprp},
        chk_upus::{parse_upus, ChkUpus},
        chk_vcod::{parse_vcod, ChkVcod},
        chk_ver::{parse_ver, ChkVer},
        chk_wav::{parse_wav, ChkWav},
    },
    chunk_name::ChunkName,
    riff::{parse_riff, validate_and_group_riff_chunks},
};
use anyhow::Result;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use tracing::instrument;

#[derive(Debug)]
pub struct ParsedChk<'a> {
    pub colr: Result<ChkColr>,
    pub crgb: Result<ChkCrgb>,
    pub dd2: Result<ChkDd2>,
    pub dim: Result<ChkDim<'a>>,
    pub era: Result<ChkEra<'a>>,
    pub forc: Result<ChkForc>,
    pub iown: Result<ChkIown>,
    pub isom: Result<ChkIsom>,
    pub ive2: Result<ChkIve2<'a>>,
    pub iver: Result<ChkIver<'a>>,
    pub mask: Result<ChkMask<'a>>,
    pub mbrf: Result<ChkMbrf>,
    pub mrgn: Result<ChkMrgn<'a>>,
    pub mtxm: Result<ChkMtxm>,
    pub ownr: Result<ChkOwnr>,
    pub ptec: Result<ChkPtec<'a>>,
    pub ptex: Result<ChkPtex<'a>>,
    pub puni: Result<ChkPuni<'a>>,
    pub pupx: Result<ChkPupx<'a>>,
    pub side: Result<ChkSide>,
    pub sprp: Result<ChkSprp<'a>>,
    pub str: Result<ChkStr2>,
    pub strx: Result<ChkStrx2>,
    pub swnm: Result<ChkSwnm<'a>>,
    pub tecs: Result<ChkTecs<'a>>,
    pub tecx: Result<ChkTecx<'a>>,
    pub thg2: Result<ChkThg2>,
    pub tile: Result<ChkTile>,
    pub trig: Result<ChkTrig>,
    pub type_: Result<ChkType<'a>>,
    pub unis: Result<ChkUnis<'a>>,
    pub unit: Result<ChkUnit>,
    pub unix: Result<ChkUnix>,
    pub upgr: Result<ChkUpgr<'a>>,
    pub upgs: Result<ChkUpgs<'a>>,
    pub upgx: Result<ChkUpgx<'a>>,
    pub uprp: Result<ChkUprp<'a>>,
    pub upus: Result<ChkUpus<'a>>,
    pub vcod: Result<ChkVcod<'a>>,
    pub ver: Result<ChkVer<'a>>,
    pub wav: Result<ChkWav<'a>>,
}

impl Serialize for ParsedChk<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        S::Error: std::error::Error,
    {
        let mut map = serializer.serialize_map(None)?;

        let _ = self.colr.as_ref().map(|x| map.serialize_entry("COLR", &x));
        let _ = self.crgb.as_ref().map(|x| map.serialize_entry("CRGB", &x));
        let _ = self.dd2.as_ref().map(|x| map.serialize_entry("DD2", &x));
        let _ = self.dim.as_ref().map(|x| map.serialize_entry("DIM", &x));
        let _ = self.era.as_ref().map(|x| map.serialize_entry("ERA", &x));
        let _ = self.forc.as_ref().map(|x| map.serialize_entry("FORC", &x));
        let _ = self.iown.as_ref().map(|x| map.serialize_entry("IOWN", &x));
        // let _ = self.isom.as_ref().map(|x| map.serialize_entry("ISOM", &x));
        let _ = self.ive2.as_ref().map(|x| map.serialize_entry("IVE2", &x));
        let _ = self.iver.as_ref().map(|x| map.serialize_entry("IVER", &x));
        let _ = self.mask.as_ref().map(|x| map.serialize_entry("MASK", &x));
        let _ = self.mbrf.as_ref().map(|x| map.serialize_entry("MBRF", &x));
        let _ = self.mrgn.as_ref().map(|x| map.serialize_entry("MRGN", &x));
        // let _ = self.mtxm.as_ref().map(|x| map.serialize_entry("MTXM", &x));
        let _ = self.ownr.as_ref().map(|x| map.serialize_entry("OWNR", &x));
        let _ = self.ptec.as_ref().map(|x| map.serialize_entry("PTEC", &x));
        let _ = self.ptex.as_ref().map(|x| map.serialize_entry("PTEx", &x));
        let _ = self.puni.as_ref().map(|x| map.serialize_entry("PUNI", &x));
        let _ = self.pupx.as_ref().map(|x| map.serialize_entry("PUPx", &x));
        let _ = self.side.as_ref().map(|x| map.serialize_entry("SIDE", &x));
        let _ = self.sprp.as_ref().map(|x| map.serialize_entry("SPRP", &x));
        let _ = self.str.as_ref().map(|x| map.serialize_entry("STR", &x));
        let _ = self.strx.as_ref().map(|x| map.serialize_entry("STRx", &x));
        let _ = self.swnm.as_ref().map(|x| map.serialize_entry("SWNM", &x));
        let _ = self.tecs.as_ref().map(|x| map.serialize_entry("TECS", &x));
        let _ = self.tecx.as_ref().map(|x| map.serialize_entry("TECx", &x));
        let _ = self.thg2.as_ref().map(|x| map.serialize_entry("THG2", &x));
        // let _ = self.tile.as_ref().map(|x| map.serialize_entry("TILE", &x));
        // let _ = self.trig.as_ref().map(|x| map.serialize_entry("TRIG", &x));
        let _ = self.type_.as_ref().map(|x| map.serialize_entry("TYPE", &x));
        let _ = self.unis.as_ref().map(|x| map.serialize_entry("UNIS", &x));
        let _ = self.unit.as_ref().map(|x| map.serialize_entry("UNIT", &x));
        let _ = self.unix.as_ref().map(|x| map.serialize_entry("UNIx", &x));
        let _ = self.upgr.as_ref().map(|x| map.serialize_entry("UPGR", &x));
        let _ = self.upgs.as_ref().map(|x| map.serialize_entry("UPGS", &x));
        let _ = self.upgx.as_ref().map(|x| map.serialize_entry("UPGx", &x));
        let _ = self.uprp.as_ref().map(|x| map.serialize_entry("UPRP", &x));
        let _ = self.upus.as_ref().map(|x| map.serialize_entry("UPUS", &x));
        let _ = self.vcod.as_ref().map(|x| map.serialize_entry("VCOD", &x));
        let _ = self.ver.as_ref().map(|x| map.serialize_entry("VER", &x));
        let _ = self.wav.as_ref().map(|x| map.serialize_entry("WAV", &x));

        map.end()
    }
}

impl<'a> ParsedChk<'a> {
    #[instrument(level = "trace", skip_all)]
    pub fn from_bytes(chk: &'a [u8]) -> ParsedChk<'a> {
        let riff_chunks = parse_riff(chk);
        let riff_chunks = validate_and_group_riff_chunks(riff_chunks.as_slice());

        #[rustfmt::skip]
        let ret = ParsedChk {
            colr: riff_chunks.get(&ChunkName::COLR).map(|x| parse_colr(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            crgb: riff_chunks.get(&ChunkName::CRGB).map(|x| parse_crgb(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            dd2:  riff_chunks.get(&ChunkName::DD2 ).map(|x| parse_dd2 (x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            dim:  riff_chunks.get(&ChunkName::DIM ).map(|x| parse_dim (x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            era:  riff_chunks.get(&ChunkName::ERA ).map(|x| parse_era (x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            forc: riff_chunks.get(&ChunkName::FORC).map(|x| parse_forc(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            iown: riff_chunks.get(&ChunkName::IOWN).map(|x| parse_iown(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            isom: riff_chunks.get(&ChunkName::ISOM).map(|x| parse_isom(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            ive2: riff_chunks.get(&ChunkName::IVE2).map(|x| parse_ive2(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            iver: riff_chunks.get(&ChunkName::IVER).map(|x| parse_iver(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            mask: riff_chunks.get(&ChunkName::MASK).map(|x| parse_mask(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            mbrf: riff_chunks.get(&ChunkName::MBRF).map(|x| parse_mbrf(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            mrgn: riff_chunks.get(&ChunkName::MRGN).map(|x| parse_mrgn(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            mtxm: riff_chunks.get(&ChunkName::MTXM).map(|x| parse_mtxm(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            ownr: riff_chunks.get(&ChunkName::OWNR).map(|x| parse_ownr(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            ptec: riff_chunks.get(&ChunkName::PTEC).map(|x| parse_ptec(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            ptex: riff_chunks.get(&ChunkName::PTEx).map(|x| parse_ptex(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            puni: riff_chunks.get(&ChunkName::PUNI).map(|x| parse_puni(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            pupx: riff_chunks.get(&ChunkName::PUPx).map(|x| parse_pupx(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            side: riff_chunks.get(&ChunkName::SIDE).map(|x| parse_side(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            sprp: riff_chunks.get(&ChunkName::SPRP).map(|x| parse_sprp(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            str:  riff_chunks.get(&ChunkName::STR ).map(|x| parse_str (x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            strx: riff_chunks.get(&ChunkName::STRx).map(|x| parse_strx(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            swnm: riff_chunks.get(&ChunkName::SWNM).map(|x| parse_swnm(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            tecs: riff_chunks.get(&ChunkName::TECS).map(|x| parse_tecs(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            tecx: riff_chunks.get(&ChunkName::TECx).map(|x| parse_tecx(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            thg2: riff_chunks.get(&ChunkName::THG2).map(|x| parse_thg2(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            tile: riff_chunks.get(&ChunkName::TILE).map(|x| parse_tile(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            trig: riff_chunks.get(&ChunkName::TRIG).map(|x| parse_trig(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            type_:riff_chunks.get(&ChunkName::TYPE).map(|x| parse_type(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            unis: riff_chunks.get(&ChunkName::UNIS).map(|x| parse_unis(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            unit: riff_chunks.get(&ChunkName::UNIT).map(|x| parse_unit(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            unix: riff_chunks.get(&ChunkName::UNIx).map(|x| parse_unix(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            upgr: riff_chunks.get(&ChunkName::UPGR).map(|x| parse_upgr(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            upgs: riff_chunks.get(&ChunkName::UPGS).map(|x| parse_upgs(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            upgx: riff_chunks.get(&ChunkName::UPGx).map(|x| parse_upgx(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            uprp: riff_chunks.get(&ChunkName::UPRP).map(|x| parse_uprp(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            upus: riff_chunks.get(&ChunkName::UPUS).map(|x| parse_upus(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            vcod: riff_chunks.get(&ChunkName::VCOD).map(|x| parse_vcod(x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            ver:  riff_chunks.get(&ChunkName::VER ).map(|x| parse_ver (x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
            wav:  riff_chunks.get(&ChunkName::WAV ).map(|x| parse_wav (x.as_slice())).unwrap_or(Err(anyhow::anyhow!("Not Found"))),
        };

        ret
    }

    #[cfg(feature = "full")]
    #[instrument(level = "trace", skip(self))]
    pub fn get_string(
        &self,
        // encoding_order: &Vec<&'static encoding_rs::Encoding>,
        index: usize,
    ) -> Result<String> {
        use crate::util::parse_null_terminated_bytestring_unsigned;

        if index == 0 {
            return Ok("Zero index provided to 'get_string'".to_owned());
        }

        let index = index - 1;

        let bytes = if let Ok(x) = &self.strx {
            let mut offset = 4;
            offset += index * 4;

            if offset + 4 >= x.string_data.len() {
                return Ok(format!(
                    "Out of bounds access STRx offset table. index: {index}, offset+4: {}, string_data.len(): {}",
                    offset + 4,
                    x.string_data.len()
                ));
            }

            let str_offset: usize =
                u32::from_le_bytes(x.string_data[offset..offset + 4].try_into()?).try_into()?;

            if str_offset >= x.string_data.len() {
                return Ok(format!(
                    "Out of bounds access STRx. index: {index}, offset: {offset}, string_data.len(): {}, str_offset: {str_offset}",
                    x.string_data.len(),
                ));
            }

            parse_null_terminated_bytestring_unsigned(&x.string_data[str_offset..])
        } else if let Ok(x) = &self.str {
            let mut offset = 2;
            offset += index * 2;

            if offset + 2 >= x.string_data.len() {
                return Ok(format!(
                    "Out of bounds access STR offset table. index: {index}, offset+4: {}, string_data.len(): {}",
                    offset + 2,
                    x.string_data.len()
                ));
            }

            let str_offset: usize =
                u16::from_le_bytes(x.string_data[offset..offset + 2].try_into()?).try_into()?;

            if str_offset >= x.string_data.len() {
                return Ok(format!(
                    "Out of bounds access STR. index: {index}, offset: {offset}, string_data.len(): {}, str_offset: {str_offset}",
                    x.string_data.len(),
                ));
            }

            parse_null_terminated_bytestring_unsigned(&x.string_data[str_offset..])
        } else {
            anyhow::bail!("No STR or STRx section")
        };

        if bytes.is_empty() {
            return Ok("".to_owned());
        }

        let mut euc_kr_failed = false;
        let mut euc_kr_characters_decoded_successfully: i64 = 0;
        let euc_kr_characters_len: usize;
        let mut utf8_failed = false;
        let mut utf8_characters_decoded_successfully: i64 = 0;
        let utf8_characters_len: usize;

        let mut win1252_characters_7f_or_above: i64 = 0;

        {
            let conversion = encoding_rs::EUC_KR.decode(bytes);
            // println!("EUC_KR: {}", conversion.0);
            euc_kr_characters_len = conversion.0.chars().count();

            if conversion.2 {
                euc_kr_failed = true;
            } else {
                euc_kr_characters_decoded_successfully += conversion
                    .0
                    .chars()
                    .filter(|&c| ('가'..='힣').contains(&c))
                    .count() as i64;
            }
        }

        {
            let conversion = encoding_rs::UTF_8.decode(bytes);
            // println!("UTF-8: {}", conversion.0);
            utf8_characters_len = conversion.0.chars().count();

            if conversion.2 {
                utf8_failed = true;
            } else {
                utf8_characters_decoded_successfully +=
                    conversion.0.chars().filter(|&c| c >= '\u{7f}').count() as i64;
            }
        }

        {
            let conversion = encoding_rs::WINDOWS_1252.decode(bytes);
            // println!("WIN1252: {}", conversion.0);
            win1252_characters_7f_or_above +=
                conversion.0.chars().filter(|&c| c >= '\u{7f}').count() as i64;
        }

        let uchardet_guessed_encoding = unsafe {
            let handle = uchardet_bindings::uchardet_new();
            scopeguard::defer! {
                uchardet_bindings::uchardet_delete(handle);
            }

            // filter out color codes because it might be breaking uchardet.
            let vec: Vec<_> = bytes.iter().filter(|&&x| x >= 0x20).copied().collect();

            if uchardet_bindings::uchardet_handle_data(handle, vec.as_ptr() as *const i8, vec.len())
                != 0
            {
                panic!();
            }

            uchardet_bindings::uchardet_data_end(handle);

            let charset = std::ffi::CStr::from_ptr(uchardet_bindings::uchardet_get_charset(handle))
                .to_str()?
                .to_string();

            match charset.as_str() {
                "UTF-8" => anyhow::Ok(Some(encoding_rs::UTF_8)),
                "UHC" => anyhow::Ok(Some(encoding_rs::EUC_KR)),
                "ASCII" | "ISO-8859-7" | "ISO-8859-2" | "WINDOWS-1252" | "WINDOWS-1250"
                | "MAC-CENTRALEUROPE" | "WINDOWS-1257" | "ISO-8859-10" | "ISO-8859-1" => {
                    anyhow::Ok(Some(encoding_rs::WINDOWS_1252))
                }
                "" | "IBM852" | "VISCII" | "ISO-8859-13" | "ISO-8859-9" | "ISO-8859-3" => {
                    anyhow::Ok(None)
                }
                _ => {
                    anyhow::Ok(None)
                    //panic!("{charset}")
                }
            }
        }?;

        let (compact_enc_det_encoding_guess, compact_enc_det_encoding_guess_is_reliable) = unsafe {
            // filter out color codes because it might be breaking uchardet.
            let vec: Vec<_> = bytes.iter().filter(|&&x| x >= 0x20).copied().collect();

            let mut bytes_consumed = 0;
            let mut is_reliable = false;

            let encoding = compact_enc_det_bindings::CompactEncDet_DetectEncoding(
                vec.as_ptr() as *const i8,
                vec.len() as i32,
                std::ptr::null::<i8>(),
                std::ptr::null::<i8>(),
                std::ptr::null::<i8>(),
                compact_enc_det_bindings::Encoding_UNKNOWN_ENCODING as i32,
                compact_enc_det_bindings::Language_UNKNOWN_LANGUAGE,
                compact_enc_det_bindings::CompactEncDet_TextCorpusType_WEB_CORPUS,
                true,
                &mut bytes_consumed,
                &mut is_reliable,
            );

            // println!("is_reliable: {is_reliable}, bytes_consumed: {bytes_consumed}");

            (
                match encoding {
                    compact_enc_det_bindings::Encoding_UTF8 => Some(encoding_rs::UTF_8),
                    compact_enc_det_bindings::Encoding_KOREAN_EUC_KR => Some(encoding_rs::EUC_KR),
                    compact_enc_det_bindings::Encoding_ISO_8859_1
                    | compact_enc_det_bindings::Encoding_MSFT_CP1252
                    | compact_enc_det_bindings::Encoding_ASCII_7BIT => {
                        Some(encoding_rs::WINDOWS_1252)
                    }
                    compact_enc_det_bindings::Encoding_CHINESE_GB
                    | compact_enc_det_bindings::Encoding_CHINESE_BIG5
                    | compact_enc_det_bindings::Encoding_JAPANESE_EUC_JP => None,
                    _ => {
                        None
                        //panic!("encoding panic'd on: {encoding}")
                    }
                },
                is_reliable,
            )
        };

        let mut encoding_map = std::collections::HashMap::new();

        encoding_map.insert(encoding_rs::UTF_8, 0.0);
        encoding_map.insert(encoding_rs::EUC_KR, 0.0);
        encoding_map.insert(encoding_rs::WINDOWS_1252, 0.0);

        if let Some(uchardet_guessed_encoding) = uchardet_guessed_encoding {
            *encoding_map.get_mut(uchardet_guessed_encoding).unwrap() += 0.7;
        }

        if let Some(compact_enc_det_encoding_guess) = compact_enc_det_encoding_guess {
            *encoding_map
                .get_mut(compact_enc_det_encoding_guess)
                .unwrap() += if compact_enc_det_encoding_guess_is_reliable {
                0.7
            } else {
                0.2
            };
        }

        *encoding_map.get_mut(encoding_rs::EUC_KR).unwrap() +=
            (euc_kr_characters_decoded_successfully as f64) / (euc_kr_characters_len as f64);

        *encoding_map.get_mut(encoding_rs::WINDOWS_1252).unwrap() -=
            (win1252_characters_7f_or_above as f64) / (bytes.len() as f64);

        *encoding_map.get_mut(encoding_rs::UTF_8).unwrap() +=
            (utf8_characters_decoded_successfully as f64) / (utf8_characters_len as f64);

        // println!(
        //     "\n\
        //     euc_kr_failed: {euc_kr_failed}, \
        //     euc_kr_characters_decoded_successfully: {euc_kr_characters_decoded_successfully}, \
        //     euc_kr_characters_len: {euc_kr_characters_len}, \
        //     utf8_failed: {utf8_failed}, \
        //     utf8_characters_decoded_successfully: {utf8_characters_decoded_successfully}, \
        //     utf8_characters_len: {utf8_characters_len}, \
        //     win1252_characters_7f_or_above: {win1252_characters_7f_or_above}, \
        //     win1252_total_characters: {win1252_total_characters}, \
        //     \n\
        //     uchardet_guess: {uchardet_guessed_encoding:?}, \
        //     \n\
        //     compact_enc_det_encoding_guess: {compact_enc_det_encoding_guess:?}, \
        //     \n\
        //     encoding_map: {encoding_map:?}, str: {}\n\
        //     ------------------------------------------------------------------------------------",
        //     encoding_rs::WINDOWS_1252.decode(bytes).0
        // );

        if euc_kr_failed {
            encoding_map.remove(encoding_rs::EUC_KR);
        }

        if utf8_failed {
            encoding_map.remove(encoding_rs::UTF_8);
        }

        let mut encodings: Vec<_> = encoding_map.drain().collect();

        encodings.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());

        Ok(encodings[0].0.decode(bytes).0.to_string())

        // TODO: Implement voting idea.
        // decoding failure = complete veto.
        // udetchar can vote with some weight.
        // number of chars successfully decoded specifically in that range can also vote as some weight.
        // Other strings in the map can also vote with some weight but not sure how to implement that exactly.
        // Table of exceptions can also make a vote.
    }

    #[instrument(level = "trace", skip(self))]
    pub fn get_all_string_references(&self) -> Result<Vec<u32>, anyhow::Error> {
        let mut ret = Vec::new();

        if let Ok(x) = &self.sprp {
            ret.push(*x.scenario_name_string_number as u32);
            ret.push(*x.description_string_number as u32);
        }

        if let Ok(x) = &self.forc {
            for string_number in x.force_name {
                ret.push(string_number as u32);
            }
        }

        if let Ok(x) = &self.unix {
            for i in 0..x.string_number.len() {
                if x.config[i] == 0 {
                    ret.push(x.string_number[i] as u32);
                }
            }
        } else if let Ok(x) = &self.unis {
            for i in 0..x.string_number.len() {
                if x.config[i] == 0 {
                    ret.push(x.string_number[i] as u32);
                }
            }
        }

        if let Ok(x) = &self.wav {
            for string_number in x.wav_string_number {
                ret.push(*string_number);
            }
        }

        if let Ok(x) = &self.swnm {
            for switch_name_string_number in x.switch_name_string_number {
                ret.push(*switch_name_string_number);
            }
        }

        if let Ok(x) = &self.mrgn {
            for location in x.locations {
                ret.push(location.name_string_number as u32);
            }
        }

        if let Ok(x) = &self.mbrf {
            for trigger in &x.triggers {
                for action in trigger.actions {
                    ret.push(action.string_number);
                }
            }
        }

        if let Ok(x) = &self.trig {
            for trigger in &x.triggers {
                for action in trigger.actions {
                    ret.push(action.string_number);
                }
            }
        }

        Ok(ret.into_iter().filter(|&x| x != 0).collect())
    }

    #[cfg(feature = "full")]
    #[instrument(level = "trace", skip(self))]
    pub fn get_location_name(&self, index: usize) -> Result<String> {
        if index == 0 {
            return Ok("No Location".to_owned());
        }

        if let Ok(mrgn) = &self.mrgn {
            if mrgn.locations.len() <= index {
                return Ok("Location index out of bounds".to_owned());
            }

            if mrgn.locations[index - 1].name_string_number == 0 {
                Ok(format!("Location {} has a string name index of 0", index))
            } else {
                self.get_string(mrgn.locations[index - 1].name_string_number as usize)
            }
        } else {
            Ok("No Location".to_owned())
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{
        test::{get_all_test_chks, get_chk},
        ParsedChk,
    };
    use futures::{pin_mut, TryStreamExt};

    #[tokio::test]
    async fn can_parse_all_maps() {
        let stream = get_all_test_chks();

        pin_mut!(stream);

        while let Some(chk_data) = stream.try_next().await.unwrap() {
            let parsed_chk = ParsedChk::from_bytes(chk_data.as_slice());

            assert!(parsed_chk.ver.is_ok());
            assert!(parsed_chk.vcod.is_ok());
            assert!(parsed_chk.ownr.is_ok());
            assert!(parsed_chk.era.is_ok());
            assert!(parsed_chk.dim.is_ok());
            assert!(parsed_chk.sprp.is_ok());
            assert!(parsed_chk.forc.is_ok());
            assert!(parsed_chk.side.is_ok());
            assert!(parsed_chk.mtxm.is_ok());
            // assert!(parsed_chk.unit.is_ok());
            // assert!(parsed_chk.mask.is_ok());
            // assert!(parsed_chk.mrgn.is_ok());
        }
    }

    #[tokio::test]
    async fn specific_test_rise_of_empires() {
        let chk_data = get_chk("ef32c5707243bccb588a92d6180274f09597bfdae7b0aec1316f5b22f7b5bd74")
            .await
            .unwrap();
        let parsed_chk = ParsedChk::from_bytes(chk_data.as_slice());

        assert!(parsed_chk.sprp.is_ok());
    }

    #[tokio::test]
    async fn specific_test_achievementpyth() {
        let chk_data = get_chk("9b3b7e3c62b84021008be15626001c4da3b8c13c5118037861bfa4c94566b468")
            .await
            .unwrap();
        let parsed_chk = ParsedChk::from_bytes(chk_data.as_slice());

        assert!(parsed_chk.forc.is_ok());
    }
}
