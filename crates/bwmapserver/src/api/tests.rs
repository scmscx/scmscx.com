use actix_web::{get, web, HttpResponse, Responder};
use bwcommon::MyError;

#[get("/api/tests/all_maps")]
async fn get_all_maps(
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, MyError> {
    let hashes = {
        let con = pool.get().await?;
        let rows = con.query("select hash from mapblob", &[]).await?;

        rows.into_iter()
            .map(|x| anyhow::Ok(x.try_get::<_, String>(0)?))
            .collect::<Result<Vec<String>, _>>()?
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&hashes).unwrap()))
}
