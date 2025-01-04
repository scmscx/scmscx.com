use anyhow::Result;
use bytes::Bytes;
use futures::Stream;
use pin_project::pin_project;
use reqwest::{Body, Client, StatusCode};
use serde::Deserialize;
use B2Error::Network;

#[derive(thiserror::Error, Debug, Deserialize)]
#[error("status: {status}, code: {code}, message: {message}")]
pub struct B2ErrorBody {
    pub status: usize,
    pub code: String,
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum B2Error {
    #[error("Unknown Error Occured: {}", .0)]
    Unknown(B2ErrorBody),

    #[error("Network Error Occured: {}", .0)]
    Network(reqwest::Error),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Bad Bucket Id: {0}")]
    BadBucketId(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Bad Auth Token: {0}")]
    BadAuthToken(String),

    #[error("Expired Auth Token: {0}")]
    ExpiredAuthToken(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Storage Cap Exceeded: {0}")]
    StorageCapExceeded(String),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),
}

impl From<B2ErrorBody> for B2Error {
    fn from(value: B2ErrorBody) -> Self {
        use B2Error::*;

        match value.code.as_str() {
            "bad_bucket_id" => BadBucketId(value.message),
            "bad_request" => BadRequest(value.message),
            "bad_auth_token" => BadAuthToken(value.message),
            "expired_auth_token" => ExpiredAuthToken(value.message),
            "unauthorized" => Unauthorized(value.message),
            "storage_cap_exceeded" => StorageCapExceeded(value.message),
            "service_unavailable" => ServiceUnavailable(value.message),
            "not_found" => NotFound(value.message),
            _ => Unknown(value),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct B2AuthorizeAccount {
    pub authorizationToken: String,
    pub apiUrl: String,
    pub downloadUrl: String,
}

pub async fn b2_authorize_account(
    client: &Client,
    key_id: &str,
    application_key: &str,
) -> Result<B2AuthorizeAccount, B2Error> {
    let response = client
        .get("https://api.backblazeb2.com/b2api/v2/b2_authorize_account")
        .basic_auth(key_id, Some(application_key))
        .send()
        .await
        .map_err(Network)?;

    match response.status() {
        StatusCode::OK => Ok(response
            .json::<B2AuthorizeAccount>()
            .await
            .map_err(Network)?),
        _ => Err(response
            .json::<B2ErrorBody>()
            .await
            .map_err(Network)?
            .into()),
    }
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct B2GetUploadUrl {
    pub bucketId: String,
    pub uploadUrl: String,
    pub authorizationToken: String,
}

pub async fn b2_get_upload_url(
    client: &Client,
    api_info: &B2AuthorizeAccount,
    bucket_id: &str,
) -> Result<B2GetUploadUrl, B2Error> {
    let response = client
        .post(format!(
            "{}{}",
            api_info.apiUrl, "/b2api/v2/b2_get_upload_url"
        ))
        .header("Authorization", &api_info.authorizationToken)
        .header("Content-Type", "application/json; charset=utf-8")
        .body(serde_json::json!({ "bucketId": bucket_id }).to_string())
        .send()
        .await
        .map_err(Network)?;

    match response.status() {
        StatusCode::OK => Ok(response.json::<B2GetUploadUrl>().await.map_err(Network)?),
        _ => Err(response
            .json::<B2ErrorBody>()
            .await
            .map_err(Network)?
            .into()),
    }
}

#[pin_project]
pub struct B2DownloadFileByName<S>(#[pin] S)
where
    S: Stream<Item = Result<Bytes, reqwest::Error>>;

impl<S> Stream for B2DownloadFileByName<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>>,
{
    type Item = S::Item;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let pinned = this.0;
        pinned.poll_next(cx)
    }
}

impl<T: futures::Stream<Item = Result<Bytes, reqwest::Error>>> std::fmt::Debug
    for B2DownloadFileByName<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("B2DownloadFileByName")
    }
}

pub async fn b2_download_file_by_name(
    client: &Client,
    api_info: &B2AuthorizeAccount,
    bucket_name: &str,
    file_name: &str,
) -> Result<B2DownloadFileByName<impl Stream<Item = Result<Bytes, reqwest::Error>>>, B2Error> {
    use crate::api::B2Error::*;

    let response = client
        .get(format!(
            "{}/file/{}/{}",
            api_info.downloadUrl, bucket_name, file_name
        ))
        .header("Authorization", &api_info.authorizationToken)
        .send()
        .await
        .map_err(Network)?;

    match response.status() {
        StatusCode::OK => Ok(B2DownloadFileByName(response.bytes_stream())),
        _ => Err(response
            .json::<B2ErrorBody>()
            .await
            .map_err(Network)?
            .into()),
    }
}

pub async fn b2_upload_file<S>(
    client: &Client,
    upload_info: &B2GetUploadUrl,
    file_name: &str,
    len: usize,
    sha1_hash: String,
    stream: S,
) -> Result<(), B2Error>
where
    S: futures_core::stream::TryStream + Send + Sync + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Bytes: From<S::Ok>,
{
    let response = client
        .post(&upload_info.uploadUrl)
        .header("Authorization", &upload_info.authorizationToken)
        .header("X-Bz-File-Name", file_name)
        .header("X-Bz-Content-Sha1", sha1_hash)
        .header("Content-Length", len)
        .header("Content-Type", "application/octet-stream")
        // .header("X-Bz-Test-Mode", "fail_some_uploads")
        .body(Body::wrap_stream(stream))
        .send()
        .await
        .map_err(Network)?;

    match response.status() {
        StatusCode::OK => Ok(()),
        _ => Err(response
            .json::<B2ErrorBody>()
            .await
            .map_err(Network)?
            .into()),
    }
}
