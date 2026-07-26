use crate::access;
use crate::db;
use crate::middleware::UserSession;
use crate::webutil::{MaybeUser, Pool, PoolExt};
use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;
use bwmap::ParsedChk;
use common::gsfs::gsfs_get_map_image;
use tracing::error;

/// A map's chk data, or `None` when the caller must be answered `404`.
///
/// A map that does not exist and a map that is blackholed are deliberately
/// indistinguishable here: blackholing is meant to look exactly like deletion, so
/// both cases collapse to the same 404 rather than one 404 and one 500.
///
/// Every `get_chk_*` handler wants the blob rather than the hash, so the fetch
/// lives here too and each handler is left with one `else { 404 }`.
async fn visible_chkblob(
    pool: &Pool,
    map_id: i64,
    user: Option<&UserSession>,
) -> Result<Option<Vec<u8>>, MyError> {
    let chkhash = {
        let con = pool.acquire().await?;
        let Some(row) = con
            .query_opt(
                "select map.chkblob, map.blackholed from map where map.id = $1",
                &[&map_id],
            )
            .await?
        else {
            return Ok(None);
        };

        if access::blackholed_is_hidden_from(row.try_get("blackholed")?, user) {
            return Ok(None);
        }

        row.try_get::<_, String>("chkblob")?
    };

    Ok(Some(db::get_chk(chkhash, pool.clone()).await?))
}

pub async fn get_chk_strings(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(chkblob) = visible_chkblob(&pool, map_id, user.session()).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
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
    user: MaybeUser,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(chkblob) = visible_chkblob(&pool, map_id, user.session()).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let raw_chunks = bwmap::parse_riff(chkblob.as_slice());

    Ok(Json(raw_chunks).into_response())
}

pub async fn get_chk_json(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(chkblob) = visible_chkblob(&pool, map_id, user.session()).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    Ok(Json(parsed_chk).into_response())
}

pub async fn get_chk_trig_json(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(chkblob) = visible_chkblob(&pool, map_id, user.session()).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let trigs = bwmap::parse_triggers(&parsed_chk);

    Ok(Json(trigs).into_response())
}

pub async fn get_chk_mbrf_json(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(chkblob) = visible_chkblob(&pool, map_id, user.session()).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let trigs = bwmap::parse_mission_briefing(&parsed_chk);

    Ok(Json(trigs).into_response())
}

pub async fn get_eups(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(chkblob) = visible_chkblob(&pool, map_id, user.session()).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
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
    user: MaybeUser,
) -> Result<Response, MyError> {
    // Same gate as the minimap routes: a chk is reachable through any map that
    // references it, so it inherits the most restrictive flag among them. The
    // `restricted` flag is unused here because this response sets no
    // `Cache-Control` and so inherits `no-store` — never shared-cacheable.
    if let Err(status) = access::check_chk_access(&pool, &chkhash, user.session())
        .await?
        .restricted_or_refusal()
    {
        return Ok(status.into_response());
    }

    let chkblob = db::get_chk(chkhash, pool.clone()).await?;

    Ok((
        [(header::CONTENT_TYPE, "application/octet-stream")],
        chkblob,
    )
        .into_response())
}

pub async fn get_map_img(
    Extension(reqwest_client): Extension<reqwest::Client>,
    Extension(pool): Extension<Pool>,
    Path((chk_hash,)): Path<(String,)>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    // `restricted` is deliberately ignored: unlike a minimap, a full-resolution
    // image is only reachable by knowing the exact chk hash and asking for it, so
    // a shared cache copy doesn't put it in front of anyone who wasn't already
    // going to fetch it. These stay long-lived and publicly cacheable even for a
    // blackholed or NSFW map — do not "fix" this to match `get_minimap`.
    if let Err(status) = access::check_chk_access(&pool, &chk_hash, user.session())
        .await?
        .restricted_or_refusal()
    {
        return Ok((status, [(header::CACHE_CONTROL, "no-cache")]).into_response());
    }

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
