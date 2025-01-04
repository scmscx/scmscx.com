use crate::actix::BackblazeAuth;
use crate::middleware::UserSession;
use actix_multipart::Field;
use actix_web::{post, web, HttpMessage, HttpRequest};
use actix_web::{HttpResponse, Responder};
use anyhow::anyhow;
use anyhow::Result;
use async_stream::stream;
use backblaze::api::{b2_get_upload_url, b2_upload_file, B2GetUploadUrl};
use bb8_postgres::tokio_postgres::IsolationLevel;
use bwmap::ParsedChk;
use bytes::BytesMut;
use futures::lock::Mutex;
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use rand::Rng;
use reqwest::Client;
use serde::Serialize;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::error;

#[post("/api/upload-maps/{playlist}")]
pub(crate) async fn post_handler(
    req: HttpRequest,
    path: web::Path<(String,)>,
    mut payload: actix_multipart::Multipart,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    backblaze_auth: web::Data<Mutex<BackblazeAuth>>,
) -> Result<impl Responder, bwcommon::MyError> {
    let user_id = req
        .extensions()
        .get::<UserSession>()
        .map(|x| x.id)
        .unwrap_or(10);

    let (playlist,) = path.into_inner();

    let playlist_id: i64 = {
        let con = pool.get().await?;

        if let Some(row) = con
            .query(
                "select id from playlist where name = $1 and owner = $2",
                &[&playlist, &user_id],
            )
            .await?
            .pop()
        {
            row.get("id")
        } else {
            con.query_one(
                "insert into playlist (owner, name) values ($1, $2) returning id",
                &[&user_id, &playlist],
            )
            .await?
            .get("id")
        }
    };

    let client = Client::new();

    // const ARCHIVE_BUCKET: &'static str = "b80b4f2e8e26dc107ee50d1c";
    // const UPLOAD_BUCKET: &'static str = "38cb8fae7e062c108e550d1c";
    const MAPBLOB_BUCKET: &'static str = "784baffe8e56dc107ee50d1c";

    let (_, api_info) = crate::actix::get_auth(&client, backblaze_auth.clone(), None).await?;

    let upload_info = b2_get_upload_url(&client, &api_info, MAPBLOB_BUCKET)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut ret = Vec::new();

    #[derive(Debug, Serialize)]
    enum Ret {
        Link(String),
        Err(String),
    }

    while let Ok(Some(field)) = payload.try_next().await {
        ret.push(
            match process_field(
                &client,
                &upload_info,
                user_id,
                playlist_id,
                (**pool).clone(),
                field,
            )
            .await
            {
                Ok(link) => Ret::Link(link),
                Err(err) => Ret::Err(err.to_string()),
            },
        );
    }

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&ret)?)
        .customize())
}

