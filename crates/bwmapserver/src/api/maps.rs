//! Map blob downloads: `/api/maps/{mapblob_hash}`.
//!
//! Served from GSFS when one is configured, falling back to Backblaze B2. The
//! shared B2 authorization handle lives here with its only consumer; the
//! pumpers authorize separately.

use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use backblaze::api::{b2_authorize_account, b2_download_file_by_name, B2AuthorizeAccount};
use bwcommon::MyError;
use common::gsfs::gsfs_get_mapblob;
use common::register_counter;
use futures::lock::Mutex;
use futures::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::error;

use crate::access;
use crate::util::finalize_hash_of_hasher;
use crate::webutil::{MaybeUser, Pool, PoolExt};

pub type BackblazeAuthState = Arc<Mutex<BackblazeAuth>>;

#[derive(Default)]
pub struct BackblazeAuth {
    pub version: usize,
    pub auth: Option<B2AuthorizeAccount>,
}

pub async fn get_auth(
    client: &reqwest::Client,
    backblaze_auth: &Mutex<BackblazeAuth>,
    bad_version: Option<usize>,
) -> Result<(usize, B2AuthorizeAccount)> {
    let mut lock = backblaze_auth.lock().await;

    let mut reacquire = false;

    if let Some(bv) = bad_version {
        if lock.version <= bv {
            reacquire = true;
            lock.version = bv;
        }
    }

    if lock.auth.is_none() || reacquire {
        let auth = b2_authorize_account(
            client,
            &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
            &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
        )
        .await;
        register_counter!(
            "scmscx",
            backblaze_auth,
            "Backblaze B2 authorize-account calls, by result",
            result = if auth.is_ok() { "ok" } else { "error" }
        )
        .inc();
        lock.auth = Some(auth?);

        lock.version = lock.version.checked_add(1).unwrap();
    }

    Ok((lock.version, lock.auth.clone().unwrap()))
}

pub async fn get_map(
    Extension(pool): Extension<Pool>,
    Extension(backblaze_auth): Extension<BackblazeAuthState>,
    Extension(reqwest_client): Extension<reqwest::Client>,
    headers: HeaderMap,
    Path((mapblob_hash,)): Path<(String,)>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    // The download is the whole point of blackholing: without this gate the map
    // file stays fetchable by hash to anyone who saved the URL, which is exactly
    // what a re-uploader has.
    if access::mapblob_is_hidden(&pool, &mapblob_hash, user.session()).await? {
        return Ok((StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-store")]).into_response());
    }

    {
        let mapblob_hash = mapblob_hash.clone();
        if let Some(useragent) = headers.get("user-agent") {
            if let Ok(useragent) = useragent.to_str() {
                if !useragent.contains("norecord") {
                    let time_since_epoch = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs() as i64;

                    let con = pool.acquire().await?;
                    let rows = con.execute(
                            "update map set downloads = downloads + 1, last_downloaded = $1 where mapblob2 = $2", &[&time_since_epoch, &mapblob_hash]).await?;
                    (|| {
                        anyhow::ensure!(rows == 1);
                        anyhow::Ok(())
                    })()?;
                }
            }
        }
    }

    const MAPBLOB_BUCKET_NAME: &str = "seventyseven-mapblob";
    let client = Client::new();

    let mut retries_remaining = 5;
    let mut bad_version = None;

    if let Ok(endpoint) = std::env::var("GSFSFE_ENDPOINT") {
        match gsfs_get_mapblob(&reqwest_client, &endpoint, &mapblob_hash).await {
            Ok(mut stream) => {
                register_counter!(
                    "scmscx",
                    map_download,
                    "Map blob download attempts, by source that served the blob",
                    source = "gsfs"
                )
                .inc();
                return Ok(IntoResponse::into_response((
                    [
                        (header::CONTENT_TYPE, "application/octet-stream"),
                        // Each download bumps a counter, so it must reach the origin
                        // every time — never let a cache serve this.
                        (header::CACHE_CONTROL, "no-store"),
                    ],
                    Body::from_stream(async_stream::stream! {
                        use sha2::Digest;
                        let mut hasher = sha2::Sha256::new();
                        let bytes_total = register_counter!(
                            "scmscx",
                            map_download_bytes,
                            "Total bytes streamed to clients for map downloads, by source",
                            source = "gsfs"
                        );

                        while let Some(chunk) = stream.next().await {
                            let chunk = chunk?;
                            bytes_total.inc_by(chunk.len() as u64);
                            hasher.update(&chunk);
                            yield Result::<_, anyhow::Error>::Ok(chunk);
                        }

                        if finalize_hash_of_hasher(hasher) != mapblob_hash {
                            yield Err(anyhow::anyhow!("Hash mismatch"));
                        }
                    }),
                )));
            }
            Err(error) => {
                error!("Failed to download from gsfs: {}", error);
            }
        }
    }

    while retries_remaining > 0 {
        let (version, api_info) = get_auth(&client, &backblaze_auth, bad_version).await?;

        retries_remaining -= 1;

        match b2_download_file_by_name(
            &client,
            &api_info,
            MAPBLOB_BUCKET_NAME,
            mapblob_hash.as_str(),
        )
        .await
        {
            Err(e) => {
                error!("Failed to download from backblaze: {}", e);
                bad_version = Some(version);
            }
            Ok(mut stream) => {
                register_counter!(
                    "scmscx",
                    map_download,
                    "Map blob download attempts, by source that served the blob",
                    source = "backblaze"
                )
                .inc();
                tokio::fs::create_dir_all("./pending/downloading").await?;

                let temp_filename = format!(
                    "./pending/downloading/{}.scx",
                    uuid::Uuid::new_v4().as_simple()
                );
                let mut temp_file = tokio::fs::File::create_new(&temp_filename).await;

                return Ok(IntoResponse::into_response((
                    [
                        (header::CONTENT_TYPE, "application/octet-stream"),
                        // Download counter side-effect: must not be cached.
                        (header::CACHE_CONTROL, "no-store"),
                    ],
                    Body::from_stream(async_stream::stream! {
                        use sha2::Digest;
                        let mut hasher = sha2::Sha256::new();
                        let bytes_total = register_counter!(
                            "scmscx",
                            map_download_bytes,
                            "Total bytes streamed to clients for map downloads, by source",
                            source = "backblaze"
                        );

                        while let Some(chunk) = stream.next().await {
                            let chunk = chunk?;
                            bytes_total.inc_by(chunk.len() as u64);
                            if let Ok(temp) = &mut temp_file {
                                if let Err(e) = temp.write_all(&chunk).await {
                                    error!("Failed to write to temp file: {e}, temp_filename: {temp_filename}");
                                    temp_file = Err(std::io::Error::from(std::io::ErrorKind::Other));
                                } else {
                                    hasher.update(&chunk);
                                }
                            }
                            yield Result::<_, anyhow::Error>::Ok(chunk);
                        }

                        if let Err(e) = tokio::fs::remove_file(&temp_filename).await {
                            error!("Failed to remove temp file: {e}, temp_filename: {temp_filename}");
                        }
                    }),
                )));
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    register_counter!(
        "scmscx",
        map_download,
        "Map blob download attempts, by source that served the blob",
        source = "failed"
    )
    .inc();
    Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
