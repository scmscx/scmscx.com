use crate::middleware::UserSession;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use bwcommon::MyError;
use bwcommon::{get_db_id_from_web_id, insert_extension, ApiSpecificInfoForLogging};
use tracing::info;

#[get("/api/flags/{map_id}/{flag}")]
async fn get_flag(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>,
) -> Result<impl Responder, MyError> {
    let user_id = req.extensions().get::<UserSession>().map(|x| x.id);

    let (map_id, flag) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let con = pool.get().await?;
    info!("flag: {}", flag);

    let statement = match flag.as_str() {
        "nsfw" => "select nsfw from map where map.id = $1",
        "unfinished" => "select unfinished from map where map.id = $1",
        "outdated" => "select outdated from map where map.id = $1",
        "broken" => "select broken from map where map.id = $1",
        "blackholed" => "select blackholed from map where map.id = $1",
        "spoiler_unit_names" => "select spoiler_unit_names from map where map.id = $1",
        _ => return Ok(HttpResponse::NotFound().finish()),
    };

    let checked: bool = con.query_one(statement, &[&map_id]).await?.try_get(0)?;

    let info = ApiSpecificInfoForLogging {
        user_id: user_id,
        map_id: Some(map_id),
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&checked)?))
}

#[post("/api/flags/{map_id}/{flag}")]
async fn set_flag(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    info: web::Json<bool>,
    pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>,
) -> Result<impl Responder, MyError> {
    let user_id = if let Some(user_id) = req.extensions().get::<UserSession>().map(|x| x.id) {
        user_id
    } else {
        return Ok(HttpResponse::Unauthorized().finish());
    };

    let (map_id, flag) = path.into_inner();

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let con = pool.get().await?;
    let checked = *info;
    info!("flag: {}", flag);

    let statement = match flag.as_str() {
        "nsfw" => "update map set nsfw = $1 where map.id = $2 and (map.uploaded_by = $3 or $3 = 4)",
        "unfinished" => {
            "update map set unfinished = $1 where map.id = $2 and (map.uploaded_by = $3 or $3 = 4)"
        }
        "outdated" => {
            "update map set outdated = $1 where map.id = $2 and (map.uploaded_by = $3 or $3 = 4)"
        }
        "broken" => {
            "update map set broken = $1 where map.id = $2 and (map.uploaded_by = $3 or $3 = 4)"
        }
        "blackholed" => {
            "update map set blackholed = $1 where map.id = $2 and (map.uploaded_by = $3 or $3 = 4)"
        }
        "spoiler_unit_names" => {
            "update map set spoiler_unit_names = $1 where map.id = $2 and (map.uploaded_by = $3 or $3 = 4)"
        }
        _ => return Ok(HttpResponse::NotFound().finish()),
    };

    con.execute(statement, &[&checked, &map_id, &user_id])
        .await?;

    let info = ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        map_id: Some(map_id),
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info).finish())
}