async fn process_field(
    client: &Client,
    upload_info: &B2GetUploadUrl,
    user_id: i64,
    playlist_id: i64,
    pool: bb8_postgres::bb8::Pool<
        bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
    >,
    mut field: Field,
) -> Result<String> {
    let content_disposition = field
        .content_disposition()
        .ok_or(anyhow!("content_disposition not found"))?;

    match content_disposition.disposition {
        actix_web::http::header::DispositionType::FormData => {}
        _ => anyhow::bail!(
            "content disposition is invalid: content_disposition{:?}",
            content_disposition
        ),
    }

    let mut last_modified_time = Err(anyhow!("last_modified_time not found"));
    let mut file_size = Err(anyhow!("file_size not found"));
    let mut file_name = Err(anyhow!("file_name not found"));
    let mut file_sha256hash = Err(anyhow!("file_sha256hash not found"));
    let mut file_sha1hash = Err(anyhow!("file_sha1hash not found"));

    for param in &content_disposition.parameters {
        match param {
            actix_web::http::header::DispositionParam::Name(s) => {
                let split: Vec<&str> = s.split("/").collect();
                if split.len() != 4 {
                    continue;
                }

                if let Ok(x) = split[0].parse::<i64>() {
                    last_modified_time = Ok(x / 1000);
                }

                if let Ok(x) = split[1].parse::<usize>() {
                    file_size = Ok(x);
                }

                file_sha256hash = Ok(split[2].to_owned());
                file_sha1hash = Ok(split[3].to_owned());
            }
            actix_web::http::header::DispositionParam::Filename(s) => {
                let split = s.split("/");
                file_name = Ok(split.last().unwrap().to_owned());
            }
            actix_web::http::header::DispositionParam::FilenameExt(_) => {}
            actix_web::http::header::DispositionParam::Unknown(_, _) => {}
            actix_web::http::header::DispositionParam::UnknownExt(_, _) => {}
        }
    }

    let last_modified_time = last_modified_time?;
    let file_size = file_size?;
    let file_name = file_name?;
    let file_sha256hash = file_sha256hash?;
    let file_sha1hash = file_sha1hash?;

    let fake_filename = format!("/tmp/{}", uuid::Uuid::new_v4().as_simple());

    let mut sha256hasher = Sha256::new();
    let mut sha1hasher = Sha1::new();

    {
        let mut file = tokio::fs::File::create(fake_filename.as_str()).await?;
        let mut total_file_size = 0;

        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(bytes) => {
                    total_file_size += bytes.len();
                    if total_file_size > file_size {
                        anyhow::bail!("file too big. total_file_size: {total_file_size}, file_size: {total_file_size}");
                    }

                    sha1hasher.update(&bytes[..]);
                    sha256hasher.update(&bytes[..]);
                    file.write_all(&bytes[..]).await?;
                }
                Err(err) => anyhow::bail!("err: {:?}", err),
            }
        }

        if total_file_size != file_size {
            anyhow::bail!("incorrect file size. total_file_size: {total_file_size}, file_size: {total_file_size}");
        }

        file.sync_all().await?;
        file.flush().await?;
    }

    let sha256hash = format!("{:x}", sha256hasher.finalize());
    let sha1hash = format!("{:x}", sha1hasher.finalize());

    anyhow::ensure!(sha256hash == *file_sha256hash);
    anyhow::ensure!(sha1hash == *file_sha1hash);

    let sm = {
        let fake_filename = fake_filename.clone();

        stream! {
            let mut file = tokio::fs::File::open(fake_filename.as_str()).await?;

            loop {
                let mut buf = BytesMut::with_capacity(1024*1024);
                let bytes_read = file.read_buf(&mut buf).await?;
                if bytes_read == 0 {
                    break;
                }

                yield anyhow::Ok(buf);
            }
        }
    };

    b2_upload_file(
        &client,
        &upload_info,
        sha256hash.as_str(),
        file_size,
        sha1hash.clone(),
        sm,
    )
    .await?;

    let mut new_tags = HashMap::new();
    new_tags.insert("autogen_bulk_uploaded2".to_owned(), "true".to_owned());

    let map_id = insert_map(
        file_name.as_str(),
        fake_filename.as_str(),
        sha256hash.as_str(),
        file_size,
        user_id,
        playlist_id,
        new_tags,
        pool,
        Some(last_modified_time),
    )
    .await?;

    tokio::fs::remove_file(fake_filename).await?;

    let map_id = bwcommon::get_web_id_from_db_id(map_id, crate::util::SEED_MAP_ID)?;

    anyhow::Ok(format!("https://scmscx.com/map/{}", map_id))
}

