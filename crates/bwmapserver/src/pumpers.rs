use anyhow::Result;
use async_stream::stream;
use backblaze::api::{b2_authorize_account, b2_get_upload_url, b2_upload_file};
use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tracing::{error, info, warn};

pub async fn start_gsfs_pumper(client: reqwest::Client) -> Result<()> {
    info!("starting gsfs pumper");

    tokio::fs::create_dir_all("./pending/gsfs").await?;

    let Ok(endpoint) = std::env::var("GSFSFE_ENDPOINT") else {
        warn!(
            "GSFSFE_ENDPOINT not set, gsfs pumper DISABLED, maps will NOT be uploaded to gsfs!!!"
        );
        return Ok(());
    };

    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let mut entries = match tokio::fs::read_dir("./pending/gsfs").await {
                Ok(v) => v,
                Err(e) => {
                    error!("could not readdir ./pending/gsfs: {e:?}");
                    continue;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                // Files are named by their sha256 (the mapblob_hash).
                let Ok(mapblob_hash) = entry.file_name().into_string() else {
                    error!("could not stringify filename: {:?}", entry.file_name());
                    continue;
                };

                info!("attempting to upload mapblob to gsfs: {mapblob_hash}");

                match common::gsfs::gsfs_put_mapblob_file(
                    &client,
                    &endpoint,
                    &mapblob_hash,
                    entry.path(),
                )
                .await
                {
                    Ok(()) => {}
                    Err(e) => {
                        error!("failed to upload mapblob to gsfs: {e}");
                        continue;
                    }
                }

                // Only remove the file once it has been uploaded successfully.
                match tokio::fs::remove_file(entry.path()).await {
                    Ok(()) => {}
                    Err(e) => {
                        error!("failed to remove file: {e}");
                        continue;
                    }
                }

                info!("Successfully uploaded mapblob to gsfs: {mapblob_hash}");
            }
        }
    });

    Ok(())
}

pub async fn start_backblaze_pumper(client: reqwest::Client) -> Result<()> {
    info!("starting backblaze pumper");

    tokio::fs::create_dir_all("./pending/backblaze").await?;

    match std::env::var("BACKBLAZE_DISABLED") {
        Ok(v) if v == "true" => {
            warn!("backblaze is DISABLED, maps will NOT be uploaded to backblaze!!!");
            return Ok(());
        }
        _ => {}
    }

    tokio::task::spawn(async move {
        'full_retry: loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let api_info = match b2_authorize_account(
                &client,
                &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
                &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
            )
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to authorize account: {e}");
                    continue;
                }
            };

            let upload_url = match b2_get_upload_url(
                &client,
                &api_info,
                &std::env::var("BACKBLAZE_MAPBLOB_BUCKET").unwrap(),
            )
            .await
            {
                Ok(upload_url) => upload_url,
                Err(e) => {
                    error!("Failed to get upload url, trying again: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                let mut entries = match tokio::fs::read_dir("./pending/backblaze").await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("could not readdir: {e:?}");
                        continue;
                    }
                };

                while let Ok(Some(entry)) = entries.next_entry().await {
                    let Ok(filename) = entry.file_name().into_string() else {
                        error!("could not stringify filename: {:?}", entry.file_name());
                        continue;
                    };

                    info!("attempting to upload file: {filename}");

                    let mut split = filename.split('-');
                    let Some(sha1) = split.next() else {
                        error!("could not extract sha1 part: {:?}", filename);
                        continue;
                    };
                    let Some(sha256) = split.next() else {
                        error!("could not extract sha256 part: {:?}", filename);
                        continue;
                    };

                    let mut file = match tokio::fs::File::open(entry.path()).await {
                        Ok(v) => v,
                        Err(e) => {
                            error!("failed to open file: {e:?}");
                            continue;
                        }
                    };

                    let metadata = match file.metadata().await {
                        Ok(v) => v,
                        Err(e) => {
                            error!("failed to get file metadata: {e:?}");
                            continue;
                        }
                    };

                    let sm = stream! {
                        loop {
                            let mut bytes = BytesMut::with_capacity(8 * 1024 * 1024);
                            let bytes_read = file.read_buf(&mut bytes).await?;
                            if bytes_read == 0 {
                                break;
                            }

                            yield anyhow::Ok(bytes);
                        }
                    };

                    match b2_upload_file(
                        &client,
                        &upload_url,
                        sha256,
                        metadata.len() as usize,
                        sha1.to_owned(),
                        sm,
                    )
                    .await
                    {
                        Ok(()) => {}
                        Err(e) => {
                            error!("failed to b2_upload_file: {e}");
                            continue 'full_retry;
                        }
                    }

                    // Only proceed the file to the next stage if it was uploaded successfully.
                    match tokio::fs::remove_file(entry.path()).await {
                        Ok(()) => {}
                        Err(e) => {
                            error!("failed to remove file: {e}");
                            continue;
                        }
                    }

                    info!(
                        "Successfully uploaded file to backblaze: {}",
                        entry.path().display()
                    );
                }
            }
        }
    });

    Ok(())
}
