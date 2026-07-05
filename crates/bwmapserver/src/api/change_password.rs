use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::with_logging_info;

use crate::webutil::{MaybeUser, Pool};

#[derive(serde::Deserialize)]
pub(crate) struct ChangePasswordPostData {
    password: String,
    password_confirm: String,
}

async fn handler2(
    user: MaybeUser,
    form: ChangePasswordPostData,
    pool: Pool,
) -> Result<Response, bwcommon::MyError> {
    let Some(user_id) = user.id() else {
        return Ok((
            StatusCode::UNAUTHORIZED,
            "Unauthorized. Try logging in first/again.",
        )
            .into_response());
    };

    if form.password != form.password_confirm {
        return Ok((
            StatusCode::BAD_REQUEST,
            "The two provided passwords must match",
        )
            .into_response());
    }

    if form.password.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "The provided password must not be the empty string",
        )
            .into_response());
    }

    if form.password.len() > 100 {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Why would you try to create a password that long",
        )
            .into_response());
    }

    crate::db::change_password(user_id, form.password.clone(), pool).await?;

    let info = bwcommon::ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        ..Default::default()
    };

    Ok(with_logging_info(info, "Password changed successfully"))
}

pub async fn post_handler(
    user: MaybeUser,
    Extension(pool): Extension<Pool>,
    Json(form): Json<ChangePasswordPostData>,
) -> Result<Response, bwcommon::MyError> {
    if std::env::var("SCMSCX_READONLY").unwrap_or_else(|_| "false".to_owned()) == "true" {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            "server is in maintenance mode, try again later.".to_owned(),
        )
            .into_response());
    }

    handler2(user, form, pool).await
}
