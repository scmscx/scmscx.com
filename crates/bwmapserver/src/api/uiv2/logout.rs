use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::webutil::{append_cookie, removal_cookie};

pub async fn logout2() -> Result<Response, bwcommon::MyError> {
    let mut resp = StatusCode::TEMPORARY_REDIRECT.into_response();
    resp.headers_mut()
        .insert(header::LOCATION, HeaderValue::from_static("/uiv2"));
    append_cookie(&mut resp, removal_cookie("username"));
    append_cookie(&mut resp, removal_cookie("token"));
    Ok(resp)
}
