use crate::api::bulkupload::{insert_parsed_map, parse_map};
use crate::webutil::{MaybeUser, Pool};
use axum::body::Body;
use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::with_logging_info;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;
use futures_util::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use sha1::Digest;
use sha1::Sha1;
use sha2::Sha256;
use std::collections::HashMap;
use std::env;
use tokio::io::AsyncWriteExt;
use tracing::info;

// const url = `/api/uiv2/upload-map?${new URLSearchParams({
//     filename: file.name,
//     sha1: sha1hash,
//     sha256: sha256hash,
//     lastModified: `${file.lastModified}`,
//     length: `${file.size}`,
//   })}`;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct UploadQuery {
    filename: String,
    sha256: String,
    last_modified: i64,
    length: usize,
    playlist: String,
}

pub async fn upload_map(
    Query(query): Query<UploadQuery>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
    body: Body,
) -> Result<Response, MyError> {
    if env::var("SCMSCX_READONLY").unwrap_or_else(|_| "false".to_owned()) == "true" {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            "server is in maintenance mode, try again later.",
        )
            .into_response());
    }

    let user_id = user.id().unwrap_or(10);

    tokio::fs::create_dir_all("./pending/tmp").await?;
    tokio::fs::create_dir_all("./pending/backblaze").await?;
    tokio::fs::create_dir_all("./pending/gsfs").await?;
    let fake_filename = format!("./pending/tmp/{}.scx", uuid::Uuid::new_v4().as_simple());

    let mut sha256hasher = Sha256::new();
    let mut sha1hasher = Sha1::new();
    let mut total_file_size = 0;

    info!("Starting read payload");

    {
        let mut file = tokio::fs::File::create(fake_filename.as_str()).await?;

        let mut stream = body.into_data_stream();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| anyhow::anyhow!("error reading body: {e}"))?;
            total_file_size += bytes.len();

            bwcommon::ensure!(total_file_size <= query.length);

            sha1hasher.update(&bytes[..]);
            sha256hasher.update(&bytes[..]);

            file.write_all(&bytes[..]).await?;
        }

        // file.sync_all().await?;
        file.flush().await?;
    }

    let sha256hash = format!("{:x}", sha256hasher.finalize());
    let sha1hash = format!("{:x}", sha1hasher.finalize());

    bwcommon::ensure!(sha256hash == query.sha256);
    bwcommon::ensure!(total_file_size == query.length);

    // Parse + validate the map up front so we reject garbage early
    let parsed = parse_map(fake_filename.clone()).await?;
    if parsed.insert.is_none() {
        return Ok(Json(json!(-1)).into_response());
    }

    info!("playlist");
    let playlist_id: i64 = {
        let con = pool.get().await?;

        if let Some(row) = con
            .query(
                "select id from playlist where name = $1 and owner = $2",
                &[&query.playlist, &user_id],
            )
            .await?
            .pop()
        {
            row.get("id")
        } else {
            con.query_one(
                "insert into playlist (owner, name) values ($1, $2) returning id",
                &[&user_id, &query.playlist],
            )
            .await?
            .get("id")
        }
    };

    let mut new_tags = HashMap::new();
    new_tags.insert("autogen_uploaded".to_owned(), "v3".to_owned());

    // Stage the mapblob for delivery *before* inserting the map row, so the map
    // only becomes visible to the renderer once its blob is durably queued for
    // gsfs/backblaze.

    // backblaze
    {
        info!("copying mpq for backblaze");
        let fake_filename2 = format!("./pending/tmp/{}", uuid::Uuid::new_v4().as_simple());
        tokio::fs::copy(&fake_filename, fake_filename2.as_str()).await?;
        tokio::fs::rename(
            fake_filename2,
            format!("./pending/backblaze/{sha1hash}-{sha256hash}"),
        )
        .await?;
    }

    // gsfs
    {
        info!("copying mpq for gsfs");
        let fake_filename2 = format!("./pending/tmp/{}", uuid::Uuid::new_v4().as_simple());
        tokio::fs::copy(&fake_filename, fake_filename2.as_str()).await?;
        tokio::fs::rename(fake_filename2, format!("./pending/gsfs/{sha256hash}")).await?;
    }

    info!("insert map");
    let map_id = insert_parsed_map(
        parsed,
        query.filename.as_str(),
        sha256hash.as_str(),
        total_file_size,
        user_id,
        playlist_id,
        new_tags,
        pool,
        Some(query.last_modified / 1000),
    )
    .await?;
    let info = ApiSpecificInfoForLogging {
        map_id: Some(map_id),
        ..Default::default()
    };

    info!("removing temp file");
    tokio::fs::remove_file(&fake_filename).await?;

    let map_id = bwcommon::get_web_id_from_db_id(map_id, crate::util::SEED_MAP_ID)?;

    info!("responding");
    Ok(with_logging_info(info, Json(json!(map_id))))
}
