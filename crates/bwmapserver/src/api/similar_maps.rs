use actix_web::get;
use actix_web::{web, HttpResponse, Responder};
use anyhow::Result;
use bwcommon::{get_web_id_from_db_id, insert_extension, ApiSpecificInfoForLogging};
use serde::{Deserialize, Serialize};
use serde_json::json;

use tracing::instrument;

// type KdTree = kiddo::KdTree<f32, i64, 256>;

// #[cached(key = "(i64)", convert = r#"{_len}"#, size = 1, result = true)]
// async fn get_kd_tree(
//     _len: i64,
//     pool: bb8_postgres::bb8::Pool<
//         bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
//     >,
// ) -> Result<Arc<KdTree>> {
//     let mut tree: KdTree = KdTree::new();

//     let con = pool.get().await?;

//     let rows = con
//         .query(
//             "select map.id, ph16x16 from minimap join map on map.chkblob = minimap.chkhash",
//             &[],
//         )
//         .await?
//         .into_iter()
//         .map(|x| {
//             let map_id = x.try_get::<_, i64>("id")?;
//             let ph32x32 = x.try_get::<_, Vec<u8>>("ph16x16")?;

//             let mut ret = [0f32; 256];

//             for (index, x) in ph32x32.iter().enumerate() {
//                 ret[index * 8] = (x & 0x01) as f32;
//                 ret[index * 8 + 1] = ((x & 0x02) >> 1) as f32;
//                 ret[index * 8 + 2] = ((x & 0x04) >> 2) as f32;
//                 ret[index * 8 + 3] = ((x & 0x08) >> 3) as f32;
//                 ret[index * 8 + 4] = ((x & 0x10) >> 4) as f32;
//                 ret[index * 8 + 5] = ((x & 0x20) >> 5) as f32;
//                 ret[index * 8 + 6] = ((x & 0x40) >> 6) as f32;
//                 ret[index * 8 + 7] = ((x & 0x80) >> 7) as f32;
//             }

//             anyhow::Ok((map_id, ret))
//         })
//         .collect::<Result<Vec<(i64, [f32; 256])>>>()?;

//     for r in rows {
//         tree.add(&r.1, r.0)?;
//     }

//     anyhow::Ok(Arc::new(tree))
// }

