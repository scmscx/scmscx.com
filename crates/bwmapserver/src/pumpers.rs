use anyhow::Result;
use async_stream::stream;
use backblaze::api::{b2_authorize_account, b2_get_upload_url, b2_upload_file};
use bytes::BytesMut;
use common::gsfs::gsfs_put_file;
use tokio::io::AsyncReadExt;
use tracing::{error, info, warn};

pub async fn start_gsfs_pumper(client: reqwest::Client) -> Result<()> {
    tokio::fs::create_dir_all("./pending/gsfs/mapblob").await?;
    tokio::fs::create_dir_all("./pending/gsfs/chkblob").await?;
    tokio::fs::create_dir_all("./pending/gsfs/img").await?;

    let Ok(endpoint) = std::env::var("GSFSFE_ENDPOINT") else {
        warn!("GSFSFE_ENDPOINT is not set, files will NOT be uploaded to GSFS!!!");
        return Ok(());
    };

    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let mut entries = walkdir::WalkDir::new("./pending/gsfs").into_iter();
            let (tx, mut rx) = tokio::sync::mpsc::channel(1);
            tokio::task::spawn_blocking(move || {
                while let Some(entry) = entries.next() {
                    if let Err(e) = tx.blocking_send(entry) {
                        error!("failed to send to channel: {e}");
                        return;
                    }
                }
            });

            while let Some(entry) = rx.recv().await {
                let entry = match entry {
                    Ok(v) => v,
                    Err(e) => {
                        error!("failed to walkdir, joinerr: {e}");
                        continue;
                    }
                };

                if !entry.file_type().is_file() {
                    continue;
                }

                if entry.file_name().to_str().unwrap().ends_with(".tmp") {
                    continue;
                }

                // get path relative to /pending/gsfs
                let filename = format!(
                    "/{}",
                    entry
                        .path()
                        .strip_prefix("./pending/gsfs")
                        .unwrap()
                        .to_str()
                        .unwrap()
                );

                info!("attempting to upload blob to gsfs: {filename}");
                match gsfs_put_file(
                    &client,
                    &endpoint,
                    entry.path().to_owned(),
                    filename.clone(),
                )
                .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        error!("failed to put file to gsfs: {err}, filename: {filename}");
                        continue;
                    }
                }

                if let Err(e) = tokio::fs::remove_file(entry.path()).await {
                    error!("failed to remove file: {e}");
                    continue;
                }

                info!("Successfully uploaded file to gsfs",);
            }
        }
    });

    Ok(())
}

// pub async fn start_gsfs_pumper(client: reqwest::Client) -> Result<()> {
//     tokio::fs::create_dir_all("./pending/gsfs").await?;

//     let Ok(endpoint) = std::env::var("GSFSFE_ENDPOINT") else {
//         warn!("GSFSFE_ENDPOINT is not set, maps will NOT be uploaded to GSFS!!!");
//         return Ok(());
//     };

//     tokio::task::spawn(async move {
//         loop {
//             tokio::time::sleep(std::time::Duration::from_secs(1)).await;

//             let mut entries = match tokio::fs::read_dir("./pending/gsfs").await {
//                 Ok(v) => v,
//                 Err(e) => {
//                     error!("could not readdir: {e:?}");
//                     continue;
//                 }
//             };

//             while let Ok(Some(entry)) = entries.next_entry().await {
//                 if let Ok(filetype) = entry.file_type().await {
//                     if !filetype.is_file() {
//                         continue;
//                     }
//                 }

//                 let Ok(mapblob_hash) = entry.file_name().into_string() else {
//                     error!("could not stringify filename: {:?}", entry.file_name());
//                     continue;
//                 };

//                 let mapblob_hash = mapblob_hash.replace(".scx", "");

//                 info!("attempting to upload mapblob to gsfs: {mapblob_hash}");
//                 match gsfs_put_mapblob(&client, &endpoint, entry.path(), &mapblob_hash).await {
//                     Ok(_) => {}
//                     Err(err) => {
//                         error!(
//                             "failed to put mapblob to gsfs: {err}, mapblob_hash: {mapblob_hash}"
//                         );
//                         continue;
//                     }
//                 }

//                 let chkblob_hash = {
//                     let path = entry.path().to_path_buf();
//                     let chkblob = match tokio::task::spawn_blocking(move || {
//                         bwmpq::get_chk_from_mpq_filename(&path)
//                     })
//                     .await
//                     {
//                         Ok(Ok(chkblob)) => chkblob,
//                         Ok(Err(error)) => {
//                             error!("failed to parse mpq: {error}");
//                             continue;
//                         }
//                         Err(error) => {
//                             error!("failed to join with task: {error}");
//                             continue;
//                         }
//                     };

//                     let chkblob_hash = calculate_hash_of_object(chkblob);

//                     info!("attempting to upload chkblob to gsfs: {chkblob_hash}");
//                     match gsfs_put_chkblob(&client, &endpoint, entry.path(), &chkblob_hash).await {
//                         Ok(_) => {}
//                         Err(err) => {
//                             error!("failed to put chkblob to gsfs: {err}, mapblob_hash: {mapblob_hash}, chkblob_hash: {chkblob_hash}");
//                             continue;
//                         }
//                     }

//                     chkblob_hash
//                 };

//                 if let Err(e) = tokio::fs::remove_file(entry.path()).await {
//                     error!("failed to remove file: {e}");
//                     continue;
//                 }

//                 info!(
//                     "Successfully uploaded file to gsfs: /scmscx.com/mapblob/{mapblob_hash}, /scmscx.com/chkblob/{chkblob_hash}",
//                 );
//             }
//         }
//     });

//     Ok(())
// }

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
                        Ok(_) => {}
                        Err(e) => {
                            error!("failed to b2_upload_file: {e}");
                            continue 'full_retry;
                        }
                    }

                    // Only proceed the file to the next stage if it was uploaded successfully.
                    match tokio::fs::remove_file(entry.path()).await {
                        Ok(_) => {}
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
