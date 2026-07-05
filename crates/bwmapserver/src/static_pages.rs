use std::path::PathBuf;

use axum::http::header;
use axum::response::{IntoResponse, Response};

async fn serve_html_file(path: PathBuf) -> Result<Response, bwcommon::MyError> {
    let body = tokio::fs::read(path).await?;
    Ok(([(header::CONTENT_TYPE, "text/html; charset=utf-8")], body).into_response())
}

pub async fn redirect_map() -> Result<Response, bwcommon::MyError> {
    let root = std::path::Path::new(std::env::var("ROOT_DIR")?.as_str()).join("public");
    let new_path = root.join("map-redirect.html");

    serve_html_file(new_path).await
}

pub async fn redirect_replay() -> Result<Response, bwcommon::MyError> {
    let root = std::path::Path::new(std::env::var("ROOT_DIR")?.as_str()).join("public");
    let new_path = root.join("replay-redirect.html");

    serve_html_file(new_path).await
}
