use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use bwcommon::insert_extension;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;

#[get("/api/uiv2/timestamps/{map_id}")]
async fn timestamps(
    // req: HttpRequest,
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

    let pool = pool.clone();
    let con = pool.get().await?;

    let filetimes: Vec<i64> = con
        .query(
            "select distinct modified_time
            from map
            join filetime on filetime.map = map.id
            where map.id = $1
            order by modified_time",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| anyhow::Ok(row.try_get("modified_time")?))
        .collect::<Result<Vec<_>, _>>()?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&filetimes)?)
        .customize())
}
