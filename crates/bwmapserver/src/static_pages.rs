use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::Result;
use actix_web::{get, Responder};

fn do_named_file_stuff(path: PathBuf) -> Result<impl Responder, bwcommon::MyError> {
    Ok(NamedFile::open(path)?
        .use_last_modified(true)
        .use_etag(false)
        .disable_content_disposition())
}

#[get("/map")]
pub async fn redirect_map() -> Result<impl Responder, bwcommon::MyError> {
    let root = std::path::Path::new(std::env::var("ROOT_DIR")?.as_str()).join("public");
    let new_path = root.join("map-redirect.html");

    do_named_file_stuff(new_path)
}

#[get("/replay")]
pub async fn redirect_replay() -> Result<impl Responder, bwcommon::MyError> {
    let root = std::path::Path::new(std::env::var("ROOT_DIR")?.as_str()).join("public");
    let new_path = root.join("replay-redirect.html");

    do_named_file_stuff(new_path)
}
