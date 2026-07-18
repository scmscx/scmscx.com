use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;

use crate::webutil::{MaybeUser, Pool};

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
    _user: MaybeUser,
    Path((map_id, flag)): Path<(String, String)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(column) = validate_flag(&flag) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let con = pool.get().await?;
    let statement = format!("select {column} from map where map.id = $1");
    let checked: bool = con.query_one(&statement, &[&map_id]).await?.try_get(0)?;

    Ok(Json(checked).into_response())
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

    let Some(user_id) = user.id() else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(column) = validate_flag(&flag) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let mut con = pool.get().await?;
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

    if owner != user_id && user_id != 4 {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    tx.execute(&statement, &[&checked, &map_id]).await?;
    tx.commit().await?;

    Ok(StatusCode::OK.into_response())
}
