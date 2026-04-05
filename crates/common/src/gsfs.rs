use anyhow::Result;
use bytes::{Bytes, BytesMut};
use futures::Stream;
use futures_util::StreamExt;
use reqwest::{Body, Client};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info};

fn read_vec_as_stream(
    slice: Vec<u8>,
    block_size: usize,
) -> impl Stream<Item = Result<Bytes, std::io::Error>> + 'static {
    async_stream::stream! {
        for s in slice.chunks(block_size) {
            yield Ok(Bytes::copy_from_slice(s));
        }
    }
}

async fn read_file_as_stream(
    path: impl AsRef<Path>,
    block_size: usize,
) -> Result<impl Stream<Item = Result<Bytes, std::io::Error>>> {
    let mut file = tokio::fs::File::open(path).await?;
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
    anyhow::ensure!(dst.as_ref().starts_with('/'), "dst must start with /");

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
    anyhow::ensure!(path.as_ref().starts_with('/'), "path must start with /");

    let response = client
        .get(format!(
            "{endpoint}/api/namespace/scmscx.com{}",
            path.as_ref()
        ))
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => (),
        reqwest::StatusCode::NOT_FOUND => {
            info!("gsfs get failed: {}", response.status());
            anyhow::bail!("gsfs get failed, NotFound");
        }
        e => {
            error!("gsfs get failed: {}", response.status());
            anyhow::bail!("gsfs get failed: {e}");
        }
    }

    Ok(response.bytes_stream())
}

pub async fn gsfs_get_map_image(
    client: &Client,
    endpoint: &str,
    chkblob_hash: &str,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
    gsfs_get(client, endpoint, format!("/img/{chkblob_hash}.webp")).await
}

pub async fn gsfs_get_mapblob(
    client: &Client,
    endpoint: &str,
    mapblob_hash: &str,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
    gsfs_get(client, endpoint, format!("/mapblob/{mapblob_hash}")).await
}

pub async fn gsfs_put_file(
    client: &Client,
    endpoint: &str,
    path: impl AsRef<Path> + 'static,
    filename: String,
) -> Result<()> {
    gsfs_put(
        client,
        endpoint,
        read_file_as_stream(path, 1024 * 1024).await?,
        filename,
    )
    .await
}

pub async fn gsfs_put_minimap(
    client: &Client,
    endpoint: &str,
    chkblob_hash: &str,
    png_data: Vec<u8>,
) -> Result<()> {
    gsfs_put(
        client,
        endpoint,
        read_vec_as_stream(png_data, 1024 * 1024),
        format!("/minimap/{chkblob_hash}"),
    )
    .await
}

pub async fn gsfs_get_minimap(
    client: &Client,
    endpoint: &str,
    chkblob_hash: &str,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
    gsfs_get(client, endpoint, format!("/minimap/{chkblob_hash}")).await
}

pub async fn gsfs_put_map_image(
    client: &Client,
    endpoint: &str,
    chkblob_hash: &str,
    data: Vec<u8>,
) -> Result<()> {
    gsfs_put_bytes(client, endpoint, &format!("/img/{chkblob_hash}.webp"), data).await
}

pub async fn gsfs_put_mapblob(
    client: &Client,
    endpoint: &str,
    mapblob_hash: &str,
    data: Vec<u8>,
) -> Result<()> {
    gsfs_put_bytes(client, endpoint, &format!("/mapblob/{mapblob_hash}"), data).await
}

pub async fn gsfs_put_chkblob(
    client: &Client,
    endpoint: &str,
    chkblob_hash: &str,
    data: Vec<u8>,
) -> Result<()> {
    gsfs_put_bytes(client, endpoint, &format!("/chkblob/{chkblob_hash}"), data).await
}

async fn gsfs_put_bytes(client: &Client, endpoint: &str, path: &str, data: Vec<u8>) -> Result<()> {
    gsfs_put(
        client,
        endpoint,
        read_vec_as_stream(data, 1024 * 1024),
        path.to_string(),
    )
    .await
}

pub async fn gsfs_download_mapblob_to_file(
    client: &Client,
    endpoint: &str,
    mapblob_hash: &str,
    dest_path: impl AsRef<Path>,
) -> Result<()> {
    gsfs_download_to_file(
        client,
        endpoint,
        &format!("/mapblob/{mapblob_hash}"),
        dest_path,
    )
    .await
}

async fn gsfs_download_to_file(
    client: &Client,
    endpoint: &str,
    gsfs_path: &str,
    dest_path: impl AsRef<Path>,
) -> Result<()> {
    let mut stream = gsfs_get(client, endpoint, gsfs_path.to_string()).await?;
    let mut file = tokio::fs::File::create(dest_path).await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    file.flush().await?;
    Ok(())
}