#[get("/api/similar_maps/{map_id}")]
#[instrument(skip_all)]
pub async fn handler(
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<impl Responder, bwcommon::MyError> {
    let (map_id,) = path.into_inner();
    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        map_id.parse::<i64>()?
    } else {
        bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID)?
    };

    let con = pool.get().await?;
    // let len: i64 = con
    //     .query_one("select count(*) from minimap", &[])
    //     .instrument(tracing::info_span!("len").or_current())
    //     .await?
    //     .try_get(0)?;
    // let ph16x16: Vec<u8> = con
    //     .query_one(
    //         "select minimap.ph16x16 from map join minimap on map.chkblob = minimap.chkhash where map.id = $1",
    //         &[&map_id],
    //     )
    //     .instrument(tracing::info_span!("ph16x16").or_current())
    //     .await?
    //     .try_get(0)?;

    let nearest2 = {
        let rows = con
            .query(
                "
                select 
                    map.id,
                    chkdenorm.scenario_name,
                    chkdenorm.chkblob,
                    min(modified_time) as lmt,
                    chkdenorm.width,
                    chkdenorm.height,
                    chkdenorm.tileset,
                    minimap.hamming_distance
                from (
                    select
                        minimap.chkhash,
                        vector <~> (select vector from minimap join map on map.chkblob = minimap.chkhash where map.id = $1 limit 1) as hamming_distance
                    from
                        minimap
                    order by
                        hamming_distance
                    limit 25
                ) minimap
                join map on map.chkblob = minimap.chkhash
                join chkdenorm on chkdenorm.chkblob = map.chkblob
                join filetime on filetime.map = map.id
                where
                    map.id != $1 and
                    nsfw = false and
                    outdated = false and
                    unfinished = false and
                    broken = false and
                    blackholed = false and
                    chkdenorm.scenario_name is not null
                group by
                    map.id, chkdenorm.scenario_name, chkdenorm.chkblob, hamming_distance, chkdenorm.width, chkdenorm.height, chkdenorm.tileset
                order by
                    hamming_distance
            ",
                &[&map_id],
            )
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(Chk {
                    map_id: get_web_id_from_db_id(row.try_get("id")?, crate::util::SEED_MAP_ID)?,
                    hamming_distance: row.try_get::<_, f64>("hamming_distance")? as i64,
                    scenario_name: row.try_get("scenario_name")?,
                    last_modified_time: row.try_get("lmt")?,
                    width: row.try_get("width")?,
                    height: row.try_get("height")?,
                    tileset: row.try_get("tileset")?,
                })
            })
            .collect::<Result<Vec<_>>>()
    }?;

    // let mut point = [0f32; 256];
    // for (index, x) in ph16x16.iter().enumerate() {
    //     point[index * 8] = (x & 0x01) as f32;
    //     point[index * 8 + 1] = ((x & 0x02) >> 1) as f32;
    //     point[index * 8 + 2] = ((x & 0x04) >> 2) as f32;
    //     point[index * 8 + 3] = ((x & 0x08) >> 3) as f32;
    //     point[index * 8 + 4] = ((x & 0x10) >> 4) as f32;
    //     point[index * 8 + 5] = ((x & 0x20) >> 5) as f32;
    //     point[index * 8 + 6] = ((x & 0x40) >> 6) as f32;
    //     point[index * 8 + 7] = ((x & 0x80) >> 7) as f32;
    // }

    // let kd_tree = get_kd_tree(len, (**pool).clone())
    //     .instrument(tracing::info_span!("get_kd_tree").or_current())
    //     .await?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    // let nearest: Vec<_> = {
    //     let span = info_span!("nearest");
    //     let _e = span.enter();
    //     kd_tree
    //         .iter_nearest(&point, &|x, y| {
    //             x.into_iter()
    //                 .zip(y)
    //                 .fold(0.0, |a, (&b, &c)| a + (b - c).abs())
    //         })
    //         .map_err(|x| anyhow::Error::from(x))?
    //         .filter(|(_hamming_distance, id)| **id != map_id)
    //         .take_while(|(hamming_distance, _id)| *hamming_distance <= 64.0)
    //         .take(20)
    //         .collect()
    // };

    // let hamming_distance_map: HashMap<_, _> = nearest.iter().map(|(f, i)| (**i, *f)).collect();
    // let map_ids: Vec<_> = nearest.iter().map(|(_, i)| **i).collect();

    #[derive(Debug, Serialize, Deserialize)]
    struct Chk {
        map_id: String,
        hamming_distance: i64,
        scenario_name: String,
        last_modified_time: Option<i64>,
        width: i64,
        height: i64,
        tileset: i64,
    }

    // let mut ret = {
    //     let pool = (**pool).clone();
    //     let con = pool.get().await?;

    //     #[rustfmt::skip]
    //     let row = con.query("
    //         select map.id, chkdenorm.scenario_name, chkdenorm.chkblob, min(modified_time) as lmt, chkdenorm.width, chkdenorm.height, chkdenorm.tileset
    //         from map
    //         join chkdenorm on chkdenorm.chkblob = map.chkblob
    //         join filetime on filetime.map = map.id
    //         where map.id = any($1) and nsfw = false and outdated = false and unfinished = false and broken = false and blackholed = false and chkdenorm.scenario_name is not null
    //         group by map.id, chkdenorm.scenario_name, chkdenorm.chkblob",
    //             &[&map_ids],
    //         ).await?;

    //     row.into_iter()
    //         .map(|row| {
    //             let map_id = row.try_get("id")?;
    //             Ok(Chk {
    //                 map_id: get_web_id_from_db_id(map_id, crate::util::SEED_MAP_ID)?,
    //                 hamming_distance: hamming_distance_map[&map_id] as i64,
    //                 scenario_name: row.try_get("scenario_name")?,
    //                 last_modified_time: row.try_get("lmt")?,
    //                 width: row.try_get("width")?,
    //                 height: row.try_get("height")?,
    //                 tileset: row.try_get("tileset")?,
    //             })
    //         })
    //         .collect::<Result<Vec<_>>>()
    // }?;

    // ret.sort_by_key(|x| x.hamming_distance);

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(serde_json::to_string(&json!({
            // "v1": ret,
            "v2": nearest2}))?))
}
