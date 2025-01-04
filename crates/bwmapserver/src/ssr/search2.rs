use actix_web::HttpMessage;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web::{HttpRequest, Result};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use std::sync::{Mutex};
use crate::search::SearchDatabase;
use std::collections::HashSet;

// trait MyTrait: r2d2_postgres::postgres::types::ToSql + Sync + std::fmt::Display {
//     fn to_sub(&self) -> &(dyn r2d2_postgres::postgres::types::ToSql + Sync);
// }

// impl<T: r2d2_postgres::postgres::types::ToSql + Sync + std::fmt::Display> MyTrait for T {
//     fn to_sub(&self) -> &(dyn r2d2_postgres::postgres::types::ToSql + Sync) {
//         self
//     }
// }

// trait SuperCached {
//     fn cached_query<F, T, E>(
//         &mut self,
//         query: &str,
//         params: &[&dyn MyTrait],
//         cache: &std::sync::Mutex<lru::LruCache<String, Vec<T>>>,
//         map: F,
//     ) -> Result<Vec<T>, E>
//     where
//         F: FnMut(r2d2_postgres::postgres::row::Row) -> Result<T, E>,
//         T: Clone,
//         E: From<r2d2_postgres::postgres::Error>;
// }

// impl SuperCached
//     for r2d2_postgres::r2d2::PooledConnection<
//         r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>,
//     >
// {
//     fn cached_query<F, T, E>(
//         &mut self,
//         query: &str,
//         params: &[&dyn MyTrait],
//         cache: &std::sync::Mutex<lru::LruCache<String, Vec<T>>>,
//         map: F,
//     ) -> Result<Vec<T>, E>
//     where
//         F: FnMut(r2d2_postgres::postgres::row::Row) -> Result<T, E>,
//         T: Clone,
//         E: From<r2d2_postgres::postgres::Error>,
//     {
//         let mut key = format!("{}", query);

//         for v in params {
//             key = format!("{}+{}", key, v);
//         }

//         {
//             let mut guard = cache.lock().unwrap();
//             if let Some(cached_value) = guard.get(&key.to_string()) {
//                 return Ok(cached_value.clone());
//             }
//         }

//         let mut v: Vec<&(dyn r2d2_postgres::postgres::types::ToSql + Sync)> = Vec::new();
//         for k in params {
//             v.push(k.to_sub());
//         }
//         let vec = self
//             .query(query, v.as_slice())?
//             .into_iter()
//             .map(map)
//             .collect::<Result<Vec<T>, E>>()?;

//         if let Ok(mut guard) = cache.lock() {
//             guard.put(key, vec.clone());
//         }

//         Ok(vec)
//     }
// }

fn defaulttrue() -> bool {
    true
}

fn default0() -> i64 {
    0
}

fn default256() -> i64 {
    256
}

fn default12() -> i64 {
    256
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SearchQuery {
    #[serde(default = "defaulttrue")]
    pub(crate) unit_names: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) force_names: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) file_names: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) scenario_names: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) scenario_descriptions: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) provided_by: bool,

    #[serde(default = "default0")]
    pub(crate) minimum_map_width: i64,
    #[serde(default = "default256")]
    pub(crate) maximum_map_width: i64,
    #[serde(default = "default0")]
    pub(crate) minimum_map_height: i64,
    #[serde(default = "default256")]
    pub(crate) maximum_map_height: i64,

    #[serde(default = "defaulttrue")]
    pub(crate) tileset_badlands: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) tileset_space_platform: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) tileset_installation: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) tileset_ashworld: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) tileset_jungle: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) tileset_desert: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) tileset_ice: bool,
    #[serde(default = "defaulttrue")]
    pub(crate) tileset_twilight: bool,

    #[serde(default = "default0")]
    pub(crate) minimum_human_players: i64,
    #[serde(default = "default12")]
    pub(crate) maximum_human_players: i64,
    #[serde(default = "default0")]
    pub(crate) minimum_computer_players: i64,
    #[serde(default = "default12")]
    pub(crate) maximum_computer_players: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Map {
    id: String,
    scenario_name: String,
    last_modified: i64,
    uploaded_time: i64,
}

