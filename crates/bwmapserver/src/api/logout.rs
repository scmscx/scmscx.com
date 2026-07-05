use axum::http::{header, HeaderValue, StatusCode};
use axum::response::Response;

use crate::webutil::{append_cookie, removal_cookie};

pub async fn handler() -> Result<Response, bwcommon::MyError> {
    let info = bwcommon::ApiSpecificInfoForLogging::default();

    let mut resp = bwcommon::with_logging_info(info, StatusCode::TEMPORARY_REDIRECT);
    resp.headers_mut()
        .insert(header::LOCATION, HeaderValue::from_static("/"));
    append_cookie(&mut resp, removal_cookie("username"));
    append_cookie(&mut resp, removal_cookie("token"));
    Ok(resp)
}
