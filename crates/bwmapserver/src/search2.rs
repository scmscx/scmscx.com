use actix_web::web::Data;
use anyhow::Result;
use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use std::cmp::min;

#[cached(
    size = 100,
    time = 3600,
    result = true,
    key = "(String, bool, SearchParams)",
    convert = r#"{ (query.to_owned(), allow_nsfw, { let mut qp = query_params.clone(); qp.offset = 0; qp }) }"#
)]
pub async fn search_cache(
    query: &str,
    allow_nsfw: bool,
    query_params: &SearchParams,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Vec<Map>> {
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

    let (sort, sortorder) = match query_params.sort.as_str() {
        "relevancy" => {
            if query.is_empty() {
                ("uploaded_time", "desc")
            } else {
                ("dist2", "desc")
            }
        }
        "scenario" => ("chkdenorm.scenario_name", "asc"),
        "lastmodifiedold" => ("min(filetime.modified_time)", "asc NULLS FIRST"),
        "lastmodifiednew" => ("min(filetime.modified_time)", "desc NULLS LAST"),
        "timeuploadedold" => ("uploaded_time", "asc NULLS FIRST"),
        "timeuploadednew" => ("uploaded_time", "desc NULLS LAST"),
        _ => {
            return Err(anyhow::anyhow!("Unknown sort: {}", query_params.sort));
        }
    };

    let maps = if query.is_empty() {
        let con = pool.get().await?;

        let qs = format!("
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
                chkdenorm.computer_players >= $8 and chkdenorm.computer_players <= $9 and
                map.uploaded_time <= $10 and map.uploaded_time >= $11 and
                ((modified_time <= $12 and modified_time >= $13) or modified_time is null)
            group by map.id, chkdenorm.scenario_name, uploaded_time, chkdenorm.width, chkdenorm.height, chkdenorm.tileset, chkdenorm.human_players, chkdenorm.computer_players
            order by {} {}
            ", sort, sortorder);

        con.query(
            &qs,
            &[
                &query_params.minimum_map_width,
                &query_params.maximum_map_width,
                &query_params.minimum_map_height,
                &query_params.maximum_map_height,
                &allowed_tilesets,
                &query_params.minimum_human_players,
                &query_params.maximum_human_players,
                &query_params.minimum_computer_players,
                &query_params.maximum_computer_players,
                &(query_params.time_uploaded_before / 1000),
                &(query_params.time_uploaded_after / 1000),
                &(query_params.last_modified_before / 1000),
                &(query_params.last_modified_after / 1000),
            ],
        )
        .await?
        .into_iter()
        .map(|row| {
            anyhow::Ok(Map {
                id: bwcommon::get_web_id_from_db_id(row.try_get(0)?, crate::util::SEED_MAP_ID)?,
                scenario_name: row.try_get(1)?,
                last_modified: row
                    .try_get::<_, Option<i64>>("modified_time")?
                    .unwrap_or(-1),
                uploaded_time: row.try_get::<_, i64>("uploaded_time")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?
    } else {
        let con = pool.get().await?;

        let qs =
                format!("
                select mapid as id, scenario_name, modified_time, uploaded_time, dist2 from (
                    select map.id as mapid, chkdenorm.scenario_name, min(filetime.modified_time) as modified_time, map.uploaded_time, dist2 from (
                        select max(dist*weight) as dist2, id as mapid from (
                            select word_similarity($1, data) as dist, map as id,

                            CASE
							    WHEN scenario_name THEN 1.25
							    WHEN file_names THEN 1.2
							    WHEN scenario_description THEN 1.1
							    WHEN force_names THEN 1.1
							    ELSE 1.0
							end as weight

                            from stringmap2
                            where $1 <% data and ((scenario_name = true and $3) or (scenario_description = true and $4) or (unit_names = true and $5) or (force_names = true and $6) or (file_names = true and $7))
                        ) as sq1
                        group by id
                    ) as sq2
                    join map on map.id = sq2.mapid
                    left join filetime on filetime.map = map.id
                    join chkdenorm on chkdenorm.chkblob = map.chkblob
                    where ($2 or map.nsfw = false) and outdated = false and unfinished = false and broken = false and blackholed = false and
                        chkdenorm.scenario_name is not null and 
                        chkdenorm.width >= $8 and chkdenorm.width <= $9 and
                        chkdenorm.height >= $10 and chkdenorm.height <= $11 and
                        chkdenorm.tileset = any($12) and
                        chkdenorm.human_players >= $13 and chkdenorm.human_players <= $14 and
                        chkdenorm.computer_players >= $15 and chkdenorm.computer_players <= $16 and
                        map.uploaded_time <= $17 and map.uploaded_time >= $18 and
                        ((modified_time <= $19 and modified_time >= $20) or modified_time is null)
                    group by map.id, chkdenorm.scenario_name, dist2
                    order by {} {}
                ) as sq3",  sort, sortorder);

        con.query(
            &qs,
            &[
                &query,
                &allow_nsfw,
                &query_params.scenario_names,
                &query_params.scenario_descriptions,
                &query_params.unit_names,
                &query_params.force_names,
                &query_params.file_names,
                &query_params.minimum_map_width,
                &query_params.maximum_map_width,
                &query_params.minimum_map_height,
                &query_params.maximum_map_height,
                &allowed_tilesets,
                &query_params.minimum_human_players,
                &query_params.maximum_human_players,
                &query_params.minimum_computer_players,
                &query_params.maximum_computer_players,
                &(query_params.time_uploaded_before / 1000),
                &(query_params.time_uploaded_after / 1000),
                &(query_params.last_modified_before / 1000),
                &(query_params.last_modified_after / 1000),
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

    Ok(maps)
}

fn defaultrelevancy() -> String {
    "relevancy".to_owned()
}

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

fn default2524608000000() -> i64 {
    2524608000000
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct SearchParams {
    #[serde(default = "defaultrelevancy")]
    pub(crate) sort: String,

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

    #[serde(default = "default0")]
    pub(crate) last_modified_after: i64,
    #[serde(default = "default2524608000000")]
    pub(crate) last_modified_before: i64,

    #[serde(default = "default0")]
    pub(crate) time_uploaded_after: i64,
    #[serde(default = "default2524608000000")]
    pub(crate) time_uploaded_before: i64,

    #[serde(default = "default0")]
    offset: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Map {
    pub(crate) id: String,
    scenario_name: String,
    last_modified: i64,
    uploaded_time: i64,
}

// pub async fn search(
//     query: &str,
//     allow_nsfw: bool,
//     query_params: &SearchParams,
//     pool: Data<Pool<PostgresConnectionManager<NoTls>>>,
// ) -> Result<Vec<Map>, bwcommon::MyError> {
//     let maps = search_cache(query, allow_nsfw, query_params, (**pool).clone()).await?;

//     if query.len() == 0 {
//         Ok(maps[0..min(maps.len(), 5000)].to_vec())
//     } else {
//         Ok(maps[0..min(maps.len(), 10000)].to_vec())
//     }
// }

pub async fn search2(
    query: &str,
    allow_nsfw: bool,
    query_params: &SearchParams,
    pool: Data<Pool<PostgresConnectionManager<NoTls>>>,
) -> Result<(usize, Vec<Map>), bwcommon::MyError> {
    let maps = search_cache(query, allow_nsfw, query_params, (**pool).clone()).await?;

    let offset: usize = query_params.offset.try_into()?;

    if maps.len() <= offset {
        return Ok((0, vec![]));
    }

    Ok((
        maps.len(),
        maps[query_params.offset as usize..min(query_params.offset as usize + 300, maps.len())]
            .to_vec(),
    ))
}
