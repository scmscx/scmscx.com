use crate::api::{
    b2_authorize_account, b2_download_file_by_name, b2_get_upload_url, b2_upload_file,
    B2AuthorizeAccount, B2Error,
};
use anyhow::Result;
use assert_matches::assert_matches;
use async_stream::stream;
use bytes::{Bytes, BytesMut};
use futures::stream::StreamExt;
use futures::Stream;
use reqwest::{Client, Error};
use sha1::{Digest, Sha1};
use std::pin::pin;

const TEST_BUCKET: &'static str = "386b8f2e6e36dc507ee50d1c";
const TEST_BUCKET_NAME: &'static str = "sventyseven-test";

async fn download_stream(stream: impl Stream<Item = Result<Bytes, Error>>) -> Result<Bytes> {
    let mut bytes = BytesMut::new();

    let mut stream = pin!(stream);

    while let Some(b) = stream.next().await {
        bytes.extend_from_slice(&b?[..]);
    }

    Ok(bytes.freeze())
}

#[tokio::test]
#[ignore]
async fn test_b2_authorize_account() -> Result<(), anyhow::Error> {
    let client = Client::new();
    assert_matches!(
        b2_authorize_account(
            &client,
            "bad",
            &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
        )
        .await,
        Err(B2Error::BadAuthToken(_))
    );

    assert_matches!(
        b2_authorize_account(&client, &std::env::var("BACKBLAZE_KEY_ID").unwrap(), "bad",).await,
        Err(B2Error::BadAuthToken(_))
    );

    assert_matches!(
        b2_authorize_account(
            &client,
            &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
            &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
        )
        .await,
        Ok(_)
    );

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_b2_get_upload_url() -> Result<(), anyhow::Error> {
    let client = Client::new();
    let api_info = b2_authorize_account(
        &client,
        &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
        &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
    )
    .await?;

    // B2AuthorizeAccount{ authorizationToken: "Bad Auth Token", apiUrl: "Bad Api Url", downloadUrl: "Bad Download Url" }}

    assert_matches!(
        b2_get_upload_url(
            &client,
            &B2AuthorizeAccount {
                authorizationToken: "Bad Auth Token".to_owned(),
                ..api_info.clone()
            },
            TEST_BUCKET
        )
        .await,
        Err(B2Error::BadAuthToken(_))
    );

    assert_matches!(
        b2_get_upload_url(
            &client,
            &B2AuthorizeAccount {
                apiUrl: "Bad Api Url".to_owned(),
                ..api_info.clone()
            },
            TEST_BUCKET
        )
        .await,
        Err(B2Error::Network(_))
    );

    assert_matches!(
        b2_get_upload_url(
            &client,
            &B2AuthorizeAccount {
                downloadUrl: "Bad Download Url".to_owned(),
                ..api_info.clone()
            },
            TEST_BUCKET
        )
        .await,
        Ok(_)
    );

    assert_matches!(
        b2_get_upload_url(&client, &api_info, "bad bucket id").await,
        Err(B2Error::BadRequest(_))
    );

    assert_matches!(
        b2_get_upload_url(&client, &api_info, TEST_BUCKET).await,
        Ok(_)
    );

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_b2_upload_file() -> Result<(), anyhow::Error> {
    let prepare_stream = |data: &[u8]| {
        let sha1_hash = format!("{:x}", Sha1::new_with_prefix(data).finalize());
        let bytes = Bytes::copy_from_slice(data);

        (
            data.len(),
            sha1_hash,
            stream! {
                yield anyhow::Ok(bytes);
            },
        )
    };

    let client = Client::new();
    let api_info = b2_authorize_account(
        &client,
        &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
        &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
    )
    .await?;

    let upload_url = b2_get_upload_url(&client, &api_info, TEST_BUCKET).await?;

    for _ in 0..50 {
        let filename = uuid::Uuid::new_v4().as_simple().to_string();

        let data = b"abc";

        let (len, sha1_hash, stream) = prepare_stream(data);
        assert_matches!(
            b2_upload_file(&client, &upload_url, &filename, len, sha1_hash, stream).await,
            Ok(_)
        );
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_b2_b2_download_file_by_name() -> Result<(), anyhow::Error> {
    let client = Client::new();
    let api_info = b2_authorize_account(
        &client,
        &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
        &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
    )
    .await?;

    let filename = uuid::Uuid::new_v4().as_simple().to_string();

    assert_matches!(
        b2_download_file_by_name(&client, &api_info, TEST_BUCKET_NAME, &filename).await,
        Err(B2Error::NotFound(_))
    );

    assert_matches!(
        b2_download_file_by_name(&client, &api_info, "bad bucket name with spaces", &filename)
            .await,
        Err(B2Error::BadRequest(_))
    );

    assert_matches!(
        b2_download_file_by_name(&client, &api_info, "bad_bucket_name", &filename).await,
        Err(B2Error::NotFound(_))
    );

    Ok(())
}

#[tokio::test]
#[ignore]
async fn do_some_backblaze_stuff2() -> Result<(), anyhow::Error> {
    let prepare_stream = |data: &[u8]| {
        let sha1_hash = format!("{:x}", Sha1::new_with_prefix(data).finalize());
        let bytes = Bytes::copy_from_slice(data);

        (
            data.len(),
            sha1_hash,
            stream! {
                yield anyhow::Ok(bytes);
            },
        )
    };

    let client = Client::new();

    let api_info = b2_authorize_account(
        &client,
        &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
        &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
    )
    .await?;

    let upload_url = b2_get_upload_url(&client, &api_info, TEST_BUCKET).await?;

    let filename = uuid::Uuid::new_v4().as_simple().to_string();

    let data = b"abc";

    let (len, sha1_hash, stream) = prepare_stream(data);
    b2_upload_file(&client, &upload_url, &filename, len, sha1_hash, stream).await?;

    let downloaded_data = download_stream(
        b2_download_file_by_name(&client, &api_info, "sventyseven-test", &filename).await?,
    )
    .await?;

    assert_eq!(downloaded_data[..], data[..]);

    Ok(())
}

// #[tokio::test]
// async fn do_some_backblaze_stuff() {
//     let client = Client::new();

//     let api_info = b2_authorize_account(&client).await.unwrap();
//     let files = list_files_in_bucket(&client, &api_info, TEST_BUCKET, None)
//         .await
//         .unwrap();
//     assert!(files.files.len() == 0);

//     let upload_info = b2_get_upload_url(&client, &api_info, TEST_BUCKET)
//         .await
//         .unwrap();
//     upload_file_to_backblaze_from_memory(&client, &upload_info, "eee", b"123".to_vec())
//         .await
//         .unwrap();

//     let files = list_files_in_bucket(&client, &api_info, TEST_BUCKET, None)
//         .await
//         .unwrap();
//     assert!(files.files.len() == 1);
//     let file = &files.files[0];
//     assert!(file.fileName == "eee");

//     let data = download_file(&client, &api_info, file.fileId.as_str())
//         .await
//         .unwrap();
//     assert!(data == "123");

//     let data = download_file_by_name(
//         &client,
//         &api_info,
//         "sventyseven-test",
//         file.fileName.as_str(),
//     )
//     .await
//     .unwrap();
//     assert!(data == "123");

//     let mut stream = download_stream(&client, &api_info, file.fileId.as_str())
//         .await
//         .unwrap();
//     let mut bytes = Vec::new();
//     while let Some(item) = stream.next().await {
//         bytes.extend(item.unwrap());
//     }
//     assert!(bytes == b"123");

//     let mut stream = download_file_by_name_stream(
//         &client,
//         &api_info,
//         "sventyseven-test",
//         file.fileName.as_str(),
//     )
//     .await
//     .unwrap();
//     let mut bytes = Vec::new();
//     while let Some(item) = stream.next().await {
//         bytes.extend(item.unwrap());
//     }
//     assert!(bytes == b"123");

//     delete_file(
//         &client,
//         &api_info,
//         file.fileName.as_str(),
//         file.fileId.as_str(),
//     )
//     .await
//     .unwrap();

//     let files = list_files_in_bucket(&client, &api_info, TEST_BUCKET, None)
//         .await
//         .unwrap();
//     assert!(files.files.len() == 0);

//     // test large file uploading
//     let start_large_file_response = b2_start_large_file(&client, &api_info, TEST_BUCKET, "bigfile")
//         .await
//         .unwrap();

//     let upload_part_info = b2_get_upload_part_url(&client, &api_info, &start_large_file_response)
//         .await
//         .unwrap();

//     let part1 = b2_upload_part(&client, &upload_part_info, 1, vec![1; 5 * 1024 * 1024])
//         .await
//         .unwrap();
//     let part2 = b2_upload_part(&client, &upload_part_info, 2, vec![1; 5 * 1024 * 1024])
//         .await
//         .unwrap();

//     let b2_finish_large_file_response = b2_finish_large_file(
//         &client,
//         &api_info,
//         &start_large_file_response,
//         &[part1.contentSha1, part2.contentSha1],
//     )
//     .await
//     .unwrap();

//     let data = download_file(
//         &client,
//         &api_info,
//         b2_finish_large_file_response.fileId.as_str(),
//     )
//     .await
//     .unwrap();
//     assert!(data == vec![1; 10 * 1024 * 1024]);

//     delete_file(
//         &client,
//         &api_info,
//         "bigfile",
//         upload_part_info.fileId.as_str(),
//     )
//     .await
//     .unwrap();
// }
