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
        let mut bytes = BytesMut::with_capacity(block_size);

        loop {
            let len = file.read_buf(&mut bytes).await?;

            if len == 0 {
                break;
            }

            if bytes.len() >= block_size {
                yield Ok(bytes.freeze());
                bytes = BytesMut::with_capacity(block_size);
            }
        }

        if !bytes.is_empty() {
            yield Ok(bytes.freeze());
        }
    })
}

async fn gsfs_put(
    client: &Client,
    endpoint: &str,
    src: impl Stream<Item = Result<Bytes, std::io::Error>> + Sync + Send + 'static,
    dst: impl AsRef<str> + 'static,
) -> Result<()> {
    anyhow::ensure!(dst.as_ref().starts_with("/"), "dst must start with /");

    let response = client
        .put(format!(
            "{endpoint}/api/namespace/scmscx.com{}",
            dst.as_ref()
        ))
        .body(Body::wrap_stream(src))
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

async fn gsfs_get(
    client: &Client,
    endpoint: &str,
    path: impl AsRef<str> + 'static,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
    anyhow::ensure!(path.as_ref().starts_with("/"), "path must start with /");

    let response = client
        .get(format!(
            "{endpoint}/api/namespace/scmscx.com{}",
            path.as_ref()
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

pub async fn gsfs_put_mapblob(
    client: &Client,
    endpoint: &str,
    path: impl AsRef<Path> + 'static,
    mapblob_hash: &str,
) -> Result<()> {
    gsfs_put(
        client,
        endpoint,
        read_file_as_stream(path, 1024 * 1024).await?,
        format!("/mapblob/{mapblob_hash}"),
    )
    .await
}

pub async fn gsfs_get_mapblob(
    client: &Client,
    endpoint: &str,
    mapblob_hash: &str,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
    gsfs_get(client, endpoint, format!("/mapblob/{mapblob_hash}")).await
}

pub async fn gsfs_put_chkblob(
    client: &Client,
    endpoint: &str,
    path: impl AsRef<Path> + 'static,
    chkblob_hash: &str,
) -> Result<()> {
    gsfs_put(
        client,
        endpoint,
        read_file_as_stream(path, 1024 * 1024).await?,
        format!("/chkblob/{chkblob_hash}"),
    )
    .await
}

// pub async fn gsfs_get_chkblob(
//     client: &Client,
//     endpoint: &str,
//     chkblob_hash: &str,
// ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
//     gsfs_get(client, endpoint, format!("/chkblob/{chkblob_hash}")).await
// }
