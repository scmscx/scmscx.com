use actix_web::get;
use actix_web::web;
use futures_util::FutureExt;

use crate::middleware::UserSession;
use log::info;

use actix_web::HttpMessage;
use anyhow::Result;

// use sha2::Digest;

// #[get("/api/meme")]
// pub(crate) async fn hacks_meme(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     if let Some(user_id) = bwcommon::check_auth2(&req, pool.clone()).await {
//         if user_id != 4 {
//             return Ok(actix_web::HttpResponse::Unauthorized().finish());
//         }
//     } else {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     };

//     let mut con = pool.get()?;

//     let replay_ids = con
//         .query("select id, hash from replay where chkhash is null", &[])?
//         .iter()
//         .map(|r| (r.get::<_, i64>(0), r.get::<_, String>(1)))
//         .collect::<Vec<(i64, String)>>();

//     for replay in replay_ids {
//         println!("replay: {}", replay.0);
//         let data = con
//             .query_one("select data from replayblob where hash = $1", &[&replay.1])?
//             .get::<_, Vec<u8>>(0);

//         if let Ok(parsed_replay) = bwreplay::parse_replay_blob(data.as_slice()) {
//             let hashed_chk = {
//                 let mut hasher = Sha256::new();
//                 hasher.update(parsed_replay.chk_data);
//                 format!("{:x}", hasher.finalize())
//             };
//             println!("replay: {}, chk_hash: {}", replay.0, hashed_chk);

//             con.execute(
//                 "update replay set chkhash = $1 where id = $2",
//                 &[&hashed_chk, &replay.0],
//             )?;
//         }
//     }

//     Ok(actix_web::HttpResponse::Ok().body("ok"))
// }

// #[get("/api/convert_filenames")]
// pub(crate) async fn hacks_convert_filenames(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     if let Some(user_id) = bwcommon::check_auth2(&req, pool.clone()).await {
//         if user_id != 4 {
//             return Ok(actix_web::HttpResponse::Unauthorized().finish());
//         }
//     } else {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     };

//     let mut con = pool.get()?;

//     let replay_ids = con
//         .query(
//             "select id, filename from filename where filename like '%&#%;%'",
//             &[],
//         )?
//         .iter()
//         .map(|r| (r.get::<_, i64>(0), r.get::<_, String>(1)))
//         .collect::<Vec<(i64, String)>>();

//     for filename in replay_ids {
//         let re = regex::Regex::new(r"&#([0-9]+);").unwrap();

//         let mut new_str = filename.1.clone();

//         for i in re.captures_iter(filename.1.as_str()) {
//             if let Some(c) = i.get(1) {
//                 let mut x = c.as_str().parse::<u64>().unwrap();

//                 let b: &mut Vec<u8> = &mut Vec::new();
//                 while x != 0 {
//                     (*b).push((x % 256) as u8);
//                     x = x / 256;
//                 }
//                 let mut v = Vec::new();
//                 let mut v_rev = Vec::new();
//                 v.extend(b.as_slice().iter());
//                 v_rev.extend(b.as_slice().iter().rev());

//                 let _utf8 = encoding_rs::UTF_8.decode(v.as_slice()).0.to_string();
//                 let utf8_rev = encoding_rs::UTF_8.decode(v_rev.as_slice()).0.to_string();
//                 // let utf16le = encoding_rs::UTF_16LE.decode(v.as_slice()).0.to_string();
//                 // let utf16le_rev = encoding_rs::UTF_16LE.decode(v_rev.as_slice()).0.to_string();
//                 // let windows1252 = encoding_rs::WINDOWS_1252.decode(v.as_slice()).0.to_string();
//                 // let windows1252_rev = encoding_rs::WINDOWS_1252.decode(v_rev.as_slice()).0.to_string();

//                 // let windows1252_1 = encoding_rs::WINDOWS_1252.decode(&v.as_slice()[0..1]).0.to_string();
//                 // let windows1252_2 = encoding_rs::WINDOWS_1252.decode(&v.as_slice()[1..2]).0.to_string();

//                 //println!("{}, {:?}, {:?}, utf8: {}, utf8_rev: {}, utf16le: {}, utf16le_rev: {}", copy, v, v_rev, utf8, utf8_rev, utf16le, utf16le_rev);

//                 // let utf16_le = encoding_rs::UTF_16LE.decode(entry.file_name().as_bytes()).0.to_string();
//                 // let utf16_be = encoding_rs::UTF_16BE.decode(entry.file_name().as_bytes()).0.to_string();
//                 // let windows_1250 = encoding_rs::WINDOWS_1250.decode(entry.file_name().as_bytes()).0.to_string();
//                 // let utf8 = encoding_rs::UTF_8.decode(entry.file_name().as_bytes()).0.to_string();

//                 //if !utf16le.contains('�') {
//                 //new_str = new_str.replace(format!("&#{};", c.as_str()).as_str(), format!("{}{}", windows1252_1, windows1252_2).as_str());
//                 new_str = new_str.replace(format!("&#{};", c.as_str()).as_str(), utf8_rev.as_str());
//                 // } else {
//                 //     new_str = new_str.replace(format!("&#{};", c.as_str()).as_str(), windows1252_rev.as_str());
//                 // }
//             }
//         }
//         //println!("orig: {}", filename.1);
//         println!("id: {}, tr_: {}", filename.0, new_str);

//         //con.execute("update filename set filename = $1 where id = $2", &[&new_str, &filename.0]).unwrap_or(0);

//         // if the filename is already existing then it can be remapped like this.
//         //let new_id = con.query_one("select id from filename where filename = $1", &[&new_str])?.get::<_, i64>(0);

//         //con.execute("update mapfilename set filename = $1 where filename = $2", &[&new_id, &filename.0])?;

//         // let re = regex::Regex::new(r"\s+").unwrap();

//         // let mut scenario_name = chk.scenario_name.clone();
//         // scenario_name = re.replace_all(scenario_name.as_str(), " ").to_string();
//         // scenario_name = scenario_name.chars().filter(|r|
//         //     *r >= 32 as char
//         // ).collect();

