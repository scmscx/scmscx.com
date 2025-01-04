use crate::api::bulkupload::insert_map;
use crate::middleware::UserSession;
use actix_web::post;
use actix_web::web;
use actix_web::HttpMessage;
use actix_web::HttpResponse;
use actix_web::Responder;
use bwcommon::insert_extension;
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
struct UploadQuery {
    filename: String,
    sha256: String,
    last_modified: i64,
    length: usize,
    playlist: String,
}

#[post("/api/uiv2/upload-map")]
async fn upload_map(
    query: web::Query<UploadQuery>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    req: actix_web::HttpRequest,
    mut payload: actix_web::web::Payload,
) -> Result<impl Responder, MyError> {
    let user_id = req
        .extensions()
        .get::<UserSession>()
        .map(|x| x.id)
        .unwrap_or(10);

    let query = query.into_inner();

    tokio::fs::create_dir_all("./tmp").await?;

    let fake_filename = format!("./tmp/{}.scx", uuid::Uuid::new_v4().as_simple());

    let mut sha256hasher = Sha256::new();
    let mut sha1hasher = Sha1::new();
    let mut total_file_size = 0;

    info!("Starting read payload");

    {
        let mut file = tokio::fs::File::create(fake_filename.as_str()).await?;

        while let Some(chunk) = payload.next().await {
            let bytes = chunk?;
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

    let pool = (**pool).clone();

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

    info!("insert map");
    let map_id = insert_map(
        query.filename.as_str(),
        fake_filename.as_str(),
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

    info!("renaming");
    tokio::fs::rename(&fake_filename, format!("./pending/{sha1hash}-{sha256hash}")).await?;

    let map_id = bwcommon::get_web_id_from_db_id(map_id, crate::util::SEED_MAP_ID)?;

    info!("responding");
    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&json!(map_id))?)
        .customize())
}

// let (tx, mut rx) = tokio::sync::mpsc::channel::<Bytes>(1);

// let join_handle = {
//     let client = Client::new();
//     //const MAPBLOB_BUCKET: &'static str = "784baffe8e56dc107ee50d1c";
//     const TEST_BUCKET_2: &'static str = "082baf7e0e563c508ef50d1c";

//     let mut retries_left = 5;
//     let mut bad_version = None;

//     let upload_info = loop {
//         retries_left -= 1;

//         let (version, api_info) =
//             crate::actix::get_auth(&client, backblaze_auth.clone(), bad_version).await?;

//         let upload_info = match b2_get_upload_url(&client, &api_info, TEST_BUCKET_2).await {
//             Ok(upload_info) => upload_info,
//             Err(e) => {
//                 error!("Failed to get upload url, trying again: {e}");
//                 bad_version = Some(version);
//                 tokio::time::sleep(std::time::Duration::from_millis(500)).await;
//                 continue;
//             }
//         };

//         break upload_info;
//     };

//     let sha256hash = query.sha256.clone();
//     let sha1hash = query.sha1.clone();
//     let total_file_size = query.length;

//     tokio::task::spawn(async move {
//         let sm = stream! {
//             while let Some(bytes) = rx.recv().await {
//                 error!("yielding bytes: {}", bytes.len());
//                 yield anyhow::Ok(bytes);
//             }
//         };

//         b2_upload_file(
//             &client,
//             &upload_info,
//             sha256hash.as_str(),
//             total_file_size,
//             sha1hash.clone(),
//             sm,
//         )
//         .await
//     })
// };
