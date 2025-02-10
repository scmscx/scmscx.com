use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use bwcommon::insert_extension;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;
use bwmap::ParsedChk;
use serde_json::json;

#[get("/api/uiv2/units/{map_id}")]
async fn units(
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

    let (chkblob, spoiler_unit_names) = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select length, ver, data, spoiler_unit_names
                from map
                join chkblob on chkblob.hash = map.chkblob
                where map.id = $1
                ",
                &[&map_id],
            )
            .await?;

        let length = row.try_get::<_, i64>("length")? as usize;
        let ver = row.try_get::<_, i64>("ver")?;
        let data = row.try_get::<_, Vec<u8>>("data")?;

        bwcommon::ensure!(ver == 1);
        (
            zstd::bulk::decompress(data.as_slice(), length)?,
            row.try_get::<_, bool>("spoiler_unit_names")?,
        )
    };

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let units = if let Ok(x) = &parsed_chk.unix {
        let mut v = Vec::new();

        for unit_id in 0..x.config.len() {
            if x.config[unit_id] == 0 && x.string_number[unit_id] != 0 {
                v.push(json!({
                    "unit_id": unit_id,
                    "name": if spoiler_unit_names { "SPOILER".to_owned()  } else {
                        parsed_chk.get_string(x.string_number[unit_id] as usize)
                        .unwrap_or("couldn't decode string".to_owned())
                        },
                }));
            }
        }

        v
    } else if let Ok(x) = &parsed_chk.unis {
        let mut v = Vec::new();

        for unit_id in 0..x.config.len() {
            if x.config[unit_id] == 0 && x.string_number[unit_id] != 0 {
                v.push(json!({
                    "unit_id": unit_id,
                    "name": parsed_chk.get_string(x.string_number[unit_id] as usize)
                        .unwrap_or("couldn't decode string".to_owned()),
                }));
            }
        }

        v
    } else {
        Vec::new()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&units).unwrap())
        .customize())
}
