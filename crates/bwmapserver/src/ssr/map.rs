use crate::middleware::UserSession;
use crate::ssr::get_navbar_langmap;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web::{HttpMessage, HttpRequest};
use anyhow::Result;
use bwmap::ParsedChk;
use cached::proc_macro::cached;
use serde_json::json;
use std::sync::Arc;
use tracing::{info_span, instrument};

type KdTree = kiddo::KdTree<f32, i64, 256>;

#[cached(key = "(i64)", convert = r#"{_len}"#, size = 1, result = true)]
async fn get_kd_tree(
    _len: i64,
    pool: bb8_postgres::bb8::Pool<
        bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
    >,
) -> Result<Arc<KdTree>> {
    let mut tree: KdTree = KdTree::new();

    let con = pool.get().await?;

    let rows = con
        .query(
            "select map.id, ph16x16 from minimap join map on map.chkblob = minimap.chkhash",
            &[],
        )
        .await?
        .into_iter()
        .map(|x| {
            let map_id = x.try_get::<_, i64>("id")?;
            let ph32x32 = x.try_get::<_, Vec<u8>>("ph16x16")?;

            let mut ret = [0f32; 256];

            for (index, x) in ph32x32.iter().enumerate() {
                ret[index * 8] = (x & 0x01) as f32;
                ret[index * 8 + 1] = ((x & 0x02) >> 1) as f32;
                ret[index * 8 + 2] = ((x & 0x04) >> 2) as f32;
                ret[index * 8 + 3] = ((x & 0x08) >> 3) as f32;
                ret[index * 8 + 4] = ((x & 0x10) >> 4) as f32;
                ret[index * 8 + 5] = ((x & 0x20) >> 5) as f32;
                ret[index * 8 + 6] = ((x & 0x40) >> 6) as f32;
                ret[index * 8 + 7] = ((x & 0x80) >> 7) as f32;
            }

            anyhow::Ok((map_id, ret))
        })
        .collect::<Result<Vec<(i64, [f32; 256])>>>()?;

    for r in rows {
        tree.add(&r.1, r.0)?;
    }

    anyhow::Ok(Arc::new(tree))
}

// #[get("/api/similar_maps/{chkhash}")]
// #[instrument(skip_all)]
// pub async fn handler(
//     path: web::Path<(String,)>,
//     pool: web::Data<
//         bb8_postgres::bb8::Pool<
//             bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
//         >,
//     >,
// ) -> Result<impl Responder, bwcommon::MyError> {
//     let (chkhash,) = path.into_inner();

//     let con = pool.get().await?;
//     let len: i64 = con
//         .query_one("select count(*) from minimap", &[])
//         .instrument(tracing::info_span!("len").or_current())
//         .await?
//         .try_get(0)?;
//     let ph16x16: Vec<u8> = con
//         .query_one(
//             "select ph16x16 from minimap where chkhash = $1",
//             &[&chkhash],
//         )
//         .instrument(tracing::info_span!("ph16x16").or_current())
//         .await?
//         .try_get(0)?;

//     let mut point = [0f32; 256];
//     for (index, x) in ph16x16.iter().enumerate() {
//         point[index * 8 + 0] = ((x & 0x01) >> 0) as f32;
//         point[index * 8 + 1] = ((x & 0x02) >> 1) as f32;
//         point[index * 8 + 2] = ((x & 0x04) >> 2) as f32;
//         point[index * 8 + 3] = ((x & 0x08) >> 3) as f32;
//         point[index * 8 + 4] = ((x & 0x10) >> 4) as f32;
//         point[index * 8 + 5] = ((x & 0x20) >> 5) as f32;
//         point[index * 8 + 6] = ((x & 0x40) >> 6) as f32;
//         point[index * 8 + 7] = ((x & 0x80) >> 7) as f32;
//     }

//     let kd_tree = get_kd_tree(len, (**pool).clone())
//         .instrument(tracing::info_span!("get_kd_tree").or_current())
//         .await?;