// seventyseven-uploadCreated:December 17, 2022
// Bucket ID:38cb8fae7e062c108e550d1c
// Type:Private
// File Lifecycle:Keep all versions
// Snapshots:0
// Current Files:0
// Current Size:0 bytes
// Endpoint:s3.us-west-004.backblazeb2.com
// Encryption:Disabled
#[allow(clippy::too_many_arguments)]
pub(crate) async fn insert_map(
    filename: &str,
    mpq_path: &str,
    mpq_blob_hash: &str,
    mpq_blob_len: usize,
    user_id: i64,
    playlist_id: i64,
    add_tags: std::collections::HashMap<String, String>,
    pool: bb8_postgres::bb8::Pool<
        bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
    >,
    modified_time: Option<i64>,
) -> Result<i64, anyhow::Error> {
    let filename = filename.to_string();

    let chk_blob = bwmpq::get_chk_from_mpq_filename(mpq_path)?;
    let parsed_chk = ParsedChk::from_bytes(chk_blob.as_slice());

    let chk_blob_hash = {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&chk_blob);
        format!("{:x}", hasher.finalize())
    };

    let sprp_section = if let Ok(x) = &parsed_chk.sprp {
        x
    } else {
        return Err(parsed_chk.sprp.unwrap_err());
    };

    match chk_blob_hash.as_str() {
        "15abadff92e85d0b7d25dd42dd80b149d78ac84bb8c3ec049de49d822e670135"
        | "f1cb1f46b6f6b45fdd8dfa683b17ade1717025e86d96395313fa9859aa36ab2d"
        | "424926781a5d1a679ebf90343c876acc70f6b5697c2093f591acbc3ff5ecc997"
        | "946c52c587ca9a84c927ccb64355c3983e078f11f14e40ce82f33c8a4b4fca53"
        | "294843003eb5a554294a2e6605295d944f34e84888c328291794afc8a78522b3"
        | "d5149f373ea5cb8130a79412a3f1d377685749f1a49ab2264c75b526f34c39db"
        | "2e87022c1d1b0dab0aa784593531f16801318913f6e90566c0444cd42692c9e9"
        | "10fd7657d24038e3c76c0f6a38e8dbf44558245564934e402c37e9b53e42a2c0"
        | "e47f7cb44d6c190b8a640c2708e340ccfc8b6be4e910ea12bd2b5bd5912b8fae"
        | "a05425842683296ba41a507c5d97cb126ae6d11e066a9f927167bbec283fefde"
        | "3a550d6a787a91f68bf33b233ead858926e84b45dc217ec1ebfdf2c04150d2f4"
        | "ebeffd8f677b345289667d2700c92c6d69795bd1c1c3aa52d8287a542a72ad7b"
        | "6c81a80495be17f3fbbb06569973a167f1caf20daa777ae4db51990bc8a8df43"
        | "e8839041d71d6588e67303d4ec156a421c611e155cd61afef3a6a48cec635137" => {
            return Ok(-1);
        }
        _ => {}
    }

    // calculate denormalized stuff
    let scenario_name = sanitize_sc_scenario_string(
        parsed_chk
            .get_string(*sprp_section.scenario_name_string_number as usize)?
            .as_str(),
    );

    // compress chk
    let chk_blob_compressed = zstd::bulk::compress(chk_blob.as_slice(), 15)?;

    // get now time:
    let time_since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    // begin db stuff
    async fn retry_n_times<T, E, R>(n: usize, f: impl Fn() -> R) -> Result<T, anyhow::Error>
    where
        E: Debug,
        R: futures::Future<Output = Result<T, E>>,
    {
        for _ in 0..n {
            match f().await {
                Ok(x) => return Ok(x),
                Err(e) => {
                    error!("failed to attempt transaction: error: {:?}", e);
                    tokio::time::sleep(Duration::from_millis(
                        rand::thread_rng().gen_range(300..2000),
                    ))
                    .await;
                }
            }
        }

        Err(anyhow::anyhow!("failed {} times", n))
    }

    let f = || async {
        let mut con = pool.get().await?;
        let mut tx = con
            .build_transaction()
            .isolation_level(IsolationLevel::Serializable)
            .start()
            .await?;

        // https://stackoverflow.com/questions/40878027/detect-if-the-row-was-updated-or-inserted/40880200#40880200
        // TODO: detect if row was inserted or updated.
        // using xmax it can be done.

        tx.execute("INSERT INTO chkblob (hash, ver, length, data) VALUES ($1, 1, $2, $3) ON CONFLICT DO NOTHING RETURNING Cast((xmax = 0) as boolean) AS inserted",
        &[&chk_blob_hash, &(chk_blob.len() as i64), &chk_blob_compressed]).await?;

        let map_was_inserted = tx.execute("
        INSERT INTO map (uploaded_by, uploaded_time, denorm_scenario, chkblob, mapblob2, mapblob_size)
        VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING",
        &[&user_id, &time_since_epoch, &scenario_name, &chk_blob_hash, &mpq_blob_hash, &(mpq_blob_len as i64)]).await? > 0;

        let map_id = tx
            .query_one("SELECT id from map where mapblob2 = $1", &[&mpq_blob_hash])
            .await?
            .try_get::<_, i64>("id")?;

        tx.execute(
            "INSERT INTO Filename (filename) VALUES ($1) ON CONFLICT DO NOTHING",
            &[&filename],
        )
        .await?;
        let filename_id = tx
            .query_one("SELECT id from filename where filename = $1", &[&filename])
            .await?
            .try_get::<_, i64>("id")?;

        tx.execute(
            "INSERT INTO MapFilename (filename, map) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            &[&filename_id, &map_id],
        )
        .await?;

        tx.execute("INSERT INTO filetime (map, accessed_time, modified_time, creation_time) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING", &[&map_id, &0i64, &modified_time, &0i64]).await?;

        tx.execute("INSERT INTO filenames2 (map_id, filename_id, modified_time) VALUES ($1, $2, to_timestamp($3)) ON CONFLICT DO NOTHING", &[&map_id, &filename_id, &modified_time.map(|x| x as f64)]).await?;

        if map_was_inserted {
            for kv in &add_tags {
                let tag_id = tx
                    .query_one(
                        "insert into tag (key, value) values ($1, $2) RETURNING id",
                        &[kv.0, kv.1],
                    )
                    .await?
                    .try_get::<_, i64>(0)?;

                tx.execute(
                    "insert into tagmap (map, tag) values ($1, $2)",
                    &[&map_id, &tag_id],
                )
                .await?;
            }
        }

        tx.execute(
            "INSERT into playlistmap (playlist, map) values ($1, $2)",
            &[&playlist_id, &map_id],
        )
        .await?;

        bwcommon::denormalize_map_tx(map_id, &mut tx).await?;

        tx.commit().await?;

        anyhow::Ok(map_id)
    };

    let map_id = retry_n_times(7, f).await?;

    anyhow::Ok(map_id)
}

