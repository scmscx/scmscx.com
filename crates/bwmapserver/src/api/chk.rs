use crate::db;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use bwcommon::MyError;
use bwmap::ParsedChk;

#[get("/api/chk/strings/{map_id}")]
async fn get_chk_strings(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;

        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), (**pool).clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let refs = parsed_chk.get_all_string_references()?;

    let mut strings = Vec::new();

    for r in refs {
        strings.push(
            parsed_chk
                .get_string(r as usize)
                .unwrap_or(">>> could not get string <<<<".to_owned()),
        );
    }

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&strings).unwrap()))
}

#[get("/api/chk/riff_chunks/{map_id}")]
async fn get_chk_riff_chunks(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;

        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), (**pool).clone()).await?;

    let raw_chunks = bwmap::parse_riff(chkblob.as_slice());

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&raw_chunks).unwrap()))
}

#[get("/api/chk/json/{map_id}")]
async fn get_chk_json(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;
        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), (**pool).clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&parsed_chk).unwrap()))
}

#[get("/api/chk/trig/{map_id}")]
async fn get_chk_trig_json(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;
        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), (**pool).clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let trigs = bwmap::parse_triggers(&parsed_chk);

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&trigs).unwrap()))
}

#[get("/api/chk/mbrf/{map_id}")]
async fn get_chk_mbrf_json(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;
        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), (**pool).clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let trigs = bwmap::parse_mission_briefing(&parsed_chk);

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&trigs).unwrap()))
}

#[get("/api/chk/eups/{map_id}")]
async fn get_eups(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let (map_id,) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let chkhash = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select map.chkblob from map
                where map.id = $1",
                &[&map_id],
            )
            .await?;
        row.try_get::<_, String>(0)?
    };

    let chkblob = db::get_chk(chkhash.clone(), (**pool).clone()).await?;
    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    if let Ok(unit_section) = parsed_chk.unit {
        let eups: Vec<_> = unit_section
            .units
            .iter()
            .filter(|x| x.owner > 12 || x.unit_id > 227)
            .collect();
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&eups).unwrap()))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[get("/api/chk/{chk_hash}")]
async fn download_chk(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let (chkhash,) = path.into_inner();

    let chkblob = db::get_chk(chkhash.clone(), (**pool).clone()).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(chkblob))
}
