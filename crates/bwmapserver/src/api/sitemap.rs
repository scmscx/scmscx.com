use axum::extract::Extension;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::webutil::Pool;

pub async fn handler() -> Result<Response, bwcommon::MyError> {
    let mut s = String::new();

    s.push_str("https://scmscx.com/\n");
    s.push_str("https://scmscx.com/search\n");
    s.push_str("https://scmscx.com/about\n");
    s.push_str("https://scmscx.com/recent\n");
    s.push_str("https://scmscx.com/login\n");
    s.push_str("https://scmscx.com/register\n");

    Ok(([(header::CONTENT_TYPE, "text/plain")], s).into_response())
}

pub async fn handlera(Extension(pool): Extension<Pool>) -> Result<Response, bwcommon::MyError> {
    let con = pool.get().await?;
    let ids: Vec<i64> = con.query(
            "select id from map where nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false and chkblob is not null order by id limit 50000 OFFSET 0",
            &[],
        ).await?.into_iter().map(|row| {
            anyhow::Ok(row.try_get::<_, i64>(0)?)
        }).collect::<Result<Vec<_>, _>>()?;

    let mut s = String::new();

    for i in ids {
        s.push_str(
            format!(
                "https://scmscx.com/map/{}\n",
                bwcommon::get_web_id_from_db_id(i, crate::util::SEED_MAP_ID)?
            )
            .as_str(),
        );
    }

    Ok(([(header::CONTENT_TYPE, "text/plain")], s).into_response())
}

pub async fn handlerb(Extension(pool): Extension<Pool>) -> Result<Response, bwcommon::MyError> {
    let con = pool.get().await?;
    let ids: Vec<i64> = con.query(
            "select id from map where nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false and chkblob is not null order by id limit 50000 OFFSET 50000",
            &[],
        ).await?.into_iter().map(|row| {
            anyhow::Ok(row.try_get::<_, i64>(0)?)
        }).collect::<Result<Vec<_>, _>>()?;

    let mut s = String::new();

    for i in ids {
        s.push_str(
            format!(
                "https://scmscx.com/map/{}\n",
                bwcommon::get_web_id_from_db_id(i, crate::util::SEED_MAP_ID)?
            )
            .as_str(),
        );
    }

    Ok(([(header::CONTENT_TYPE, "text/plain")], s).into_response())
}

pub async fn handlerc(Extension(pool): Extension<Pool>) -> Result<Response, bwcommon::MyError> {
    let con = pool.get().await?;
    let ids: Vec<i64> = con.query(
            "select id from map where nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false and chkblob is not null order by id limit 50000 OFFSET 100000",
            &[],
        ).await?.into_iter().map(|row| {
            anyhow::Ok(row.try_get::<_, i64>(0)?)
        }).collect::<Result<Vec<_>, _>>()?;

    let mut s = String::new();

    for i in ids {
        s.push_str(
            format!(
                "https://scmscx.com/map/{}\n",
                bwcommon::get_web_id_from_db_id(i, crate::util::SEED_MAP_ID)?
            )
            .as_str(),
        );
    }

    Ok(([(header::CONTENT_TYPE, "text/plain")], s).into_response())
}