fn sanitize_sc_scenario_string(s: &str) -> String {
    // split string by left or right marks

    let mut strings: Vec<_> = s.split(|x| x == '\u{0012}' || x == '\u{0013}').collect();

    strings.sort_by_key(|x| std::cmp::Reverse(x.len()));

    if strings.len() == 0 {
        String::new()
    } else {
        strings[0].to_string()
    }
}

// pub(crate) async fn process_stream_async_concurrent<I, T, F, J, R, F2, H, Z>(
//     mut iter: I,
//     cloner: H,
//     max_outstanding: usize,
//     on_item_completed: F2,
//     func: F,
// ) -> usize
// where
//     I: Stream<Item = T> + Unpin,

//     F: Fn(Z, T) -> R,
//     R: futures::Future<Output = J> + Send,
//     F2: Fn(usize, J),
//     H: Fn() -> Z,
// {
//     let mut futs = Vec::new();
//     let mut counter = 0;
//     loop {
//         while futs.len() < max_outstanding {
//             if let Some(entry) = iter.next() {
//                 futs.push(func(cloner(), entry).boxed());
//             } else {
//                 break;
//             }
//         }

//         if futs.len() == 0 {
//             break;
//         }

//         let (item, _, remaining_futures) = futures::future::select_all(futs).await;

//         futs = remaining_futures;

//         counter += 1;

//         on_item_completed(counter, item);
//     }

//     counter
// }
