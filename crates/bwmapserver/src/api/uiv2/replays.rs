use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use bwcommon::insert_extension;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;

#[get("/api/uiv2/replays/{map_id}")]
async fn replays(
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

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct ReplayInfo {
        id: i64,
        frames: i64,
        time_saved: i64,
        creator: String,
    }

    let replays: Vec<ReplayInfo> = con
        .query(
            "
        select replay.id, replay.denorm_frames, replay.denorm_time_saved, replay.denorm_game_creator
        from replay
        join map on map.chkblob = replay.chkhash
        where map.id = $1
        order by replay.denorm_frames",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|r| {
            anyhow::Ok(ReplayInfo {
                id: r.try_get("id")?,
                frames: r.try_get("denorm_frames")?,
                time_saved: r.try_get("denorm_time_saved")?,
                creator: encoding_rs::UTF_8
                    .decode(r.try_get::<_, Vec<u8>>("denorm_game_creator")?.as_slice())
                    .0
                    .to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&replays)?)
        .customize())
}
