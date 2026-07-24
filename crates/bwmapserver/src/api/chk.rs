use crate::db;
use crate::webutil::Pool;
use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;
use bwmap::ParsedChk;
use common::gsfs::gsfs_get_map_image;
use tracing::error;

pub async fn get_chk_strings(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;

        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), pool.clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let refs = parsed_chk.get_all_string_references()?;

    let mut strings = Vec::new();

    for r in refs {
        strings.push(
            parsed_chk
                .get_string(r as usize)
                .unwrap_or_else(|_| ">>> could not get string <<<<".to_owned()),
        );
    }

    Ok(Json(strings).into_response())
}

pub async fn get_chk_riff_chunks(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;

        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), pool.clone()).await?;

    let raw_chunks = bwmap::parse_riff(chkblob.as_slice());

    Ok(Json(raw_chunks).into_response())
}

pub async fn get_chk_json(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;
        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), pool.clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    Ok(Json(parsed_chk).into_response())
}

pub async fn get_chk_trig_json(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;
        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), pool.clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let trigs = bwmap::parse_triggers(&parsed_chk);

    Ok(Json(trigs).into_response())
}

pub async fn get_chk_mbrf_json(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;
        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), pool.clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let trigs = bwmap::parse_mission_briefing(&parsed_chk);

    Ok(Json(trigs).into_response())
}

pub async fn get_eups(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;
        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), pool.clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    if let Ok(unit_section) = parsed_chk.unit {
        let eups: Vec<_> = unit_section
            .units
            .iter()
            .filter(|x| x.owner > 12 || x.unit_id > 227)
            .collect();
        Ok(Json(eups).into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

pub async fn download_chk(
    Path((chkhash,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let chkblob = db::get_chk(chkhash.clone(), pool.clone()).await?;

    Ok((
        [(header::CONTENT_TYPE, "application/octet-stream")],
        chkblob,
    )
        .into_response())
}

pub async fn get_map_img(
    Extension(reqwest_client): Extension<reqwest::Client>,
    Path((chk_hash,)): Path<(String,)>,
) -> Result<Response, MyError> {
    if let Ok(endpoint) = std::env::var("GSFSFE_ENDPOINT") {
        match tokio::time::timeout(
            std::time::Duration::from_secs(1),
            gsfs_get_map_image(&reqwest_client, &endpoint, chk_hash.as_str()),
        )
        .await
        {
            Ok(Ok(stream)) => {
                return Ok((
                    [
                        (header::CONTENT_TYPE, "image/webp"),
                        (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                    ],
                    Body::from_stream(stream),
                )
                    .into_response());
            }
            Ok(Err(error)) => {
                error!("Failed to get mapimg from gsfs: {}", error);
            }
            Err(e) => {
                error!("Timed out trying to get mapimg from gsfs: {}", e);
            }
        }
    }

    Ok((StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-cache")]).into_response())
}