//         // let data = con.query_one("select data from replayblob where hash = $1", &[&replay.1])?.get::<_, Vec<u8>>(0);

//         // if let Ok(parsed_replay) = replay::parse_replay_blob(data.as_slice()) {
//         //     let hashed_chk = {
//         //         let mut hasher = Sha256::new();
//         //         hasher.update(parsed_replay.chk_data);
//         //         format!("{:x}", hasher.finalize())
//         //     };
//         //     println!("replay: {}, chk_hash: {}", replay.0, hashed_chk);

//         //     con.execute("update replay set chkhash = $1 where id = $2", &[&hashed_chk, &replay.0])?;
//         // }
//     }

//     Ok(actix_web::HttpResponse::Ok().body("ok"))
// }

// #[get("/api/cache_minimaps")]
// pub(crate) async fn cache_minimaps(
//     req: actix_web::HttpRequest,
//     tileset_maps: web::Data<
//         std::collections::HashMap<u32, std::collections::HashMap<u16, [u8; 3]>>,
//     >,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     if let Some(user_id) = bwcommon::check_auth2(&req, pool.clone()).await {
//         if user_id != 4 {
//             return Ok(actix_web::HttpResponse::Unauthorized().finish());
//         }
//     } else {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     }

//     let hashes = {
//         let pool = pool.clone();
//         web::block(move || {
//             let mut con = pool.get()?;

//             let hashes: Result<Vec<_>, _> = con
//                 .query(
//                     "select hash
//                 from chkblob
//                 except
//                 select chkhash from minimap",
//                     &[],
//                 )?
//                 .into_iter()
//                 .map(|x| x.try_get::<_, String>(0))
//                 .collect();

//             anyhow::Ok(hashes?)
//         })
//         .await??
//     };

//     for hashes in hashes.chunks(32) {
//         let futures: Vec<_> = hashes
//             .iter()
//             .map(|hash| {
//                 let pool = pool.clone();
//                 let hash = hash.clone();
//                 db::get_chk(hash.clone(), (**pool).clone()).then(|chk_data| {
//                     let raw_chunks = bwmap::parse_chk(chk_data.unwrap().as_slice());
//                     let merged_chunks = bwmap::merge_rawchunks(raw_chunks.as_slice());
//                     let chk = bwmap::get_parsed_chk(&merged_chunks).unwrap();
//                     let minimap = bwcommon::render_minimap(
//                         &chk.mtxm.as_slice(),
//                         chk.map_width as usize,
//                         chk.map_height as usize,
//                         chk.era as usize,
//                         tileset_maps.as_ref(),
//                     )
//                     .unwrap();

//                     println!("{:?}", hash);

//                     web::block(move || {
//                         let mut con = pool.get()?;

//                         con.execute(
//                             "INSERT INTO minimap
//                         (chkhash, width, height, minimap) VALUES
//                         ($1, $2, $3, $4)",
//                             &[
//                                 &hash,
//                                 &(chk.map_width as i32),
//                                 &(chk.map_height as i32),
//                                 &minimap,
//                             ],
//                         )?;

//                         anyhow::Ok(())
//                     })
//                 })
//             })
//             .collect();

//         futures::future::join_all(futures.into_iter()).await;
//     }

//     Ok(actix_web::HttpResponse::Ok().finish())
//}

