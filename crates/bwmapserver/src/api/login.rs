use std::sync::Arc;

use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::ratelimit::UsernameLoginLimiter;
use crate::util::is_dev_mode;
use crate::webutil::{append_cookie, auth_cookie, Pool};

#[derive(Deserialize)]
pub(crate) struct LoginFormData {
    username: String,
    password: String,
}

pub async fn post_handler(
    Extension(pool): Extension<Pool>,
    Extension(username_limiter): Extension<Arc<UsernameLoginLimiter>>,
    Json(form): Json<LoginFormData>,
) -> Result<Response, bwcommon::MyError> {
    if form.username.is_empty() || form.username.len() > 100 {
        return Ok((
            StatusCode::UNAUTHORIZED,
            "Either the username does not exist or the password is incorrect.",
        )
            .into_response());
    }

    // Username only appears in the JSON body, so this can't be enforced as middleware.
    if let Err(resp) = username_limiter.check(&form.username) {
        return Ok(*resp);
    }

    match crate::db::login(form.username.clone(), form.password.clone(), pool).await {
        Ok(token) => {
            let info = bwcommon::ApiSpecificInfoForLogging {
                username: Some(form.username.clone()),
                ..Default::default()
            };

            let secure = !is_dev_mode();
            let mut resp = bwcommon::with_logging_info(info, StatusCode::OK);
            append_cookie(&mut resp, auth_cookie("token", token, secure, true));
            append_cookie(
                &mut resp,
                auth_cookie("username", form.username.clone(), secure, false),
            );
            Ok(resp)
        }
        Err(_) => Ok((
            StatusCode::UNAUTHORIZED,
            "Either the username does not exist or the password is incorrect.",
        )
            .into_response()),
    }
}
