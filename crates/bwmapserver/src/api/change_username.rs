use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::webutil::{append_cookie, auth_cookie, MaybeUser, Pool};

#[derive(serde::Deserialize)]
pub(crate) struct ChangeUsernamePostData {
    username: String,
    username_confirm: String,
    password: String,
}

async fn handler2(
    user: MaybeUser,
    pool: Pool,
    form: ChangeUsernamePostData,
) -> Result<Response, bwcommon::MyError> {
    let Some(user_id) = user.id() else {
        return Ok((
            StatusCode::UNAUTHORIZED,
            "Unauthorized. Try logging in first/again.",
        )
            .into_response());
    };

    if form.username.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "The provided username must not be the empty string",
        )
            .into_response());
    }

    if form.username.len() > 100 {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Why would you try to create a username that long",
        )
            .into_response());
    }

    if form.username != form.username_confirm {
        return Ok((StatusCode::BAD_REQUEST, "The provided usernames must match").into_response());
    }

    let is_password_correct =
        crate::db::check_password(user_id, form.password.clone(), pool.clone()).await?;

    if !is_password_correct {
        return Ok((StatusCode::BAD_REQUEST, "Provided password is incorrect.").into_response());
    }

    crate::db::change_username(user_id, form.username.clone(), form.password.clone(), pool).await?;

    let info = bwcommon::ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        ..Default::default()
    };

    let mut resp = bwcommon::with_logging_info(info, "Username changed successfully");
    append_cookie(
        &mut resp,
        auth_cookie("username", form.username.clone(), true, false),
    );
    Ok(resp)
}

pub async fn post_handler(
    user: MaybeUser,
    Extension(pool): Extension<Pool>,
    Json(form): Json<ChangeUsernamePostData>,
) -> Result<Response, bwcommon::MyError> {
    if std::env::var("SCMSCX_READONLY").unwrap_or_else(|_| "false".to_owned()) == "true" {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            "server is in maintenance mode, try again later.",
        )
            .into_response());
    }

    handler2(user, pool, form).await
}