#[get("/api/denormalize/{map_id}")]
pub(crate) async fn denormalize(
    path: web::Path<(String,)>,
    req: actix_web::HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl actix_web::Responder, bwcommon::MyError> {
    let session = if let Some(session) = req.extensions().get::<UserSession>() {
        session.clone()
    } else {
        return Ok(actix_web::HttpResponse::NotFound().finish());
    };

    if session.id != 4 {
        return Ok(actix_web::HttpResponse::NotFound().finish());
    }

    let (map_id,) = path.into_inner();
    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let mut con = pool.get().await?;
    let mut tx = con.transaction().await?;

    bwcommon::denormalize_map_tx(map_id, &mut tx).await?;

    tx.commit().await?;

    Ok(actix_web::HttpResponse::Ok().finish())
}

#[get("/api/denormalize_all")]
pub(crate) async fn denormalize_all(
    req: actix_web::HttpRequest,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl actix_web::Responder, bwcommon::MyError> {
    let session = if let Some(session) = req.extensions().get::<UserSession>() {
        session.clone()
    } else {
        return Ok(actix_web::HttpResponse::NotFound().finish());
    };

    if session.id != 4 {
        return Ok(actix_web::HttpResponse::NotFound().finish());
    }

    let con = pool.get().await?;

    let map_ids = con
        .query("Select map.id from map where chkblob is not null", &[])
        .await?
        .into_iter()
        .map(|x| anyhow::Ok(x.try_get::<_, i64>(0)?))
        .collect::<Result<Vec<_>>>()?;

    process_iter_async_concurrent(
        map_ids.iter(),
        || {},
        128,
        |x, y| info!("Completed: {}, ret: {:?}", x, y),
        |_: (), map_id: &i64| async {
            let mut con = pool.get().await?;
            let mut tx = con.transaction().await?;
            bwcommon::denormalize_map_tx(*map_id, &mut tx).await?;
            tx.commit().await?;
            anyhow::Ok(())
        },
    )
    .await;

    Ok(actix_web::HttpResponse::Ok().finish())
}

// #[get("/api/calculate_chkdenorm")]
// pub(crate) async fn calculate_chkdenorm(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         bb8_postgres::bb8::Pool<
//             bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
//         >,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     let session = if let Some(session) = req.extensions().get::<UserSession>() {
//         session.clone()
//     } else {
//         return Ok(actix_web::HttpResponse::NotFound().finish());
//     };

//     if session.id != 4 {
//         return Ok(actix_web::HttpResponse::NotFound().finish());
//     }

//     let con = pool.get().await?;

//     let chk_hashes = con
//         .query("Select hash from chkblob", &[])
//         .await?
//         .into_iter()
//         .map(|x| anyhow::Ok(x.try_get::<_, String>(0)?))
//         .collect::<Result<Vec<_>>>()?;

//     let process = |pool: bb8_postgres::bb8::Pool<
//         bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
//     >,
//                    chkhash: String| async move {
//         let chkblob = db::get_chk(chkhash.clone(), pool.clone()).await?;
//         let raw_chunks = bwmap::parse_chk(chkblob.as_slice());
//         let merged_chunks2 = bwmap::merge_raw_chunks(raw_chunks.as_slice());
//         let parsed_chunks = bwmap::parse_merged_chunks(&merged_chunks2)?;

//         let (width, height) =
//             if let Some(bwmap::ParsedChunk::DIM(x)) = parsed_chunks.get(&bwmap::ChunkName::DIM) {
//                 (Some(*x.width as i64), Some(*x.height as i64))
//             } else {
//                 (None, None)
//             };

//         let tileset =
//             if let Some(bwmap::ParsedChunk::ERA(x)) = parsed_chunks.get(&bwmap::ChunkName::ERA) {
//                 Some((x.tileset % 8) as i64)
//             } else {
//                 None
//             };

//         let (human_players, computer_players) =
//             if let Some(bwmap::ParsedChunk::OWNR(x)) = parsed_chunks.get(&bwmap::ChunkName::OWNR) {
//                 (
//                     Some(x.player_owner.iter().filter(|&&x| x == 6).count() as i64),
//                     Some(x.player_owner.iter().filter(|&&x| x == 5).count() as i64),
//                 )
//             } else {
//                 (None, None)
//             };

//         let doodads =
//             if let Some(bwmap::ParsedChunk::DD2(x)) = parsed_chunks.get(&bwmap::ChunkName::DD2) {
//                 Some(x.doodads.len() as i64)
//             } else {
//                 None
//             };

//         let sprites =
//             if let Some(bwmap::ParsedChunk::THG2(x)) = parsed_chunks.get(&bwmap::ChunkName::THG2) {
//                 Some(x.sprites.len() as i64)
//             } else {
//                 None
//             };

//         let triggers =
//             if let Some(bwmap::ParsedChunk::TRIG(x)) = parsed_chunks.get(&bwmap::ChunkName::TRIG) {
//                 Some(x.triggers.len() as i64)
//             } else {
//                 None
//             };

//         let briefing_triggers =
//             if let Some(bwmap::ParsedChunk::MBRF(x)) = parsed_chunks.get(&bwmap::ChunkName::MBRF) {
//                 Some(x.triggers.len() as i64)
//             } else {
//                 None
//             };

//         let locations =
//             if let Some(bwmap::ParsedChunk::MRGN(x)) = parsed_chunks.get(&bwmap::ChunkName::MRGN) {
//                 Some(
//                     x.locations
//                         .iter()
//                         .filter(|&&x| !(x.left == x.right || x.top == x.bottom))
//                         .count() as i64,
//                 )
//             } else {
//                 None
//             };

//         let units =
//             if let Some(bwmap::ParsedChunk::UNIT(x)) = parsed_chunks.get(&bwmap::ChunkName::UNIT) {
//                 Some(x.units.len() as i64)
//             } else {
//                 None
//             };

//         let eups =
//             if let Some(bwmap::ParsedChunk::UNIT(x)) = parsed_chunks.get(&bwmap::ChunkName::UNIT) {
//                 let mut eups: i64 = 0;
//                 for unit in &x.units {
//                     if *unit.unit_id > 227 || *unit.owner > 27 {
//                         eups += 1;
//                     }
//                 }

//                 Some(eups)
//             } else {
//                 None
//             };

//         let (scenario_name, scenario_description) = if let Some(bwmap::ParsedChunk::SPRP(x)) =
//             parsed_chunks.get(&bwmap::ChunkName::SPRP)
//         {
//             let scenario_string = if *x.scenario_name_string_number == 0 {
//                 None
//             } else {
//                 if let Ok(s) = get_string(&parsed_chunks, *x.scenario_name_string_number as usize) {
//                     Some(s)
//                 } else {
//                     None
//                 }
//             };

//             let scenario_description_string = if *x.description_string_number == 0 {
//                 None
//             } else {
//                 if let Ok(s) = get_string(&parsed_chunks, *x.description_string_number as usize) {
//                     Some(s)
//                 } else {
//                     None
//                 }
//             };

//             (scenario_string, scenario_description_string)
//         } else {
//             (Some("Untitled Scenario".to_owned()), None)
//         };

//         let mut con = pool.get().await?;
//         let transaction = con.transaction().await?;
//         transaction
//             .execute("delete from chkdenorm where chkblob = $1", &[&chkhash])
//             .await?;
//         transaction
//             .execute("insert into chkdenorm (width, height, tileset, human_players, computer_players, sprites, triggers, briefing_triggers, locations, units, scenario_name, get_deaths_euds_or_epds, set_deaths_euds_or_epds, eups, strings, chkblob, doodads, scenario_description
//             ) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)", &[
//                 &width,
//                 &height,
//                 &tileset,
//                 &human_players,
//                 &computer_players,
//                 &sprites,
//                 &triggers,
//                 &briefing_triggers,
//                 &locations,
//                 &units,
//                 &scenario_name,
//                 &Option::<i64>::None,
//                 &Option::<i64>::None,
//                 &eups,
//                 &Option::<i64>::None,
//                 &chkhash,
//                 &doodads,
//                 &scenario_description,
//             ])
//             .await?;

//         transaction.commit().await?;

//         anyhow::Ok(())
//     };

//     process_iter_async_concurrent(
//         chk_hashes.into_iter(),
//         || (**pool).clone(),
//         64,
//         |x, y| info!("completed: {x}"),
//         process,
//     )
//     .await;

//     Ok(actix_web::HttpResponse::Ok().finish())
// }

async fn process_iter_async_concurrent<I, T, F, J, R, F2, H, Z>(
    mut iter: I,
    cloner: H,
    max_outstanding: usize,
    on_item_completed: F2,
    func: F,
) -> usize
where
    I: Iterator<Item = T>,
    F: Fn(Z, T) -> R,
    R: futures_util::Future<Output = J> + Send,
    F2: Fn(usize, J),
    H: Fn() -> Z,
{
    let mut futs = Vec::new();
    let mut counter = 0;
    loop {
        while futs.len() < max_outstanding {
            if let Some(entry) = iter.next() {
                futs.push(func(cloner(), entry).boxed());
            } else {
                break;
            }
        }

        if futs.len() == 0 {
            break;
        }

        let (item, _, remaining_futures) = futures_util::future::select_all(futs).await;

        futs = remaining_futures;

        counter += 1;

        on_item_completed(counter, item);
    }

    counter
}

// #[get("/api/extract-replay")]
// async fn extract_replay(req: HttpRequest, pool: web::Data<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>>) -> Result<HttpResponse, bwcommon::MyError> {

//     if bwcommon::check_auth2(&req, pool.clone()).await.is_none() {
//         return Ok(HttpResponse::Unauthorized().finish());
//     }

//     let replay_ids = {
//         let mut con = pool.get()?;
//         web::block::<_, _, anyhow::Error>(move || {
//             let statement = con.prepare("select id from replay order by id")?;
//             let map_ids: Vec<_> = con.query(&statement, &[])?.iter().map(|row| {
//                 row.get::<_, i64>(0)
//             }).collect();
//             Ok(map_ids)
//         }).await?
//     };

//     for replay_id in replay_ids {
//         let replay_blob = {
//             let pool = pool.clone();
//             web::block::<_, _, anyhow::Error>(move || {
//                 let mut con = pool.get()?;
//                 Ok(con.query_one("select data from replay where id=$1", &[&replay_id])?.get::<_, Vec<u8>>(0))
//             }).await?
//         };

//         let mut hasher = Sha256::new();
//         hasher.update(&replay_blob);
//         let hash = format!("{:x}", hasher.finalize());

//         let pool = pool.clone();
//         web::block::<_, _, anyhow::Error>(move || {
//             let mut con = pool.get()?;
//             Ok(con.execute("insert into replayblob (hash, data) values ($1, $2)", &[&hash, &replay_blob])?)
//         }).await?;
//     }

//     Ok(HttpResponse::Ok().finish())
// }

// #[get("/api/extract-chk")]
// async fn extract_chk(req: actix_web::HttpRequest, pool: web::Data<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>>) -> Result<actix_web::HttpResponse, bwcommon::MyError> {

//     if bwcommon::check_auth2(&req, pool.clone()).await.is_none() {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     }

//     let replay_ids = {
//         let mut con = pool.get()?;
//         web::block::<_, _, anyhow::Error>(move || {
//             let statement = con.prepare("select hash from replayblob order by hash")?;
//             let map_ids: Vec<_> = con.query(&statement, &[])?.iter().map(|row| {
//                 row.get::<_, String>(0)
//             }).collect();
//             Ok(map_ids)
//         }).await?
//     };

//     for replay_id in replay_ids {
//         let replay_blob = {
//             let pool = pool.clone();
//             web::block::<_, _, anyhow::Error>(move || {
//                 let mut con = pool.get()?;
//                 Ok(con.query_one("select data from replayblob where hash=$1", &[&replay_id])?.get::<_, Vec<u8>>(0))
//             }).await?
//         };

//         if let Ok(parsed_replay) = bwreplay::parse_replay_blob(replay_blob.as_slice()) {
//             let mut hasher = Sha256::new();
//             hasher.update(&parsed_replay.chk_data);
//             let hash = format!("{:x}", hasher.finalize());

//             let data = zstd::bulk::compress(parsed_replay.chk_data.as_slice(), 15).unwrap();

//             let pool = pool.clone();
//             web::block::<_, _, anyhow::Error>(move || {
//                 let mut con = pool.get()?;
//                 println!("added: {}", hash);
//                 Ok(con.execute("insert into chkblob (hash, length, ver, data) values ($1, $2, 1, $3) ON CONFLICT DO NOTHING", &[&hash, &(parsed_replay.chk_data.len() as i64), &data])?)
//             }).await?;
//         }
//     }

//     Ok(actix_web::HttpResponse::Ok().finish())
// }

// #[get("/api/calculate-map-denorm-data")]
// async fn calculate_map_denorm_data(req: HttpRequest, pool: web::Data<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>>) -> Result<HttpResponse, bwcommon::MyError> {

//     if bwcommon::check_auth2(&req, pool.clone()).await.is_none() {
//         return Ok(HttpResponse::Unauthorized().finish());
//     }

//     let map_ids = {
//         let mut con = pool.get()?;
//         web::block::<_, _, anyhow::Error>(move || {
//             let statement = con.prepare("select id from map order by id")?;
//             let map_ids: Vec<_> = con.query(&statement, &[])?.iter().map(|row| {
//                 row.get::<_, i64>(0)
//             }).collect();

//             Ok(map_ids)
//         }).await?
//     };

//     let mut map_id_groups = Vec::new();

//     {
//         let mut map_id_temp = Vec::new();
//         for map_id in map_ids {
//             map_id_temp.push(map_id);

//             if map_id_temp.len() > 512 {
//                 map_id_groups.push(map_id_temp.clone());
//                 map_id_temp.clear();
//             }
//         }

//         map_id_groups.push(map_id_temp);
//     }

//     for map_id_group in map_id_groups {

//         let futures: Vec<_> = map_id_group.iter().map(|id| {
//             denorm_map_data2(*id, pool.as_ref().clone())
//         }).collect();

//         let rets = futures::future::join_all(futures).await;

//         for r in rets {
//             if let Err(e) = r {
//                 println!("{:?}", e);
//             }
//         }

//         println!("group");
//     }

//     Ok(HttpResponse::Ok().finish())
// }

// async fn denorm_map_data2(map_id: i64, pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>) -> Result<(), bwcommon::MyError> {

//     let (map_id, original_length, chk_data_compressed) = {
//         let pool = pool.clone();
//         web::block(move || {
//             println!("mapid: {}", map_id);
//             let mut con = pool.get()?;
//             let row = con.query_one("select map.id, chkblob.length, chkblob.data from map join chkblob on chkblob.hash = map.chkblob where map.id = $1", &[&map_id])?;
//             Ok((row.get::<_, i64>(0), row.get::<_, i64>(1), row.get::<_, Vec<u8>>(2)))
//         }).await?
//     };

//     let chk_data = zstd::bulk::decompress(chk_data_compressed.as_slice(), original_length as usize)?;
//     let raw_chunks = bwmap::parse_chk(chk_data.as_slice());
//     let merged_chunks = bwmap::merge_rawchunks(raw_chunks.as_slice());
//     let chk = bwmap::get_parsed_chk(&merged_chunks)?;

//     let mut unit_names = String::new();
//     for unit_setting in &chk.unit_settings {
//         unit_names.push_str(unit_setting.string_name.as_str().chars().filter(|r| {
//             *r >= 32 as char
//         }).collect::<String>().as_str());
//         unit_names.push(' ');
//     }

//     let mut force_names = String::new();
//     for force_name in &chk.forces.force_names {
//         force_names.push_str(force_name.chars().filter(|r| {
//             *r >= 32 as char
//         }).collect::<String>().as_str());
//         force_names.push(' ');
//     }

//     let re = regex::Regex::new(r"\s+").unwrap();

//     let mut scenario_name = chk.scenario_name.clone();
//     scenario_name = re.replace_all(scenario_name.as_str(), " ").to_string();
//     scenario_name = scenario_name.chars().filter(|r|
//         *r >= 32 as char
//     ).collect();

//     let mut scenario_description = chk.scenario_description.clone();
//     scenario_description = re.replace_all(scenario_description.as_str(), " ").to_string();
//     scenario_description = scenario_description.chars().filter(|r| {
//         *r >= 32 as char
//     }).collect();

//     {
//         let pool = pool.clone();
//         web::block(move || {
//             let mut con = pool.get()?;

//             con.execute("update map set
//                 denorm_scenario=$1,
//                 denorm_scenario_description=$2,
//                 denorm_force_names=$3,
//                 denorm_unit_names=$4,
//                 denorm_scenario2=$5
//                 where id = $6", &[
//                     &chk.scenario_name_raw,
//                     &scenario_description,
//                     &force_names,
//                     &unit_names,
//                     &scenario_name,
//                     &map_id
//                     ])?;

//             //println!("map_id: {}, denorm scenario: {}", map_id, scenario_name);
//             Ok(())
//         }).await?
//     }

//     Ok(())
// }

// #[get("/api/extract-chk")]
// async fn extract_chk_from_mpqs_where_chk_is_null(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
//     tileset_maps: web::Data<
//         std::collections::HashMap<u32, std::collections::HashMap<u16, [u8; 3]>>,
//     >,
// ) -> Result<actix_web::HttpResponse, bwcommon::MyError> {
//     if bwcommon::check_auth2(&req, pool.clone()).await.is_none() {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     }

//     let mapblob_hashes = {
//         let mut con = pool.get()?;
//         web::block(move || {
//             let statement = con.prepare("select mapblob2 from map where chkblob is null")?;
//             let map_ids: Vec<_> = con
//                 .query(&statement, &[])?
//                 .iter()
//                 .map(|row| row.get::<_, String>(0))
//                 .collect();
//             anyhow::Ok(map_ids)
//         })
//         .await??
//     };

//     for mapblob_hash in mapblob_hashes {
//         let data = {
//             let pool = pool.clone();
//             let mapblob_hash = mapblob_hash.clone();
//             web::block(move || {
//                 let mut con = pool.get()?;
//                 anyhow::Ok(
//                     con.query_one("select data from mapblob where hash=$1", &[&mapblob_hash])?
//                         .get::<_, Vec<u8>>(0),
//                 )
//             })
//             .await??
//         };

//         let chk_blob = match bwmap::get_chk_from_mpq_in_memory(data.as_slice()) {
//             Ok(x) => x,
//             Err(_x) => {
//                 println!("{:?}, {}", mapblob_hash, _x.to_string());
//                 continue;
//             }
//         };

//         let raw_chunks = bwmap::parse_chk(chk_blob.as_slice());
//         let merged_chunks = bwmap::merge_rawchunks(raw_chunks.as_slice());
//         let chk_dump = bwmap::get_parsed_chk(&merged_chunks)?;

//         let minimap = bwcommon::render_minimap(
//             &chk_dump.mtxm.as_slice(),
//             chk_dump.map_width as usize,
//             chk_dump.map_height as usize,
//             chk_dump.era as usize,
//             &tileset_maps,
//         )?;

//         // calculate hashes
//         let mpq_blob_hash = {
//             let mut hasher = Sha256::new();
//             hasher.update(&data);
//             format!("{:x}", hasher.finalize())
//         };

//         let chk_blob_hash = {
//             let mut hasher = Sha256::new();
//             hasher.update(&chk_blob);
//             format!("{:x}", hasher.finalize())
//         };

//         match chk_blob_hash.as_str() {
//             "15abadff92e85d0b7d25dd42dd80b149d78ac84bb8c3ec049de49d822e670135"
//             | "f1cb1f46b6f6b45fdd8dfa683b17ade1717025e86d96395313fa9859aa36ab2d"
//             | "424926781a5d1a679ebf90343c876acc70f6b5697c2093f591acbc3ff5ecc997"
//             | "946c52c587ca9a84c927ccb64355c3983e078f11f14e40ce82f33c8a4b4fca53"
//             | "294843003eb5a554294a2e6605295d944f34e84888c328291794afc8a78522b3"
//             | "d5149f373ea5cb8130a79412a3f1d377685749f1a49ab2264c75b526f34c39db"
//             | "2e87022c1d1b0dab0aa784593531f16801318913f6e90566c0444cd42692c9e9"
//             | "10fd7657d24038e3c76c0f6a38e8dbf44558245564934e402c37e9b53e42a2c0" => {
//                 continue;
//             }
//             _ => {}
//         }

//         let (ph8x8, ph16x16, ph32x32) = {
//             use image::ImageDecoder;

//             let png = image::codecs::png::PngDecoder::new(minimap.as_slice())?;
//             let (x, y) = png.dimensions();

//             let mut image_data = vec![0; png.total_bytes() as usize];

//             bwcommon::ensure!(png.color_type() == image::ColorType::Rgb8);

//             png.read_image(image_data.as_mut_slice())?;

//             let image: image::ImageBuffer<image::Rgb<u8>, _> =
//                 image::ImageBuffer::from_vec(x, y, image_data).unwrap();

//             let ph8x8 = image::imageops::grayscale(&image::imageops::resize(
//                 &image,
//                 8,
//                 8,
//                 image::imageops::Lanczos3,
//             ));
//             let ph16x16 = image::imageops::grayscale(&image::imageops::resize(
//                 &image,
//                 16,
//                 16,
//                 image::imageops::Lanczos3,
//             ));
//             let ph32x32 = image::imageops::grayscale(&image::imageops::resize(
//                 &image,
//                 32,
//                 32,
//                 image::imageops::Lanczos3,
//             ));

//             // TODO: ph8x8.len() seems to return 4x the number of pixels. even though it is Luma<u8>..?
//             let ph8x8_avg =
//                 (ph8x8.iter().fold(0, |acc, x| acc as usize + (*x as usize)) / ph8x8.len()) as u8;
//             let ph16x16_avg = (ph16x16
//                 .iter()
//                 .fold(0, |acc, x| acc as usize + (*x as usize))
//                 / ph16x16.len()) as u8;
//             let ph32x32_avg = (ph32x32
//                 .iter()
//                 .fold(0, |acc, x| acc as usize + (*x as usize))
//                 / ph32x32.len()) as u8;

//             let ph8x8: Vec<_> = ph8x8
//                 .iter()
//                 .map(|x| if *x < ph8x8_avg { 0 } else { 1 })
//                 .collect::<Vec<u8>>()
//                 .chunks_exact(8)
//                 .map(|x| x.iter().fold(0u8, |acc, x| acc << 1 | *x))
//                 .collect();
//             bwcommon::ensure!(ph8x8.len() == 8 * 8 / 8);

//             let ph16x16: Vec<_> = ph16x16
//                 .iter()
//                 .map(|x| if *x < ph16x16_avg { 0 } else { 1 })
//                 .collect::<Vec<u8>>()
//                 .chunks_exact(8)
//                 .map(|x| x.iter().fold(0u8, |acc, x| acc << 1 | *x))
//                 .collect();
//             bwcommon::ensure!(ph16x16.len() == 16 * 16 / 8);

//             let ph32x32: Vec<_> = ph32x32
//                 .iter()
//                 .map(|x| if *x < ph32x32_avg { 0 } else { 1 })
//                 .collect::<Vec<u8>>()
//                 .chunks_exact(8)
//                 .map(|x| x.iter().fold(0u8, |acc, x| acc << 1 | *x))
//                 .collect();
//             bwcommon::ensure!(ph32x32.len() == 32 * 32 / 8);

//             (ph8x8, ph16x16, ph32x32)
//         };

//         // calculate denormalized stuff
//         let mut unit_names = String::new();
//         for unit_setting in &chk_dump.unit_settings {
//             unit_names.push_str(
//                 unit_setting
//                     .string_name
//                     .as_str()
//                     .chars()
//                     .filter(|r| *r >= 32 as char)
//                     .collect::<String>()
//                     .as_str(),
//             );
//             unit_names.push(' ');
//         }

//         let mut force_names = String::new();
//         for force_name in &chk_dump.forces.force_names {
//             force_names.push_str(
//                 force_name
//                     .chars()
//                     .filter(|r| *r >= 32 as char)
//                     .collect::<String>()
//                     .as_str(),
//             );
//             force_names.push(' ');
//         }

//         let re = regex::Regex::new(r"\s+").unwrap();

//         let mut scenario_name = chk_dump.scenario_name.clone();
//         scenario_name = re.replace_all(scenario_name.as_str(), " ").to_string();
//         scenario_name = scenario_name.chars().filter(|r| *r >= 32 as char).collect();

//         let mut scenario_description = chk_dump.scenario_description.clone();
//         scenario_description = re
//             .replace_all(scenario_description.as_str(), " ")
//             .to_string();
//         scenario_description = scenario_description
//             .chars()
//             .filter(|r| *r >= 32 as char)
//             .collect();

//         // compress chk
//         let chk_blob_compressed = zstd::bulk::compress(chk_blob.as_slice(), 15)?;

//         // get now time:
//         //let time_since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;

//         // begin db stuff
//         let mut con = pool.get()?;
//         let mut tx = con.transaction()?;

//         // https://stackoverflow.com/questions/40878027/detect-if-the-row-was-updated-or-inserted/40880200#40880200
//         // TODO: detect if row was inserted or updated.
//         // using xmax it can be done.

//         tx.execute("INSERT INTO chkblob (hash, ver, length, data) VALUES ($1, 1, $2, $3) ON CONFLICT DO NOTHING RETURNING Cast((xmax = 0) as boolean) AS inserted",
//         &[&chk_blob_hash, &(chk_blob.len() as i64), &chk_blob_compressed])?;

//         tx.execute(
//             "INSERT INTO minimap
//             (chkhash, width, height, minimap, ph8x8, ph16x16, ph32x32) VALUES
//             ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING",
//             &[
//                 &chk_blob_hash,
//                 &(chk_dump.map_width as i32),
//                 &(chk_dump.map_height as i32),
//                 &minimap,
//                 &ph8x8,
//                 &ph16x16,
//                 &ph32x32,
//             ],
//         )?;

//         tx.execute("
//         update map set denorm_scenario=$1, chkblob=$2, denorm_scenario_description=$3, denorm_force_names=$4, denorm_unit_names=$5, denorm_scenario2=$6
//         where mapblob2=$7",
//         &[&scenario_name.as_bytes(), &chk_blob_hash, &scenario_description, &force_names, &unit_names, &scenario_name, &mpq_blob_hash])?;

//         // let map_id = tx.query_one("SELECT id from map where mapblob2 = $1", &[&mpq_blob_hash])?.try_get::<_, i64>("id")?;

//         // tx.execute("INSERT INTO Filename (filename) VALUES ($1) ON CONFLICT DO NOTHING", &[&filename])?;
//         // let filename_id = tx.query_one("SELECT id from filename where filename = $1", &[&filename])?.try_get::<_, i64>("id")?;

//         // tx.execute("INSERT INTO MapFilename (filename, map) VALUES ($1, $2) ON CONFLICT DO NOTHING", &[&filename_id, &map_id])?;

//         println!("UPDATED mapblobhash: {:?}", mapblob_hash);

//         tx.commit()?;
//     }

//     Ok(actix_web::HttpResponse::Ok().finish())
// }

// #[get("/api/hacks/upload-all-blobs")]
// async fn upload_all_blobs(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
//     tileset_maps: web::Data<
//         std::collections::HashMap<u32, std::collections::HashMap<u16, [u8; 3]>>,
//     >,
// ) -> Result<actix_web::HttpResponse, bwcommon::MyError> {
//     if bwcommon::check_auth2(&req, pool.clone()).await.is_none() {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     }
// }

// async fn denormalize_all_strings2(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     if let Some(user_id) = bwcommon::check_auth2(&req, pool.clone()).await {
//         if user_id != 4 {
//             return Ok(actix_web::HttpResponse::Unauthorized().finish());
//         }
//     } else {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     }

//     let chk_hashes = {
//         let pool = pool.clone();
//         web::block(move || {
//             let mut con = pool.get()?;

//             let hashes: Result<Vec<_>, bwcommon::MyError> = con
//                 .query(
//                     "select chkblob.hash, map.id
//                 from chkblob
//                 join map on map.chkblob = chkblob.hash",
//                     &[],
//                 )?
//                 .into_iter()
//                 .map(|x| Ok((x.try_get::<_, String>(0)?, x.try_get::<_, i64>(1)?)))
//                 .collect();

//             con.execute("truncate stringmap2", &[])?;

//             Ok(hashes?)
//         })
//         .await?
//     };

//     let do_map = |hash: String,
//                   map_id: i64,
//                   pool: r2d2::Pool<
//         r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>,
//     >| {
//         db::get_chk(hash.clone(), pool.clone()).then(move |chk_data| {
//             let raw_chunks = bwmap::parse_chk(chk_data.unwrap().as_slice());
//             let merged_chunks = bwmap::merge_rawchunks(raw_chunks.as_slice());
//             let chk = bwmap::get_parsed_chk(&merged_chunks).unwrap();

//             web::block(move || {
//                 let mut con = pool.get()?;

//                 // scenario name
//                 con.execute(
//                     "call insert_string2($1, $2, $3)",
//                     &[
//                         &map_id,
//                         &"scenario_name",
//                         &vec![crate::util::sanitize_sc_string_preserve_newlines(
//                             chk.scenario_name.as_str(),
//                         )],
//                     ],
//                 )?;

//                 // scenario description
//                 con.execute(
//                     "call insert_string2($1, $2, $3)",
//                     &[
//                         &map_id,
//                         &"scenario_description",
//                         &vec![crate::util::sanitize_sc_string_preserve_newlines(
//                             chk.scenario_description.as_str(),
//                         )],
//                     ],
//                 )?;

//                 // unit names
//                 {
//                     let unit_names: Vec<String> = chk
//                         .unit_settings
//                         .into_iter()
//                         .map(|x| x.string_name)
//                         .map(|x| crate::util::sanitize_sc_string_preserve_newlines(x.as_str()))
//                         .collect();

//                     con.execute(
//                         "call insert_string2($1, $2, $3)",
//                         &[&map_id, &"unit_names", &unit_names],
//                     )?;
//                 }

//                 // force names
//                 {
//                     let force_names: Vec<String> = chk
//                         .forces
//                         .force_names
//                         .into_iter()
//                         .map(|x| crate::util::sanitize_sc_string_preserve_newlines(x.as_str()))
//                         .collect();

//                     con.execute(
//                         "call insert_string2($1, $2, $3)",
//                         &[&map_id, &"force_names", &force_names],
//                     )?;
//                 }

//                 // file names
//                 {
//                     let filenames = con
//                         .query(
//                             "
//                     select filename.filename
//                     from map
//                     join mapfilename on mapfilename.map = map.id
//                     join filename on filename.id = mapfilename.filename
//                     where map.id = $1",
//                             &[&map_id],
//                         )?
//                         .into_iter()
//                         .map(|x| Ok(x.try_get(0)?))
//                         .collect::<Result<Vec<String>, bwcommon::MyError>>()?;

//                     con.execute(
//                         "call insert_string2($1, $2, $3)",
//                         &[&map_id, &"file_names", &filenames],
//                     )?;
//                 }

//                 anyhow::Ok(())
//             })
//         })
//     };

//     let mut counter = 0;

//     let mut iter = chk_hashes.into_iter();
//     let mut futs = Vec::new();

//     loop {
//         while futs.len() < 1024 {
//             if let Some(map) = iter.next() {
//                 futs.push(do_map(map.0, map.1, (**pool).clone()).boxed());
//             } else {
//                 break;
//             }
//         }

//         let (_, _, remaining_futures) = futures::future::select_all(futs).await;

//         futs = remaining_futures;

//         if futs.len() == 0 {
//             break;
//         }

//         counter += 1;
//         println!("counter: {}", counter);
//     }

//     //let mut

//     // for maps in chk_hashes.chunks(128) {
//     //     let futures: Vec<_> = maps
//     //         .iter()
//     //         .map(|(hash, map_id)| {
//     //             let pool = pool.clone();
//     //             let hash = hash.clone();
//     //             let map_id = *map_id;

//     //             })
//     //         })
//     //         .collect();

//     //     counter += futures.len();
//     //     futures::future::try_join_all(futures.into_iter()).await?;
//     //     println!("counter: {counter}");
//     // }

//     Ok(actix_web::HttpResponse::Ok().finish())
// }

// #[get("/api/hack/denormalize_all_strings")]
// pub(crate) async fn denormalize_all_strings(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     denormalize_all_strings2(req, pool).await
// }

// async fn denormalize_scenario_strings2(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     if let Some(user_id) = bwcommon::check_auth2(&req, pool.clone()).await {
//         if user_id != 4 {
//             return Ok(actix_web::HttpResponse::Unauthorized().finish());
//         }
//     } else {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     }

//     let chk_hashes = {
//         let pool = pool.clone();
//         web::block(move || {
//             let mut con = pool.get()?;

//             let hashes: Result<Vec<_>, bwcommon::MyError> = con
//                 .query(
//                     "select chkblob.hash, map.id
//                 from chkblob
//                 join map on map.chkblob = chkblob.hash",
//                     &[],
//                 )?
//                 .into_iter()
//                 .map(|x| Ok((x.try_get::<_, String>(0)?, x.try_get::<_, i64>(1)?)))
//                 .collect();

//             Ok(hashes?)
//         })
//         .await?
//     };

//     let do_map = |hash: String,
//                   map_id: i64,
//                   pool: r2d2::Pool<
//         r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>,
//     >| async move {
//         let chk_data = db::get_chk(hash.clone(), pool.clone()).await?;
//         let raw_chunks = bwmap::parse_chk(chk_data.as_slice());
//         let merged_chunks = bwmap::merge_rawchunks(raw_chunks.as_slice());
//         let chk = bwmap::get_parsed_chk(&merged_chunks).unwrap();

//         Result::<_, bwcommon::MyError>::Ok(
//             web::block(move || {
//                 let mut con = pool.get()?;

//                 con.execute(
//                     "update map set denorm_scenario=$1 where id = $2",
//                     &[
//                         &crate::util::sanitize_sc_scenario_string(chk.scenario_name.as_str()),
//                         &map_id,
//                     ],
//                 )?;

//                 Ok(())
//             })
//             .await?,
//         )
//     };

//     let mut counter = 0;

//     let mut iter = chk_hashes.into_iter();
//     let mut futs = Vec::new();

//     loop {
//         while futs.len() < 1024 {
//             if let Some(map) = iter.next() {
//                 futs.push(do_map(map.0, map.1, (**pool).clone()).boxed());
//             } else {
//                 break;
//             }
//         }

//         let (_, _, remaining_futures) = futures::future::select_all(futs).await;

//         futs = remaining_futures;

//         if futs.len() == 0 {
//             break;
//         }

//         counter += 1;
//         println!("counter: {}", counter);
//     }

//     Ok(actix_web::HttpResponse::Ok().finish())
// }

// #[get("/api/hack/denormalize_scenario_strings")]
// pub(crate) async fn denormalize_scenario_strings(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     denormalize_scenario_strings2(req, pool).await
// }

// async fn insert_a_shitload_of_maps2(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
//     _tileset_maps: web::Data<
//         std::collections::HashMap<u32, std::collections::HashMap<u16, [u8; 3]>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     if let Some(user_id) = bwcommon::check_auth2(&req, pool.clone()).await {
//         if user_id != 4 {
//             return Ok(actix_web::HttpResponse::Unauthorized().finish());
//         }
//     } else {
//         return Ok(actix_web::HttpResponse::Unauthorized().finish());
//     }

//     let do_map = |entry: walkdir::DirEntry| async {
//         let pool = pool.clone();
//         Result::<_, bwcommon::MyError>::Ok(
//             web::block(move || {
//                 let mut con = pool.get()?;

//                 let metadata = entry.metadata()?;
//                 let accessed_time = metadata
//                     .accessed()?
//                     .duration_since(std::time::UNIX_EPOCH)?
//                     .as_secs() as i64;
//                 let creation_time = metadata
//                     .created()?
//                     .duration_since(std::time::UNIX_EPOCH)?
//                     .as_secs() as i64;
//                 let modified_time = metadata
//                     .modified()?
//                     .duration_since(std::time::UNIX_EPOCH)?
//                     .as_secs() as i64;

//                 let mpq_data = std::fs::read(entry.path())?;

//                 let mpq_data_hash = {
//                     let mut hasher = Sha256::new();
//                     hasher.update(&mpq_data);
//                     format!("{:x}", hasher.finalize())
//                 };

//                 con.execute(
//                     "
//                     insert into filetime (map, accessed_time, modified_time, creation_time) (select id AS map, $2 AS accessed_time, $3 AS modified_time, $4 AS creation_time from map where mapblob2 = $1) on conflict do nothing",
//                     &[&mpq_data_hash, &accessed_time, &modified_time, &creation_time])?;

//                 Ok(())
//             })
//             .await?)
//     };

//     let entries = walkdir::WalkDir::new("/home/stan/warez/starcraft/archives")
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|e| {
//             !e.file_type().is_dir()
//                 && !e.file_name().to_string_lossy().starts_with(".")
//                 && (e.file_name().to_string_lossy().ends_with(".scm")
//                     || e.file_name().to_string_lossy().ends_with(".scx"))
//         })
//         .map(|entry| entry.clone())
//         .collect::<Vec<_>>();

//     let total_processed_items = crate::util::process_iter_async_concurrent(
//         entries.into_iter(),
//         1024,
//         |total_completed_so_far, item| {
//             println!(
//                 "total_completed_so_far: {total_completed_so_far}, item: {:?}",
//                 item
//             );
//         },
//         do_map,
//     )
//     .await;

//     println!("total_processed_items: {total_processed_items}");

//     Ok(actix_web::HttpResponse::Ok().finish())
// }

// #[get("/api/hack/insert_a_shitload_of_maps")]
// pub(crate) async fn insert_a_shitload_of_maps(
//     req: actix_web::HttpRequest,
//     pool: web::Data<
//         r2d2::Pool<r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>>,
//     >,
//     tileset_maps: web::Data<
//         std::collections::HashMap<u32, std::collections::HashMap<u16, [u8; 3]>>,
//     >,
// ) -> Result<impl actix_web::Responder, bwcommon::MyError> {
//     insert_a_shitload_of_maps2(req, pool, tileset_maps).await
// }
