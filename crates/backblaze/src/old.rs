// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize)]
// pub struct B2StartLargeFileResponse {
//     pub fileId: String,
// }
// pub async fn b2_start_large_file(
//     client: &Client,
//     api_info: &B2AuthorizeAccount,
//     bucket_id: &str,
//     file_name: &str,
// ) -> Result<B2StartLargeFileResponse> {
//     let response = client
//         .post(format!(
//             "{}{}",
//             api_info.apiUrl, "/b2api/v2/b2_start_large_file"
//         ))
//         .header("Authorization", &api_info.authorizationToken)
//         .body(serde_json::json!({ "bucketId": bucket_id, "fileName": file_name, "contentType": "application/octet-stream" }).to_string())
//         .send()
//         .await?;
//     if response.status() != 200 {
//         return Err(anyhow::anyhow!(response.text().await?));
//     } else {
//         anyhow::Ok(response.json::<B2StartLargeFileResponse>().await?)
//     }
// }

// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize)]
// pub struct B2FinishLargeFileResponse {
//     pub fileId: String,
// }
// pub async fn b2_finish_large_file(
//     client: &Client,
//     api_info: &B2AuthorizeAccount,
//     large_file_response: &B2StartLargeFileResponse,
//     sha1_array: &[String],
// ) -> Result<B2FinishLargeFileResponse> {
//     let response = client
//         .post(format!(
//             "{}{}",
//             api_info.apiUrl, "/b2api/v2/b2_finish_large_file"
//         ))
//         .header("Authorization", &api_info.authorizationToken)
//         .body(
//             serde_json::json!({ "fileId": large_file_response.fileId, "partSha1Array": sha1_array }).to_string(),
//         )
//         .send()
//         .await?;
//     if response.status() != 200 {
//         return Err(anyhow::anyhow!(response.text().await?));
//     } else {
//         anyhow::Ok(response.json::<B2FinishLargeFileResponse>().await?)
//     }
// }

// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize)]
// pub struct B2GetUploadPartUrlResponse {
//     pub fileId: String,
//     pub uploadUrl: String,
//     pub authorizationToken: String,
// }
// pub async fn b2_get_upload_part_url(
//     client: &Client,
//     api_info: &B2AuthorizeAccount,
//     large_file_response: &B2StartLargeFileResponse,
// ) -> Result<B2GetUploadPartUrlResponse> {
//     let response = client
//         .post(format!(
//             "{}{}",
//             api_info.apiUrl, "/b2api/v2/b2_get_upload_part_url"
//         ))
//         .header("Authorization", &api_info.authorizationToken)
//         .body(serde_json::json!({ "fileId": large_file_response.fileId }).to_string())
//         .send()
//         .await?;
//     if response.status() != 200 {
//         return Err(anyhow::anyhow!(response.text().await?));
//     } else {
//         anyhow::Ok(response.json::<B2GetUploadPartUrlResponse>().await?)
//     }
// }

// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize)]
// pub struct B2UploadPartResponse {
//     pub contentSha1: String,
// }
// pub async fn b2_upload_part(
//     client: &Client,
//     upload_part_info: &B2GetUploadPartUrlResponse,
//     part_number: usize,
//     data: Vec<u8>,
// ) -> Result<B2UploadPartResponse> {
//     use sha1::Digest;
//     let mut sha1_hasher = sha1::Sha1::new();
//     sha1_hasher.update(data.as_slice());
//     let sha1_hash = format!("{:x}", sha1_hasher.finalize());

//     let response = client
//         .post(&upload_part_info.uploadUrl)
//         .header("Authorization", &upload_part_info.authorizationToken)
//         .header("X-Bz-Content-Sha1", &sha1_hash)
//         .header("Content-Length", data.len())
//         .header("Content-Type", "application/octet-stream")
//         .header("X-Bz-Part-Number", format!("{part_number}"))
//         .body(data)
//         .send()
//         .await?;

//     let response = if response.status() != 200 {
//         return Err(anyhow::anyhow!(response.text().await?));
//     } else {
//         anyhow::Ok(response.json::<B2UploadPartResponse>().await?)
//     }?;

//     anyhow::ensure!(response.contentSha1 == sha1_hash);

//     anyhow::Ok(response)
// }

// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize)]
// pub struct B2ListFileVersionResponse {
//     pub nextFileId: Option<String>,
//     pub files: Vec<B2ListFileVersionResponseFile>,
// }
// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize)]
// pub struct B2ListFileVersionResponseFile {
//     pub fileId: String,
//     pub fileName: String,
// }
// pub async fn list_files_in_bucket(
//     client: &Client,
//     api_info: &B2AuthorizeAccount,
//     bucket_id: &str,
//     start_file_id: Option<&str>,
// ) -> Result<B2ListFileVersionResponse> {
//     let response = client
//         .post(format!(
//             "{}{}",
//             api_info.apiUrl, "/b2api/v2/b2_list_file_versions"
//         ))
//         .header("Authorization", &api_info.authorizationToken)
//         .body(
//             serde_json::json!({
//                 "bucketId": bucket_id,
//                 "startFileId": start_file_id,
//                 "maxFileCount": 10000,
//             })
//             .to_string(),
//         )
//         .send()
//         .await?;

//     if response.status() != 200 {
//         return Err(anyhow::anyhow!(response.text().await?));
//     } else {
//         anyhow::Ok(response.json::<B2ListFileVersionResponse>().await?)
//     }
// }

// pub async fn delete_file(
//     client: &Client,
//     api_info: &B2AuthorizeAccount,
//     file_name: &str,
//     file_id: &str,
// ) -> Result<()> {
//     let response = client
//         .post(format!(
//             "{}{}",
//             api_info.apiUrl, "/b2api/v2/b2_delete_file_version"
//         ))
//         .header("Authorization", &api_info.authorizationToken)
//         .body(
//             serde_json::json!({
//                 "fileName": file_name,
//                 "fileId": file_id,
//             })
//             .to_string(),
//         )
//         .send()
//         .await?;

//     if response.status() != 200 {
//         return Err(anyhow::anyhow!(response.text().await?));
//     }

//     anyhow::Ok(())
// }

// pub async fn download_file(
//     client: &Client,
//     api_info: &B2AuthorizeAccount,
//     file_id: &str,
// ) -> Result<Bytes> {
//     let response = client
//         .post(format!(
//             "{}{}",
//             api_info.apiUrl, "/b2api/v2/b2_download_file_by_id"
//         ))
//         .header("Authorization", &api_info.authorizationToken)
//         .body(
//             serde_json::json!({
//                 "fileId": file_id,
//             })
//             .to_string(),
//         )
//         .send()
//         .await?;

//     if response.status() != 200 {
//         return Err(anyhow::anyhow!(response.text().await?));
//     }

//     anyhow::Ok(response.bytes().await?)
// }

// pub async fn download_stream(
//     client: &Client,
//     api_info: &B2AuthorizeAccount,
//     file_id: &str,
// ) -> Result<impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>> {
//     let response = client
//         .post(format!(
//             "{}{}",
//             api_info.apiUrl, "/b2api/v2/b2_download_file_by_id"
//         ))
//         .header("Authorization", &api_info.authorizationToken)
//         .body(
//             serde_json::json!({
//                 "fileId": file_id,
//             })
//             .to_string(),
//         )
//         .send()
//         .await?;

//     if response.status() != 200 {
//         return Err(anyhow::anyhow!(response.text().await?));
//     }

//     anyhow::Ok(response.bytes_stream())
// }