#[instrument(skip_all, name = "/search2")]
async fn handler2(
    req: HttpRequest,
    query: String,
    query_params: web::Query<SearchQuery>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
    searchdb: web::Data<Mutex<SearchDatabase>>,
) -> Result<impl Responder, bwcommon::MyError> {
    // lazy_static::lazy_static! {
    //     static ref CACHE: std::sync::Mutex<lru::LruCache::<String, Vec<Map>>> = {
    //         std::sync::Mutex::new(lru::LruCache::<String, Vec<Map>>::new(100))
    //     };
    // }

    // let needs_drop = {
    //     let mut guard = cache_droper.lock().unwrap();
    //     guard.insert(format!("{}{}", file!(), line!()))
    // };

    // if needs_drop {
    //     CACHE.lock().unwrap().clear();
    // }

    // cache_droper: actix_web::web::Data<std::sync::Mutex<std::collections::HashSet<String>>>,

    let mut allowed_tilesets: Vec<i64> = Vec::new();

    if query_params.tileset_badlands {
        allowed_tilesets.push(0);
    }
    if query_params.tileset_space_platform {
        allowed_tilesets.push(1);
    }
    if query_params.tileset_installation {
        allowed_tilesets.push(2);
    }
    if query_params.tileset_ashworld {
        allowed_tilesets.push(3);
    }
    if query_params.tileset_jungle {
        allowed_tilesets.push(4);
    }
    if query_params.tileset_desert {
        allowed_tilesets.push(5);
    }
    if query_params.tileset_ice {
        allowed_tilesets.push(6);
    }
    if query_params.tileset_twilight {
        allowed_tilesets.push(7);
    }

    let allow_nsfw = bwcommon::check_auth4(&req, (**pool).clone())
        .await?
        .is_some();

    let maps = if query.is_empty() {
        let con = pool.get().await?;

        con.query("
            select distinct map.id, chkdenorm.scenario_name, uploaded_time, min(filetime.modified_time) as modified_time, uploaded_time, chkdenorm.width, chkdenorm.height, chkdenorm.tileset, chkdenorm.human_players, chkdenorm.computer_players from map
            left join filetime on filetime.map = map.id
            join chkdenorm on chkdenorm.chkblob = map.chkblob
            where
                nsfw = false and
                outdated = false and
                unfinished = false and
                broken = false and
                blackholed = false and
                chkdenorm.scenario_name is not null and 
                chkdenorm.width >= $1 and chkdenorm.width <= $2 and
                chkdenorm.height >= $3 and chkdenorm.height <= $4 and
                chkdenorm.tileset = any($5) and
                chkdenorm.human_players >= $6 and chkdenorm.human_players <= $7 and
                chkdenorm.computer_players >= $8 and chkdenorm.computer_players <= $9
            group by map.id, chkdenorm.scenario_name, uploaded_time, chkdenorm.width, chkdenorm.height, chkdenorm.tileset, chkdenorm.human_players, chkdenorm.computer_players
            order by uploaded_time desc
            limit 5000
            ", &[
                &query_params.minimum_map_width,
                &query_params.maximum_map_width,
                &query_params.minimum_map_height,
                &query_params.maximum_map_height,
                &allowed_tilesets,
                &query_params.minimum_human_players,
                &query_params.maximum_human_players,
                &query_params.minimum_computer_players,
                &query_params.maximum_computer_players,
            ]).await?.into_iter().map(|row|
            {
                anyhow::Ok(Map {
                    id: bwcommon::get_web_id_from_db_id(row.try_get(0)?, crate::util::SEED_MAP_ID)?,
                    scenario_name: row.try_get(1)?,
                    last_modified: row.try_get::<_, Option<i64>>("modified_time")?.unwrap_or(-1),
                    uploaded_time: row.try_get::<_, i64>("uploaded_time")?,
                })
            }).collect::<Result<Vec<_>, _>>()?
    } else {

        let mut map_ids: HashSet<i64> = HashSet::new();

        if query_params.scenario_names {
            let lock = searchdb.lock().unwrap();
            map_ids.extend(lock.scenario_name.get(query.as_str()).iter());
        }

        if query_params.scenario_descriptions {
            let lock = searchdb.lock().unwrap();
            map_ids.extend(lock.scenario_description.get(query.as_str()).iter());
        }

        if query_params.unit_names {
            let lock = searchdb.lock().unwrap();
            map_ids.extend(lock.unit_names.get(query.as_str()).iter());
        }

        if query_params.force_names {
            let lock = searchdb.lock().unwrap();
            map_ids.extend(lock.force_names.get(query.as_str()).iter());
        }

        if query_params.file_names {
            let lock = searchdb.lock().unwrap();
            map_ids.extend(lock.file_names.get(query.as_str()).iter());
        }

        let map_ids: Vec<_> = map_ids.into_iter().collect();

        let con = pool.get().await?;

        let qs = 
                "
                select mapid as id, scenario_name, modified_time, uploaded_time from (
                    select map.id as mapid, chkdenorm.scenario_name, min(filetime.modified_time) as modified_time, map.uploaded_time from (
                        select unnest($1::int8[]) as mapid
                    ) as sq
                    join map on map.id = sq.mapid
                    left join filetime on filetime.map = map.id
                    join chkdenorm on chkdenorm.chkblob = map.chkblob
                    where ($2 or map.nsfw = false) and outdated = false and unfinished = false and broken = false and
                        chkdenorm.scenario_name is not null and 
                        chkdenorm.width >= $3 and chkdenorm.width <= $4 and
                        chkdenorm.height >= $5 and chkdenorm.height <= $6 and
                        chkdenorm.tileset = any($7) and
                        chkdenorm.human_players >= $8 and chkdenorm.human_players <= $9 and
                        chkdenorm.computer_players >= $10 and chkdenorm.computer_players <= $11
                    group by map.id, chkdenorm.scenario_name
                    -- order by dist2 desc
                ) as sq3";

        con.query(
            qs,
            &[
                &map_ids,
                &allow_nsfw,
                &query_params.minimum_map_width,
                &query_params.maximum_map_width,
                &query_params.minimum_map_height,
                &query_params.maximum_map_height,
                &allowed_tilesets,
                &query_params.minimum_human_players,
                &query_params.maximum_human_players,
                &query_params.minimum_computer_players,
                &query_params.maximum_computer_players,
            ],
        )
        .await?
        .into_iter()
        .map(|row| {
            anyhow::Ok(Map {
                id: bwcommon::get_web_id_from_db_id(row.try_get(0)?, crate::util::SEED_MAP_ID)?,
                scenario_name: row.try_get(1)?,
                last_modified: row.try_get::<_, Option<i64>>(2)?.unwrap_or(-1),
                uploaded_time: row.try_get::<_, i64>(3)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?
    };

    let lang = req
        .extensions()
        .get::<bwcommon::LangData>()
        .clone()
        .unwrap_or(&bwcommon::LangData::English)
        .to_owned();

    let page_title = if query.is_empty() {
        "Search StarCraft: Brood War Maps".to_owned()
    } else {
        format!("{} maps found for: {}", maps.len(), query)
    };

    let langmap = if lang == bwcommon::LangData::Korean {
        serde_json::json!({
            "h1": "데이터베이스의 100,000개 지도에서 지도 이름, 부대 이름, 설명 및 부대 이름 검색",
            "h4_try_popular_searches": "인기 검색어 시도",
            "h4_did_you_make_maps": "지도를 만드셨나요? 사용한 이름을 검색해 보세요",
            "random_button": "무작위의",
            "search_button": "검색",
            "unit_names": "단위 이름",
            "force_names": "포스 이름",
            "file_names": "파일 이름",
            "scenario_names": "시나리오 이름",
            "scenario_descriptions": "시나리오 설명",
            "results": "결과",
            "scenario": "대본",
            "last_modified_time": "마지막 수정 시간",
            "uploaded_time": "업로드 시간",
        })
    } else {
        serde_json::json!({
            "h1": "Search map names, unit names, descriptions, and force names across over 100,000 maps in the database",
            "h4_try_popular_searches": "Try Popular searches",
            "h4_did_you_make_maps": "Did you make maps? Try searching the name you used",
            "random_button": "Random",
            "search_button": "Search",
            "unit_names": "Unit Names",
            "force_names": "Force Names",
            "file_names": "File Names",
            "scenario_names": "Scenario Names",
            "scenario_descriptions": "Scenario Descriptions",
            "results": "Results",
            "scenario": "Scenario",
            "last_modified_time": "Last Modified Time",
            "uploaded_time": "Uploaded Time",
        })
    };

    let new_html = hb.render(
        "search",
        &serde_json::json!({
            "page_title": page_title,
            "search_results": serde_json::to_string(&maps)?,
            "langmap": langmap,
        }),
    )?;

    Ok(HttpResponse::Ok().content_type("text/html").body(new_html))
}

#[get("/uiv1/search2/{query}")]
async fn handler(
    req: HttpRequest,
    path: web::Path<(String,)>,
    query_params: web::Query<SearchQuery>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
    searchdb: web::Data<Mutex<SearchDatabase>>,
) -> Result<impl Responder, bwcommon::MyError> {
    let (query,) = path.into_inner();
    handler2(req, query, query_params, pool, hb, searchdb).await
}

#[get("/uiv1/search2")]
async fn handler_empty_query(
    req: HttpRequest,
    query_params: web::Query<SearchQuery>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
    searchdb: web::Data<Mutex<SearchDatabase>>,
) -> Result<impl Responder, bwcommon::MyError> {
    handler2(req, "".to_string(), query_params, pool, hb, searchdb).await
}
