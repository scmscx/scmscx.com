use anyhow::Result;
use bytes::{Bytes, BytesMut};
use futures::Stream;
use reqwest::{Body, Client};
use std::path::Path;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::error;

async fn read_file_as_stream(
    path: impl AsRef<Path>,
    block_size: usize,
) -> Result<impl Stream<Item = Result<Bytes, std::io::Error>>> {
    let mut file = File::open(path).await?;
    Ok(async_stream::stream! {
        loop {
            let mut buf = BytesMut::with_capacity(block_size);
            let len = file.read_buf(&mut buf).await?;

            if len == 0 {
                break;
            }

            yield Ok(buf.freeze());
        }
    })
}

pub async fn gsfs_put_mapblob(
    client: &Client,
    endpoint: &str,
    path: impl AsRef<Path> + 'static,
    mapblob_hash: &str,
) -> Result<()> {
    let response = client
        .put(format!(
            "{endpoint}/api/fs/scmscx.com/mapblob/{mapblob_hash}"
        ))
        .body(Body::wrap_stream(
            read_file_as_stream(path, 1024 * 1024).await?,
        ))
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => (),
        e => {
            error!("gsfs put failed: {}", response.status());
            anyhow::bail!("gsfs put failed: {e}");
        }
    }

    Ok(())
}

pub async fn gsfs_get_mapblob(
    client: &Client,
    endpoint: &str,
    mapblob_hash: &str,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
    let response = client
        .get(format!(
            "{endpoint}/api/fs/scmscx.com/mapblob/{mapblob_hash}"
        ))
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => (),
        e => {
            error!("gsfs put failed: {}", response.status());
            anyhow::bail!("gsfs put failed: {e}");
        }
    }

    Ok(response.bytes_stream())
}
