use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use bwcommon::insert_extension;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;

#[get("/api/uiv2/filenames/{map_id}")]
async fn filenames(
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
    let filenames: Vec<String> = con
        .query(
            "select filename.filename
            from mapfilename
            join filename on mapfilename.filename = filename.id
            where mapfilename.map = $1",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| anyhow::Ok(row.try_get(0)?))
        .collect::<Result<Vec<_>, _>>()?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&filenames)?)
        .customize())
}