//     //kd_tree.nearest(point, num, distance)

//     let info = ApiSpecificInfoForLogging {
//         chk_hash: Some(chkhash.clone()),
//         ..Default::default()
//     };

//     #[derive(Debug, Serialize, Deserialize)]
//     struct SimilarChks {
//         ph8x8: Vec<Chk>,
//         ph16x16: Vec<Chk>,
//         ph32x32: Vec<Chk>,
//     }

//     let nearest: Vec<_> = {
//         let span = info_span!("nearest");
//         let _e = span.enter();
//         kd_tree
//             .iter_nearest(&point, &|x, y| {
//                 x.into_iter()
//                     .zip(y)
//                     .fold(0.0, |a, (&b, &c)| a + (b - c).abs())
//             })
//             .map_err(|x| anyhow::Error::from(x))?
//             .take_while(|x| x.0 <= 64.0)
//             .take(20)
//             .collect()
//     };

//     let mut arr = vec![];

//     for (hamming_distance, map_id) in nearest {
//         let pool = (**pool).clone();

//         arr.push(async move {
//             let con = pool.get().await?;

//             #[rustfmt::skip]
//             let row = con.query_one("
//                 select chkdenorm.scenario_name, chkdenorm.chkblob
//                 from map
//                 join chkdenorm on chkdenorm.chkblob = map.chkblob
//                 where map.id = $1",
//                     &[&map_id],
//                 ).await?;

//             anyhow::Ok(Chk {
//                 chkhash: row.try_get::<_, String>("chkblob")?,
//                 map_id: format!("{}", map_id),
//                 hamming_distance: hamming_distance as u64,
//                 scenario_name: row.try_get::<_, String>("scenario_name")?,
//             })
//         });
//     }

//     #[derive(Debug, Serialize, Deserialize)]
//     struct Chk {
//         chkhash: String,
//         map_id: String,
//         hamming_distance: u64,
//         scenario_name: String,
//     }

//     let ret = futures::future::join_all(arr)
//         .await
//         .into_iter()
//         .filter_map(|s| s.ok())
//         .collect();

//     let similar_chks = SimilarChks {
//         ph8x8: Vec::new(),
//         ph16x16: Vec::new(),
//         ph32x32: ret,
//     };

//     Ok(insert_extension(HttpResponse::Ok(), info)
//         .content_type("application/json")
//         .body(serde_json::to_string(&similar_chks).unwrap()))
// }

