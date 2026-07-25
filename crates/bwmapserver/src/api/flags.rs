use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;

use crate::access;
use crate::webutil::{MaybeUser, Pool, PoolExt};

/// Whitelist of flag column names that callers are allowed to read/write.
/// Returning `&'static str` (the literal, not the caller's borrow) keeps the
/// value safe to interpolate into SQL.
fn validate_flag(flag: &str) -> Option<&'static str> {
    Some(match flag {
        "nsfw" => "nsfw",
        "unfinished" => "unfinished",
        "outdated" => "outdated",
        "broken" => "broken",
        "blackholed" => "blackholed",
        "spoiler_unit_names" => "spoiler_unit_names",
        _ => return None,
    })
}

pub async fn get_flag(
    user: MaybeUser,
    Path((map_id, flag)): Path<(String, String)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(column) = validate_flag(&flag) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // The blackhole gate rides along on the flag read instead of costing a second
    // checkout via `access::map_is_hidden`: a map page asks for six flags, so the
    // extra round trip would be paid six times per view. The alias matters —
    // `blackholed` is itself one of the readable flags, and `select blackholed,
    // blackholed` would otherwise be ambiguous to `try_get` by name.
    let con = pool.acquire().await?;
    let statement = format!("select {column} as value, blackholed from map where map.id = $1");
    let Some(row) = con.query_opt(&statement, &[&map_id]).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if access::blackholed_is_hidden_from(row.try_get("blackholed")?, user.session()) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    Ok(Json(row.try_get::<_, bool>("value")?).into_response())
}

pub async fn set_flag(
    user: MaybeUser,
    Path((map_id, flag)): Path<(String, String)>,
    Extension(pool): Extension<Pool>,
    Json(info): Json<bool>,
) -> Result<Response, MyError> {
    if std::env::var("SCMSCX_READONLY").unwrap_or_else(|_| "false".to_owned()) == "true" {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            "server is in maintenance mode, try again later.".to_owned(),
        )
            .into_response());
    }

    // Anonymous is 401 here rather than the 403 `may_modify_map` would produce
    // below: "log in" and "you may not touch this map" are different answers.
    let Some(session) = user.session() else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(column) = validate_flag(&flag) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let mut con = pool.acquire().await?;
    let checked = info;

    let statement = format!("update map set {column} = $1 where map.id = $2");

    let tx = con.transaction().await?;

    let owner: Option<i64> = tx
        .query_opt(
            "select uploaded_by from map where map.id = $1 for update",
            &[&map_id],
        )
        .await?
        .map(|r| r.try_get::<_, i64>(0))
        .transpose()?;

    let Some(owner) = owner else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if !access::may_modify_map(owner, Some(session)) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    tx.execute(&statement, &[&checked, &map_id]).await?;
    tx.commit().await?;

    Ok(StatusCode::OK.into_response())
}
