use crate::middleware::UserSession;
use crate::search2::search2;
use crate::search2::SearchParams;
use actix_web::get;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::web::Query;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use bb8_postgres::bb8::Pool;
use bb8_postgres::tokio_postgres::NoTls;
use bb8_postgres::PostgresConnectionManager;
use bwcommon::insert_extension;
use bwcommon::ApiSpecificInfoForLogging;
use serde_json::json;

async fn handler(
    req: HttpRequest,
    query: String,
    query_params: Query<SearchParams>,
    pool: Data<Pool<PostgresConnectionManager<NoTls>>>,
) -> Result<impl Responder, bwcommon::MyError> {
    let query_params = query_params.into_inner();
    let allow_nsfw = req.extensions().get::<UserSession>().is_some();

    match query_params.sort.as_str() {
        "relevancy" | "scenario" | "lastmodifiedold" | "lastmodifiednew" | "timeuploadedold"
        | "timeuploadednew" => {}
        _ => {
            return Ok(HttpResponse::BadRequest().finish());
        }
    }

    let maps = search2(query.as_str(), allow_nsfw, &query_params, pool.clone()).await?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&json!({
            "total_results": maps.0,
            "maps": maps.1,
        }))?))
}

#[get("/api/uiv2/search/{query}")]
async fn search_query(
    req: HttpRequest,
    query: Path<String>,
    query_params: Query<SearchParams>,
    pool: Data<Pool<PostgresConnectionManager<NoTls>>>,
) -> Result<impl Responder, bwcommon::MyError> {
    handler(req, query.into_inner(), query_params, pool).await
}

#[get("/api/uiv2/search")]
async fn search(
    req: HttpRequest,
    query_params: Query<SearchParams>,
    pool: Data<Pool<PostgresConnectionManager<NoTls>>>,
) -> Result<impl Responder, bwcommon::MyError> {
    handler(req, "".to_owned(), query_params, pool).await
}
