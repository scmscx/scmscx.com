use crate::middleware::UserSession;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use bwcommon::MyError;
use bwcommon::{insert_extension, ApiSpecificInfoForLogging};

/// Whitelist of flag column names that callers are allowed to read/write.
/// Returning `&'static str` (the literal, not the caller's borrow) keeps the
/// value safe to interpolate into SQL.
fn validate_flag(flag: &str) -> Option<&'static str> {
    Some(match flag {
        "nsfw" => "nsfw",
        "unfinished" => "unfinished",
        "outdated" => "outdated",
        "broken" => "broken",
        "blackholed" => "blackholed",
        "spoiler_unit_names" => "spoiler_unit_names",
        _ => return None,
    })
}

#[get("/api/flags/{map_id}/{flag}")]
async fn get_flag(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>,
) -> Result<impl Responder, MyError> {
    let user_id = req.extensions().get::<UserSession>().map(|x| x.id);

    let (map_id, flag) = path.into_inner();

    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(column) = validate_flag(&flag) else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let con = pool.get().await?;
    let statement = format!("select {column} from map where map.id = $1");
    let checked: bool = con.query_one(&statement, &[&map_id]).await?.try_get(0)?;

    let info = ApiSpecificInfoForLogging {
        user_id,
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
    if std::env::var("SCMSCX_READONLY").unwrap_or_else(|_| "false".to_owned()) == "true" {
        return Ok(HttpResponse::ServiceUnavailable()
            .body("server is in maintenance mode, try again later.".to_owned()));
    }

    let Some(user_id) = req.extensions().get::<UserSession>().map(|x| x.id) else {
        return Ok(HttpResponse::Unauthorized().finish());
    };

    let (map_id, flag) = path.into_inner();

    let map_id = crate::util::parse_map_id(&map_id)?;

    let Some(column) = validate_flag(&flag) else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let mut con = pool.get().await?;
    let checked = *info;

    let statement = format!("update map set {column} = $1 where map.id = $2");

    let tx = con.transaction().await?;

    let owner: Option<i64> = tx
        .query_opt(
            "select uploaded_by from map where map.id = $1 for update",
            &[&map_id],
        )
        .await?
        .map(|r| r.try_get::<_, i64>(0))
        .transpose()?;

    let info = ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        map_id: Some(map_id),
        ..Default::default()
    };

    let Some(owner) = owner else {
        return Ok(insert_extension(HttpResponse::NotFound(), info).finish());
    };

    if owner != user_id && user_id != 4 {
        return Ok(insert_extension(HttpResponse::Forbidden(), info).finish());
    }

    tx.execute(&statement, &[&checked, &map_id]).await?;
    tx.commit().await?;

    Ok(insert_extension(HttpResponse::Ok(), info).finish())
}
