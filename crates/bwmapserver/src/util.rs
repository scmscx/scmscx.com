// use futures::FutureExt;

use sha1::digest;

pub const SEED_MAP_ID: u8 = 97;

pub fn is_dev_mode() -> bool {
    std::env::var("DEV_MODE")
        .unwrap_or("false".to_string())
        .as_str()
        == "true"
}

pub(crate) fn sanitize_sc_string(s: &str) -> String {
    // split string by left or right marks

    let mut strings: Vec<_> = s.split(|x| x == '\u{0012}' || x == '\u{0013}').collect();

    strings.sort_by_key(|x| std::cmp::Reverse(x.len()));

    if strings.len() == 0 {
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
