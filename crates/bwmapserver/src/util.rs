// use futures::FutureExt;

use sha1::digest;

pub const SEED_MAP_ID: u8 = 97;

pub fn parse_map_id(s: &str) -> Result<i64, anyhow::Error> {
    if s.chars().all(|c| c.is_ascii_digit()) && s.len() < 8 {
        Ok(s.parse::<i64>()?)
    } else {
        bwcommon::get_db_id_from_web_id(s, SEED_MAP_ID)
    }
}

pub fn is_dev_mode() -> bool {
    std::env::var("DEV_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .as_str()
        == "true"
}

pub(crate) fn sanitize_sc_string(s: &str) -> String {
    // split string by left or right marks

    let mut strings: Vec<_> = s.split(['\u{0012}', '\u{0013}']).collect();

    strings.sort_by_key(|x| std::cmp::Reverse(x.len()));

    if strings.is_empty() {
        String::new()
    } else {
        strings[0].chars().filter(|&x| x >= ' ').collect()
    }
}

// pub fn calculate_hash_of_object(object: impl AsRef<[u8]>) -> String {
//     use sha2::Digest;
//     let mut hasher = sha2::Sha256::new();
//     hasher.update(&object);
//     finalize_hash_of_hasher(hasher)
// }

pub fn finalize_hash_of_hasher<D: digest::Digest + digest::FixedOutput>(hasher: D) -> String
where
    <D as digest::OutputSizeUser>::OutputSize: std::ops::Add,
    <<D as digest::OutputSizeUser>::OutputSize as std::ops::Add>::Output:
        digest::generic_array::ArrayLength<u8>,
{
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_map_id_accepts_short_numeric() {
        // Fewer than 8 all-ASCII-digit chars → parsed as a raw DB id.
        assert_eq!(parse_map_id("0").unwrap(), 0);
        assert_eq!(parse_map_id("123").unwrap(), 123);
        assert_eq!(parse_map_id("1234567").unwrap(), 1_234_567);
    }

    #[test]
    fn parse_map_id_decodes_web_ids_roundtrip() {
        // An 8-char web id round-trips through the obfuscation scheme.
        for db_id in [0i64, 1, 42, 39807, (1 << 20) - 1] {
            let web = bwcommon::get_web_id_from_db_id(db_id, SEED_MAP_ID).unwrap();
            assert_eq!(web.len(), 8, "web ids are 8 chars");
            assert_eq!(
                parse_map_id(&web).unwrap(),
                db_id,
                "parse_map_id must decode web id {web} back to {db_id}"
            );
        }
    }

    #[test]
    fn parse_map_id_long_digit_string_is_treated_as_web_id() {
        // 8+ digits is NOT a raw id (the `s.len() < 8` guard); it's routed to the
        // web-id decoder, which rejects it (digits aren't all valid base32 chars,
        // or the checksum fails).
        assert!(parse_map_id("12345678").is_err());
    }

    #[test]
    fn parse_map_id_rejects_garbage() {
        assert!(parse_map_id("not-an-id").is_err());
        assert!(parse_map_id("").is_err());
    }

    #[test]
    fn sanitize_sc_string_picks_longest_segment_between_control_marks() {
        // \u{0012} and \u{0013} are the left/right alignment marks used to split.
        let s = "short\u{0012}the longest segment\u{0013}mid";
        assert_eq!(sanitize_sc_string(s), "the longest segment");
    }

    #[test]
    fn sanitize_sc_string_filters_control_chars() {
        // Characters below ' ' (0x20) are stripped from the chosen segment.
        let s = "ab\u{0001}c\u{0007}d";
        assert_eq!(sanitize_sc_string(s), "abcd");
    }

    #[test]
    fn sanitize_sc_string_empty() {
        assert_eq!(sanitize_sc_string(""), "");
    }

    #[test]
    fn is_dev_mode_reflects_env() {
        // `is_dev_mode` is a thin wrapper over the DEV_MODE env var; only the exact
        // string "true" enables it. Exercising the true branch is the one case the
        // E2E suite can't (it always runs with DEV_MODE removed → prod mode), so
        // without this a "-> false" mutation of the whole function goes unnoticed.
        // DEV_MODE isn't read by any other unit test, so mutating this process-global
        // for the duration of one test is safe.
        let prev = std::env::var("DEV_MODE").ok();

        std::env::set_var("DEV_MODE", "true");
        assert!(is_dev_mode(), "DEV_MODE=true enables dev mode");

        std::env::set_var("DEV_MODE", "false");
        assert!(!is_dev_mode(), "DEV_MODE=false is prod mode");

        std::env::set_var("DEV_MODE", "1");
        assert!(!is_dev_mode(), "only the literal \"true\" enables dev mode");

        std::env::remove_var("DEV_MODE");
        assert!(!is_dev_mode(), "an unset DEV_MODE is prod mode");

        match prev {
            Some(v) => std::env::set_var("DEV_MODE", v),
            None => std::env::remove_var("DEV_MODE"),
        }
    }

    #[test]
    fn finalize_hash_of_hasher_formats_lowercase_hex_digest() {
        use sha2::{Digest, Sha256};

        // The known SHA-256 of the empty input, as lowercase hex — pins that the
        // function returns the real digest (not an empty/placeholder string).
        let mut hasher = Sha256::new();
        hasher.update(b"");
        assert_eq!(
            finalize_hash_of_hasher(hasher),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );

        // A different input hashes to a different 64-char hex string.
        let mut hasher = Sha256::new();
        hasher.update(b"scmscx.com");
        let digest = finalize_hash_of_hasher(hasher);
        assert_eq!(digest.len(), 64, "sha-256 hex is 64 chars");
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(
            digest, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "a non-empty input must not collide with the empty digest"
        );
    }
}

// pub(crate) fn sanitize_sc_scenario_string(s: &str) -> String {
//     // split string by left or right marks

//     let mut strings: Vec<_> = s.split(|x| x == '\u{0012}' || x == '\u{0013}').collect();

//     strings.sort_by_key(|x| std::cmp::Reverse(x.len()));

//     if strings.len() == 0 {
//         String::new()
//     } else {
//         strings[0].to_string()
//     }
// }

// pub(crate) fn sanitize_sc_string_preserve_newlines(s: &str) -> String {
//     s.split('\n')
//         .map(sanitize_sc_string)
//         .collect::<Vec<_>>()
//         .join("\n")
// }

// pub(crate) async fn process_iter_async_concurrent<I, T, F, J, R, F2>(
//     mut iter: I,
//     max_outstanding: usize,
//     on_item_completed: F2,
//     func: F,
// ) -> usize
// where
//     I: Iterator<Item = T>,
//     F: Fn(T) -> R,
//     R: futures::Future<Output = J> + Send,
//     F2: Fn(usize, J),
// {
//     use futures::FutureExt;

//     let mut futs = Vec::new();
//     let mut counter = 0;
//     loop {
//         while futs.len() < max_outstanding {
//             if let Some(entry) = iter.next() {
//                 futs.push(func(entry).boxed());
//             } else {
//                 break;
//             }
//         }

//         if futs.len() == 0 {
//             break;
//         }

//         let (item, _, remaining_futures) = futures::future::select_all(futs).await;

//         futs = remaining_futures;

//         counter += 1;

//         on_item_completed(counter, item);
//     }

//     counter
// }

// pub async fn upload_file_to_backblaze(
//     file_name: &str,
//     len: usize,
//     data: &[u8],
// ) -> anyhow::Result<()> {
//     upload_file_to_backblaze_from_stream(
//         file_name,
//         len,
//         "asds".to_owned(),
//         futures_util::stream::iter(data),
//     )
//     .await
// }

// pub async fn upload_file_to_backblaze_from_stream<S>(
//     file_name: &str,
//     len: usize,
//     sha1_hash: String,
//     stream: S,
// ) -> anyhow::Result<()>
// where
//     S: futures_core::stream::TryStream + Send + Sync + 'static,
//     S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
//     Bytes: From<S::Ok>,
// {
//     let client = Client::new();

//     let api_info = get_backblaze_auth_info(&client).await?;
//     let upload_info =
//         get_backblaze_upload_url(&client, &api_info, "b80b4f2e8e26dc107ee50d1c").await?;
//     upload_file_to_backblaze(&client, &upload_info, file_name, len, sha1_hash, stream).await?;

//     anyhow::Ok(())
// }

// pub(crate) async fn process_iter_async_concurrent<I, T, F, J, R, F2, H, Z>(
//     mut iter: I,
//     cloner: H,
//     max_outstanding: usize,
//     on_item_completed: F2,
//     func: F,
// ) -> usize
// where
//     I: Iterator<Item = T>,
//     F: Fn(Z, T) -> R,
//     R: futures::Future<Output = J> + Send,
//     F2: Fn(usize, J),
//     H: Fn() -> Z,
// {
//     let mut futs = Vec::new();
//     let mut counter = 0;
//     loop {
//         while futs.len() < max_outstanding {
//             if let Some(entry) = iter.next() {
//                 futs.push(func(cloner(), entry).boxed());
//             } else {
//                 break;
//             }
//         }

//         if futs.len() == 0 {
//             break;
//         }

//         let (item, _, remaining_futures) = futures::future::select_all(futs).await;

//         futs = remaining_futures;

//         counter += 1;

//         on_item_completed(counter, item);
//     }

//     counter
// }
