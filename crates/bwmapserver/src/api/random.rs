// select map.id as id from (
//     select distinct map as id
//     from stringmap2
//     where $1 <% data and ((scenario_name = true and $3) or (scenario_description = true and $4) or (unit_names = true and $5) or (force_names = true and $6) or (file_names = true and $7))
// ) as sq
// join map on map.id = sq.id
// where ($2 or map.nsfw = false) and outdated = false and unfinished = false and broken = false
// order by random()
// limit 1

use crate::middleware::UserSession;
use crate::search2::{search_cache, SearchParams};
use actix_web::web::Path;
use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use bwcommon::{insert_extension, ApiSpecificInfoForLogging};
use rand::Rng;

async fn random_core(
    query: &str,
    req: HttpRequest,
    query_params: web::Query<SearchParams>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<String, bwcommon::MyError> {
    let query_params = query_params.into_inner();
    let allow_nsfw = req.extensions().get::<UserSession>().is_some();

    match query_params.sort.as_str() {
        "relevancy" | "scenario" | "lastmodifiedold" | "lastmodifiednew" | "timeuploadedold"
        | "timeuploadednew" => {}
        _ => {
            return Err(anyhow::anyhow!("unknown sort").into());
        }
    }

    let maps = search_cache(query, allow_nsfw, &query_params, (**pool).clone()).await?;

    if maps.len() == 0 {
        return Err(anyhow::anyhow!("no maps found").into());
    }

    let mut rng = rand::rng();
    let random_number = rng.random_range(0..maps.len());

    Ok(maps[random_number].id.clone())
}

#[get("/api/uiv2/random/{query}")]
async fn handler(
    query: Path<String>,
    req: HttpRequest,
    query_params: web::Query<SearchParams>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let query = query.into_inner();

    let map_id = random_core(query.as_str(), req, query_params, pool).await?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .body(serde_json::to_string(&map_id)?)
        .customize())
}

#[get("/api/uiv2/random")]
async fn handler_noquery(
    req: HttpRequest,
    query_params: web::Query<SearchParams>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let map_id = random_core("", req, query_params, pool).await?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .body(serde_json::to_string(&map_id)?)
        .customize())
}
