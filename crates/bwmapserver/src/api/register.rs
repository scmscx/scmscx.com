use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::webutil::{append_cookie, auth_cookie, Pool};

#[derive(serde::Deserialize)]
pub(crate) struct RegisterFormData {
    username: String,
    password: String,
    password_confirm: String,
}

async fn handler2(pool: Pool, form: RegisterFormData) -> Result<Response, bwcommon::MyError> {
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

    if let Ok(token) = crate::db::register(form.username.clone(), form.password.clone(), pool).await
    {
        let info = bwcommon::ApiSpecificInfoForLogging {
            username: Some(form.username.clone()),
            ..Default::default()
        };

        let mut resp = bwcommon::with_logging_info(info, StatusCode::OK);
        append_cookie(&mut resp, auth_cookie("token", token, true, true));
        append_cookie(
            &mut resp,
            auth_cookie("username", form.username.clone(), true, false),
        );
        Ok(resp)
    } else {
        Ok((StatusCode::UNAUTHORIZED, "Could not register account").into_response())
    }
}

pub async fn post_handler(
    Extension(pool): Extension<Pool>,
    Json(form): Json<RegisterFormData>,
) -> Result<Response, bwcommon::MyError> {
    if std::env::var("SCMSCX_READONLY").unwrap_or_else(|_| "false".to_owned()) == "true" {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            "server is in maintenance mode, try again later.",
        )
            .into_response());
    }

    handler2(pool, form).await
}