#[instrument(skip_all, name = "/map")]
#[get("/uiv1/map/{map_id}")]
async fn handler(
    req: HttpRequest,
    path: web::Path<(String,)>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
) -> Result<impl Responder, bwcommon::MyError> {
    let lang = req
        .extensions()
        .get::<bwcommon::LangData>()
        .unwrap_or(&bwcommon::LangData::English)
        .to_owned();

    let user_username = req
        .extensions()
        .get::<UserSession>()
        .map(|x| (x.username.clone(), true))
        .unwrap_or_default();

    // convert IDs
    let (map_id,) = path.into_inner();

    let mut redirect = false;

    let map_id = if map_id.chars().all(|x| x.is_ascii_digit()) && map_id.len() < 8 {
        redirect = true;
        map_id.parse::<i64>()?
    } else {
        let map_id = bwcommon::get_db_id_from_web_id(&map_id, crate::util::SEED_MAP_ID);
        if map_id.is_err() {
            return Ok(HttpResponse::NotFound().finish().customize());
        } else {
            map_id.unwrap()
        }
    };

    tracing::info!("real map id: {}", map_id);

    {
        let pool = pool.clone();
        let con = pool.get().await?;
        let rows = con
            .query_one("select count(*) from map where map.id = $1", &[&map_id])
            .await?
            .try_get::<_, i64>(0)?;

        if rows == 0 {
            return Ok(HttpResponse::NotFound().finish().customize());
        } else if redirect {
            return Ok(HttpResponse::PermanentRedirect()
                .finish()
                .customize()
                .insert_header((
                    "Location",
                    format!(
                        "/map/{}",
                        bwcommon::get_web_id_from_db_id(map_id, crate::util::SEED_MAP_ID)?
                    ),
                )));
        }
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Filename {
        filename: String,
    }

    let current_user = bwcommon::check_auth4(&req, (**pool).clone());

    let filenames = async {
        let pool = pool.clone();
        let con = pool.get().await?;
        anyhow::Ok(
            con.query(
                "select filename.filename
            from mapfilename
            join filename on mapfilename.filename = filename.id
            where mapfilename.map = $1",
                &[&map_id],
            )
            .await?
            .into_iter()
            .map(|row| {
                anyhow::Ok(Filename {
                    filename: row.try_get(0)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        )
    };

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Filetime {
        accessed_time: Option<i64>,
        creation_time: Option<i64>,
        modified_time: Option<i64>,
    }

    let filetimes = async {
        let pool = pool.clone();
        let con = pool.get().await?;
        anyhow::Ok(
            con.query(
                "select accessed_time, creation_time, modified_time
            from map
            join filetime on filetime.map = map.id
            where map.id = $1
            order by modified_time, creation_time, accessed_time",
                &[&map_id],
            )
            .await?
            .into_iter()
            .map(|row| {
                anyhow::Ok(Filetime {
                    accessed_time: row.try_get(0)?,
                    creation_time: row.try_get(1)?,
                    modified_time: row.try_get(2)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        )
    };

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Tag {
        key: String,
        value: String,
    }

    let tags = async {
        let pool = pool.clone();

        let con = pool.get().await?;
        anyhow::Ok(
            con.query(
                "
            select key, value
            from tagmap
            join tag on tagmap.tag = tag.id
            where tagmap.map = $1",
                &[&map_id],
            )
            .await?
            .into_iter()
            .map(|row| {
                anyhow::Ok(Tag {
                    key: row.try_get(0)?,
                    value: row.try_get(1)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        )
    };

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Flags {
        nsfw: bool,
        unfinished: bool,
        broken: bool,
        outdated: bool,
        blackholed: bool,
    }
    let flags = async {
        let pool = pool.clone();

        let con = pool.get().await?;
        let row = con
            .query_one(
                "
            select nsfw, unfinished, broken, outdated, blackholed
            from map
            where map.id = $1",
                &[&map_id],
            )
            .await?;

        anyhow::Ok(Flags {
            nsfw: row.try_get(0)?,
            unfinished: row.try_get(1)?,
            broken: row.try_get(2)?,
            outdated: row.try_get(3)?,
            blackholed: row.try_get(4)?,
        })
    };

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct ReplayInfo {
        id: i64,
        frames: i64,
        time_saved: i64,
        scenario_name: String,
        creator: String,
    }

    let replays = async {
        let pool = pool.clone();
        let con = pool.get().await?;
        anyhow::Ok(con.query("
                select replay.id, replay.denorm_frames, replay.denorm_time_saved, replay.denorm_scenario, replay.denorm_game_creator
                from replay
                join map on map.chkblob = replay.chkhash
                where map.id = $1
                order by replay.denorm_frames", &[&map_id]).await?.into_iter().map(|r| {
                anyhow::Ok(ReplayInfo {
                    id: r.try_get(0)?,
                    frames: r.try_get(1)?,
                    time_saved: r.try_get(2)?,
                    scenario_name: encoding_rs::UTF_8.decode(r.try_get::<_, Vec<u8>>(3)?.as_slice()).0.to_string(),
                    creator: encoding_rs::UTF_8.decode(r.try_get::<_, Vec<u8>>(4)?.as_slice()).0.to_string(),
                })
            }).collect::<Result<Vec<_>, _>>()?)
    };

    let chkblob = async {
        let pool = pool.clone();
        let con = pool.get().await?;
        let row = con.query_one("
                select map.mapblob2, map.chkblob, mapblob_size, chkblob.length, chkblob.ver, chkblob.data, nsfw, uploaded_by, blackholed, views, downloads from map
                join chkblob on chkblob.hash = map.chkblob
                where map.id = $1",
                &[&map_id]).await?;

        let mapblob_hash = row.try_get::<_, String>(0)?;
        let chkhash = row.try_get::<_, String>(1)?;
        let mapblob_size = row.try_get::<_, i64>(2)?;
        let length = row.try_get::<_, i64>(3)? as usize;
        let ver = row.try_get::<_, i64>(4)?;
        let data = row.try_get::<_, Vec<u8>>(5)?;
        let is_nsfw = row.try_get::<_, bool>(6)?;
        let uploaded_by = row.try_get::<_, i64>(7)?;
        let is_blackholed = row.try_get::<_, bool>(8)?;
        let views = row.try_get::<_, i64>(9)?;
        let downloads = row.try_get::<_, i64>(10)?;

        anyhow::ensure!(ver == 1);

        let chkblob = zstd::bulk::decompress(data.as_slice(), length)?;

        anyhow::Ok((
            mapblob_hash,
            mapblob_size,
            chkhash,
            chkblob,
            is_nsfw,
            uploaded_by,
            is_blackholed,
            views,
            downloads,
        ))
    };

    let ph16x16 = async {
        let pool = pool.clone();
        let con = pool.get().await?;
        let row = con
            .query_one(
                "
                select ph16x16 from map
                join minimap on minimap.chkhash = map.chkblob
                where map.id = $1",
                &[&map_id],
            )
            .await?;

        let mapblob_hash = row.try_get::<_, Vec<u8>>(0)?;

        let mut point = [0f32; 256];
        for (index, x) in mapblob_hash.into_iter().enumerate() {
            point[index * 8] = (x & 0x01) as f32;
            point[index * 8 + 1] = ((x & 0x02) >> 1) as f32;
            point[index * 8 + 2] = ((x & 0x04) >> 2) as f32;
            point[index * 8 + 3] = ((x & 0x08) >> 3) as f32;
            point[index * 8 + 4] = ((x & 0x10) >> 4) as f32;
            point[index * 8 + 5] = ((x & 0x20) >> 5) as f32;
            point[index * 8 + 6] = ((x & 0x40) >> 6) as f32;
            point[index * 8 + 7] = ((x & 0x80) >> 7) as f32;
        }

        anyhow::Ok(point)
    };

    let denormalize_key = async {
        let pool = pool.clone();
        let con = pool.get().await?;
        let row = con
            .query_one(
                "
                select count(*) from minimap",
                &[],
            )
            .await?;

        let denormalize_key = row.try_get::<_, i64>(0)?;

        anyhow::Ok(denormalize_key)
    };

    let (
        filenames,
        tags,
        chkblob,
        replays,
        flags,
        current_user,
        filetimes,
        ph16x16,
        denormalize_key,
    ) = futures::try_join!(
        filenames,
        tags,
        chkblob,
        replays,
        flags,
        current_user,
        filetimes,
        ph16x16,
        denormalize_key
    )?;

    let (
        mapblob_hash,
        mapblob_size,
        chkhash,
        chkblob,
        is_nsfw,
        uploaded_by,
        is_blackholed,
        views,
        downloads,
    ) = chkblob;

    if is_nsfw && current_user.is_none() {
        return Ok(HttpResponse::Unauthorized().finish().customize());
    }

    if is_blackholed {
        if let Some(user_id) = current_user {
            if user_id != 4 {
                return Ok(HttpResponse::Unauthorized().finish().customize());
            }
        } else {
            return Ok(HttpResponse::Unauthorized().finish().customize());
        }
    }

    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let kdtree = get_kd_tree(denormalize_key, (**pool).clone()).await?;

    let nearest: Vec<_> = {
        let span = info_span!("nearest");
        let _e = span.enter();
        kdtree
            .iter_nearest(&ph16x16, &|x, y| {
                x.into_iter()
                    .zip(y)
                    .fold(0.0, |a, (&b, &c)| a + (b - c).abs())
            })
            .map_err(|x| anyhow::Error::from(x))?
            .filter(|x| *x.1 != map_id)
            .take_while(|x| x.0 <= 64.0)
            .take(20)
            .collect()
    };

    let mut similar_maps = vec![];

    for (hamming_distance, map_id) in nearest {
        let pool = (**pool).clone();

        similar_maps.push(async move {
            let con = pool.get().await?;

            #[rustfmt::skip]
            let row = con.query_one("
                select
                    chkdenorm.scenario_name,
                    chkdenorm.chkblob,
                    chkdenorm.width,
                    chkdenorm.height,
                    chkdenorm.tileset,
                    map.nsfw,
                    map.outdated,
                    map.unfinished,
                    map.broken,
                    min(filetime.modified_time) as modified_time
                from map
                join chkdenorm on chkdenorm.chkblob = map.chkblob
                left join filetime on filetime.map = map.id
                where map.id = $1
                group by scenario_name, chkdenorm.chkblob, width, height, tileset, nsfw, outdated, unfinished, broken",
                    &[&map_id],
                ).await?;

            let tileset = match row.try_get::<_, i64>("tileset")? % 8 {
                0 => "Badlands",
                1 => "Space Platform",
                2 => "Installation",
                3 => "Ashworld",
                4 => "Jungle",
                5 => "Desert",
                6 => "Arctic",
                7 => "Twilight",
                _ => "Unknown tileset",
            }
            .to_owned();

            if row.try_get::<_, bool>("nsfw")? || row.try_get::<_, bool>("outdated")? || row.try_get::<_, bool>("unfinished")? || row.try_get::<_, bool>("broken")? {
                return Err(anyhow::anyhow!("nsfw"));
            }

            anyhow::Ok(json!( {
                "chkhash": row.try_get::<_, String>("chkblob")?,
                "map_id": bwcommon::get_web_id_from_db_id(*map_id, crate::util::SEED_MAP_ID)?,
                "hamming_distance": hamming_distance as u64,
                "scenario_name": row.try_get::<_, String>("scenario_name")?,
                "width": row.try_get::<_, i64>("width")?,
                "height": row.try_get::<_, i64>("height")?,
                "tileset": tileset,
                "modified_time": row.try_get::<_, i64>("modified_time").unwrap_or(-1),
            }))
        });
    }

    let similar_maps: Vec<_> = futures::future::join_all(similar_maps)
        .await
        .into_iter()
        .filter_map(|x| x.ok())
        // .map(|x| x.unwrap())
        .collect();

    // hb.register_helper(
    //     "json",
    //     Box::new(
    //         |h: &handlebars::Helper,
    //          _r: &handlebars::Handlebars,
    //          _: &handlebars::Context,
    //          _rc: &mut handlebars::RenderContext,
    //          out: &mut dyn handlebars::Output|
    //          -> handlebars::HelperResult {
    //             let param = h
    //                 .param(0)
    //                 .ok_or(handlebars::RenderError::new("param not found"))?;

    //             let obj = if let Some(obj) = param.value().as_object() {
    //                 obj.clone().into_iter().filter(|x| x.0 != "type").collect()
    //             } else {
    //                 serde_json::Map::<String, handlebars::JsonValue>::new()
    //             };

    //             out.write(&serde_json::to_string(&obj)?)?;
    //             Ok(())
    //         },
    //     ),
    // );

    // hb.register_helper(
    //     "debug",
    //     Box::new(
    //         |h: &handlebars::Helper,
    //          _r: &handlebars::Handlebars,
    //          _: &handlebars::Context,
    //          _rc: &mut handlebars::RenderContext,
    //          out: &mut dyn handlebars::Output|
    //          -> handlebars::HelperResult {
    //             let param = h
    //                 .param(0)
    //                 .ok_or(handlebars::RenderError::new("param not found"))?;

    //             out.write(&format!("{:?}", param.value()))?;
    //             Ok(())
    //         },
    //     ),
    // );

    let (scenario_name, scenario_description) = if let Ok(x) = &parsed_chk.sprp {
        let scenario_string = if *x.scenario_name_string_number == 0 {
            "Untitled Scenario".to_owned()
        } else {
            if let Ok(s) = parsed_chk.get_string(*x.scenario_name_string_number as usize) {
                s
            } else {
                "Failed to get scenario string".to_owned()
            }
        };

        let scenario_description_string = if *x.description_string_number == 0 {
            "".to_owned()
        } else {
            if let Ok(s) = parsed_chk.get_string(*x.description_string_number as usize) {
                s
            } else {
                "Failed to get scenario description string string".to_owned()
            }
        };

        (scenario_string, scenario_description_string)
    } else {
        (
            "Untitled Scenario".to_owned(),
            ">>> No SPRP section in chk <<<<".to_owned(),
        )
    };

    let mut details = Vec::new();

    if let Ok(x) = &parsed_chk.ver {
        let version = match x.file_format_version {
            206 => "Remastered 1.21",
            205 => "Broodwar 1.04+",
            64 => "Starcraft Remastered 1.21 hybrid",
            63 => "Starcraft 1.04+ hybrid",
            59 => "Starcraft 1.00",
            61 | 75 | 201 | 203 => "BroodWar Internal",
            47 => "StarCraft Beta",
            0..=19 => "Warcraft II",
            _ => "Unknown Version",
        };
        details.push(json!([
            "Version",
            format!("{} ({})", version, x.file_format_version)
        ]));
    }

    details.push(json!(["MPQ Hash", mapblob_hash]));
    details.push(json!(["MPQ Size", mapblob_size]));
    details.push(json!(["CHK Hash", chkhash]));
    details.push(json!(["CHK Size", chkblob.len()]));

    let mut map_width = 0;
    let mut map_height = 0;

    if let Ok(x) = &parsed_chk.dim {
        map_width = *x.width;
        map_height = *x.height;
        details.push(json!([
            "Map Dimensions",
            format!("{}x{}", x.width, x.height)
        ]));
    }

    let mut map_era = 0;

    if let Ok(x) = &parsed_chk.era {
        map_era = x.tileset % 8;
        details.push(json!([
            "Map Tileset",
            format!(
                "{} ({} mod 8 = {})",
                match x.tileset % 8 {
                    0 => "Badlands",
                    1 => "Space Platform",
                    2 => "Installation",
                    3 => "Ashworld",
                    4 => "Jungle",
                    5 => "Desert",
                    6 => "Arctic",
                    7 => "Twilight",
                    _ => "Unknown tileset",
                },
                x.tileset,
                x.tileset % 8
            )
        ]));
    }

    if let Ok(x) = &parsed_chk.ownr {
        // u8[12]: One byte for each player, specifies the owner of the player:
        // 00 - Inactive
        // 01 - Computer (game)
        // 02 - Occupied by Human Player
        // 03 - Rescue Passive
        // 04 - Unused
        // 05 - Computer
        // 06 - Human (Open Slot)
        // 07 - Neutral
        // 08 - Closed slot
        details.push(json!([
            "Human Players",
            x.player_owner.iter().filter(|&&x| x == 6).count()
        ]));

        details.push(json!([
            "Computer Players",
            x.player_owner.iter().filter(|&&x| x == 5).count()
        ]));

        details.push(json!([
            "OWNR Players",
            x.player_owner
                .iter()
                .fold("".to_owned(), |a, &b| format!("{}{}", a, b))
        ]));
    }

    if let Ok(x) = &parsed_chk.dd2 {
        details.push(json!(["Doodads", x.doodads.len()]));
    }

    if let Ok(x) = &parsed_chk.thg2 {
        details.push(json!(["Sprites", x.sprites.len()]));
    }

    if let Ok(x) = &parsed_chk.trig {
        details.push(json!(["Triggers", x.triggers.len()]));
    }

    if let Ok(x) = &parsed_chk.mbrf {
        details.push(json!(["Briefing Triggers", x.triggers.len()]));
    }

    if let Ok(x) = &parsed_chk.mrgn {
        details.push(json!([
            "Locations",
            x.locations
                .iter()
                .filter(|&&x| !(x.left == x.right || x.top == x.bottom))
                .count()
        ]));
    }

    if let Ok(x) = &parsed_chk.unit {
        details.push(json!(["Units", x.units.len()]));
    }

    let mut mtxm = Vec::new();

    if let Ok(x) = &parsed_chk.mtxm {
        mtxm = x.data.clone();
        let hash_set: std::collections::HashSet<u16> = x.data.iter().cloned().collect();
        details.push(json!(["Unique Terrain Tiles", hash_set.len()]));
    }

    let parsed_triggers = bwmap::parse_triggers(&parsed_chk);

    let mission_briefings = bwmap::parse_mission_briefing(&parsed_chk);

    {
        let mut set = std::collections::HashSet::new();

        for trigger in &parsed_triggers {
            for action in &trigger.actions {
                match action {
                    bwmap::Action::PlayWav { wave, wave_time: _ } => {
                        set.insert(wave);
                    }
                    bwmap::Action::Transmission {
                        text: _,
                        unit_type: _,
                        location: _,
                        time: _,
                        modifier: _,
                        wave,
                        wave_time: _,
                    } => {
                        set.insert(wave);
                    }
                    _ => {}
                }
            }
        }

        for trigger in &mission_briefings {
            for action in &trigger.actions {
                match action {
                    bwmap::MissionBriefingAction::PlayWav { wave, wave_time: _ } => {
                        set.insert(wave);
                    }
                    bwmap::MissionBriefingAction::DisplayTransmission {
                        text: _,
                        slot: _,
                        time: _,
                        modifier: _,
                        wave,
                        wave_time: _,
                    } => {
                        set.insert(wave);
                    }
                    _ => {}
                }
            }
        }

        details.push(json!(["Wav Files", set.len()]));
    }

    let units = if let Ok(x) = &parsed_chk.unix {
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

    let forces = if let Ok(x) = &parsed_chk.forc {
        // pub player_forces: &'a [u8],
        // pub force_name: &'a [u16],
        // pub force_properties: &'a [u8],

        let mut v = Vec::new();

        let mut force_number = 1;

        for force in x.force_name {
            if force == 0 {
                v.push(json!({ "force": format!("Force {force_number}") }))
            } else {
                v.push(json!({
                    "force": parsed_chk.get_string(force as usize).unwrap_or("couldn't get force name".to_owned())
                }))
            }
            force_number += 1;
        }

        v
    } else {
        Vec::new()
    };

    {
        #[allow(non_snake_case)]
        let mut get_deaths_EUDs: i32 = 0;
        #[allow(non_snake_case)]
        let mut get_deaths_EPDs: i32 = 0;
        #[allow(non_snake_case)]
        let mut set_deaths_EUDs: i32 = 0;
        #[allow(non_snake_case)]
        let mut set_deaths_EPDs: i32 = 0;
        #[allow(non_snake_case)]
        let mut EUPs: i32 = 0;

        if let Ok(x) = parsed_chk.unit {
            for unit in &x.units {
                if unit.unit_id > 227 || unit.owner > 27 {
                    EUPs += 1;
                }
            }
        }

        for trigger in &parsed_triggers {
            for condition in &trigger.conditions {
                match condition {
                    bwmap::Condition::Deaths {
                        player,
                        comparison: _,
                        unit_type,
                        number: _,
                        eud_offset: _,
                    } => {
                        match player {
                            bwmap::Group::Unknown(_) => {
                                get_deaths_EPDs += 1;
                            }
                            _ => {}
                        }
                        match unit_type {
                            bwmap::UnitType::Unknown(_) => {
                                get_deaths_EUDs += 1;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            for action in &trigger.actions {
                match action {
                    bwmap::Action::SetDeaths {
                        player,
                        unit_type,
                        number: _,
                        modifier: _,
                        eud_offset: _,
                    } => {
                        match player {
                            bwmap::Group::Unknown(_) => {
                                set_deaths_EPDs += 1;
                            }
                            _ => {}
                        }
                        match unit_type {
                            bwmap::UnitType::Unknown(_) => {
                                set_deaths_EUDs += 1;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        details.push(json!(["GetDeaths EUDs", get_deaths_EUDs]));
        details.push(json!(["GetDeaths EPDs", get_deaths_EPDs]));
        details.push(json!(["SetDeaths EUDs", set_deaths_EUDs]));
        details.push(json!(["SetDeaths EPDs", set_deaths_EPDs]));
        details.push(json!(["EUPs", EUPs]));

        details.push(json!(["Views", views]));
        details.push(json!(["Downloads", downloads]));
    }

    let details: Vec<_> = details
        .into_iter()
        .map(|x| json!({"key": x[0], "value": x[1]}))
        .collect();

    // let minimap_scaling_factor = f32::min(512.0 / map_width as f32, 512.0 / map_height as f32);

    // let minimap_width = map_width as f32 * minimap_scaling_factor;
    // let minimap_height = map_width as f32 * minimap_scaling_factor;

    let new_html = info_span!("rendering").in_scope(|| {
        anyhow::Ok(hb.render(
            "map",
            &json!({
                "minimap_id": chkhash.clone(),
                "minimap_width": map_width,
                "minimap_height": map_height,
                "sanitized_scenario_name": crate::util::sanitize_sc_string(scenario_name.as_str()),
                "sanitized_scenario_description": crate::util::sanitize_sc_string_preserve_newlines(scenario_description.as_str()),
                "scenario_name": scenario_name,
                "scenario_description": scenario_description,
                "filenames": serde_json::to_string(&filenames)?,
                "filenames2": filenames,
                "tags": serde_json::to_string(&tags)?,
                "replays2": replays,
                "flags": serde_json::to_string(&flags)?,
                "filetimes": serde_json::to_string(&filetimes)?,
                "filetimes2": filetimes,
                "show_flags": current_user.is_some() && (current_user.unwrap() == uploaded_by || current_user.unwrap() == 4),
                "details": details,
                "mapblob_hash": mapblob_hash,
                "mapblob_size": mapblob_size,
                "chkblob_hash": chkhash.clone(),
                "units": serde_json::to_string(&units)?,
                "units2": units,
                "forces": serde_json::to_string(&forces)?,
                "forces2": forces,
                "mtxm": mtxm,
                "map_width": serde_json::to_string(&map_width)?,
                "map_height": serde_json::to_string(&map_height)?,
                "map_era": serde_json::to_string(&map_era)?,
                "download_size": format!("{} KB", mapblob_size / 1024),
                "download_filename": filenames[0].filename,
                "similar_maps": similar_maps,
                "langmap": json!({ "navbar": get_navbar_langmap(lang) }),
                "is_logged_in": user_username.1,
                "username": user_username.0,
            }),
        )?)
    })?;

    {
        let time_since_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let pool = pool.clone();
        let con = pool.get().await?;
        let rows = con
            .execute(
                "update map set views = views + 1, last_viewed = $1 where map.id = $2",
                &[&time_since_epoch, &map_id],
            )
            .await?;

        (|| {
            anyhow::ensure!(rows == 1);
            anyhow::Ok(())
        })()?;
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(new_html)
        .customize())
}
