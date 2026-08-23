use crate::webutil::PoolExt;
use anyhow::Result;
use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::sync::Arc;
use std::time::Duration;

/// Public entry point. In dev mode this goes straight to the database.
///
/// The rows come back as an `Arc<[Map]>` rather than a `Vec<Map>` because the cache
/// clones its value on every hit, and a whole-database search holds hundreds of
/// thousands of rows: as a `Vec` that clone was a per-request deep copy of every row
/// and every string in it, paid even by searches that never touch the database.
/// Behind an `Arc` it is a refcount bump, and only the page handed to the caller is
/// actually copied.
///
/// The cache below holds a result for an hour, which is the right call in
/// production but makes local work confusing: upload a map, search for it, and
/// it is missing until the entry expires -- the same query keeps answering from
/// before the upload with no sign that it is doing so.
pub async fn search_cache(
    query: &str,
    allow_nsfw: bool,
    query_params: &SearchParams,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Arc<[Map]>> {
    if crate::util::is_dev_mode() {
        search_uncached(query, allow_nsfw, query_params, pool).await
    } else {
        search_cached(query, allow_nsfw, query_params, pool).await
    }
}

#[cached(
    size = 100,
    time = 3600,
    result = true,
    key = "(String, bool, SearchParams)",
    convert = r#"{ (query.to_owned(), allow_nsfw, { let mut qp = query_params.clone(); qp.offset = 0; qp }) }"#
)]
async fn search_cached(
    query: &str,
    allow_nsfw: bool,
    query_params: &SearchParams,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Arc<[Map]>> {
    search_uncached(query, allow_nsfw, query_params, pool).await
}

/// The predicate that decides which of a map's filenames become result rows.
///
/// The keyword branch knows which strings the query actually matched, and a map
/// that matched on one of its filenames lists only the filenames that matched --
/// searching "bgh" should not answer with the same map's `big game v2.scx`. A map
/// that matched on something else (its scenario name, a unit name, ...) has a NULL
/// `matched_filenames` and keeps all of its filenames. The empty-query branch has
/// nothing to match against, so every filename is a row.
///
/// `alias` is the `filename` table alias the predicate is written against, because
/// [`filename_rows`] applies it under two different ones. It is `&'static str`
/// rather than `&str` so that only a literal in this file can reach the SQL:
/// anything a request carries arrives as an owned `String` and will not compile
/// here. Same rule as `flags::validate_flag`.
fn filename_filter(has_matched_filenames: bool, alias: &'static str) -> String {
    if has_matched_filenames {
        format!("(sq2.matched_filenames is null or {alias}.filename = any(sq2.matched_filenames))")
    } else {
        "true".to_owned()
    }
}

/// A correlated subquery producing one row -- filename and last-modified time --
/// per known filename of `map.id`. This is the unit of a search result: a map with
/// three known filenames is three rows, each carrying the time that belongs to
/// *its* file rather than one time smeared across the whole map.
///
/// Joined with LEFT JOIN LATERAL ... ON TRUE, so a map we have no filename for at
/// all still produces its one row, with a NULL filename.
fn filename_rows(has_matched_filenames: bool) -> String {
    let filter = filename_filter(has_matched_filenames, "f");
    // The `not exists` below re-applies the filter under its own alias.
    let filter_inner = filename_filter(has_matched_filenames, "f2");

    format!(
        "
            -- Preferred source: filenames2 records a modified time per (map,
            -- filename) pair. The same filename can be observed with several
            -- times (different copies of the same file); collapse those to the
            -- oldest, the same convention the map-wide time used. A filename
            -- observed only with unknown times falls back to the map-wide one.
            select f.filename,
                   coalesce(
                       extract(epoch from min(fn2.modified_time))::int8,
                       (select min(ft.modified_time) from filetime ft where ft.map = map.id)
                   ) as modified_time
            from filenames2 fn2
            join filename f on f.id = fn2.filename_id
            where fn2.map_id = map.id and {filter}
            group by f.filename

            union all

            -- Fallback for maps uploaded before filenames2 existed: mapfilename
            -- knows the filenames but not which time belongs to which, so every
            -- row shows the map-wide oldest -- exactly what the search used to
            -- show for the whole map.
            select distinct f.filename,
                   (select min(ft.modified_time) from filetime ft where ft.map = map.id)
            from mapfilename mf
            join filename f on f.id = mf.filename
            where mf.map = map.id and {filter}
              and not exists (
                  select 1
                  from filenames2 fn2
                  join filename f2 on f2.id = fn2.filename_id
                  where fn2.map_id = map.id and {filter_inner}
              )
        "
    )
}

async fn search_uncached(
    query: &str,
    allow_nsfw: bool,
    query_params: &SearchParams,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Arc<[Map]>> {
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

    // `sort` is the one request-carried value that reaches the query *text* rather
    // than a bind parameter, so the match launders it into a literal. The
    // annotation is what enforces that: without it the type is inferred, and an arm
    // that handed back a slice of `query_params.sort` would still compile.
    let (sort, sortorder): (&'static str, &'static str) = match query_params.sort.as_str() {
        "relevancy" => {
            if query.is_empty() {
                ("uploaded_time", "desc")
            } else {
                ("dist2", "desc")
            }
        }
        // `scenario` and `filename` are the ascending sorts; their names predate
        // the descending variants, so they keep them.
        "scenario" => ("chkdenorm.scenario_name", "asc"),
        "scenariodesc" => ("chkdenorm.scenario_name", "desc"),
        "filename" => ("filerow.filename", "asc"),
        "filenamedesc" => ("filerow.filename", "desc"),
        "lastmodifiedold" => ("filerow.modified_time", "asc NULLS FIRST"),
        "lastmodifiednew" => ("filerow.modified_time", "desc NULLS LAST"),
        "timeuploadedold" => ("uploaded_time", "asc NULLS FIRST"),
        "timeuploadednew" => ("uploaded_time", "desc NULLS LAST"),
        _ => {
            return Err(anyhow::anyhow!("Unknown sort: {}", query_params.sort));
        }
    };

    // Rows of one map differ only in the filename, so keep them together and in a
    // fixed order instead of letting them scatter through a tie in the sort key.
    // The filename sorts already lead with the filename, which makes the trailing
    // copy redundant -- but a redundant sort key is only ever consulted once the
    // ones before it have tied, and filename plus map.id tying means the rows are
    // identical. Spelling it unconditionally is the same ordering and the same
    // work, without a branch whose two sides no caller can tell apart.
    let order_by = format!("{sort} {sortorder}, map.id, filerow.filename");

    let maps = if query.is_empty() {
        let con = pool.acquire().await?;

        let rows = filename_rows(false);
        let qs = format!("
            select map.id, chkdenorm.scenario_name, filerow.filename, filerow.modified_time, map.uploaded_time from map
            join chkdenorm on chkdenorm.chkblob = map.chkblob
            left join account on account.id = map.uploaded_by
            left join lateral ({rows}) filerow on true
            where
                (map.nsfw = false or ($14 = true and $15 = true)) and
                (map.outdated = false or $16 = true) and
                (map.unfinished = false or $17 = true) and
                (map.broken = false or $18 = true) and
                map.blackholed = false and
                chkdenorm.scenario_name is not null and
                chkdenorm.width >= $1 and chkdenorm.width <= $2 and
                chkdenorm.height >= $3 and chkdenorm.height <= $4 and
                chkdenorm.tileset = any($5) and
                chkdenorm.human_players >= $6 and chkdenorm.human_players <= $7 and
                chkdenorm.computer_players >= $8 and chkdenorm.computer_players <= $9 and
                map.uploaded_time <= $10 and map.uploaded_time >= $11 and
                ((filerow.modified_time <= $12 and filerow.modified_time >= $13) or filerow.modified_time is null) and
                ($19 = '' or account.username = $19)
            order by {order_by}
            ");

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
                &query_params.include_nsfw,
                &allow_nsfw,
                &query_params.include_outdated,
                &query_params.include_unfinished,
                &query_params.include_broken,
                &query_params.uploaded_by,
            ],
        )
        .await?
        .into_iter()
        .map(|row| {
            anyhow::Ok(Map {
                id: bwcommon::get_web_id_from_db_id(row.try_get("id")?, crate::util::SEED_MAP_ID)?,
                scenario_name: row.try_get("scenario_name")?,
                filename: row.try_get("filename")?,
                last_modified: row
                    .try_get::<_, Option<i64>>("modified_time")?
                    .unwrap_or(-1),
                uploaded_time: row.try_get::<_, i64>("uploaded_time")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?
    } else {
        let con = pool.acquire().await?;

        let rows = filename_rows(true);
        let qs =
                format!("
                select map.id, chkdenorm.scenario_name, filerow.filename, filerow.modified_time, map.uploaded_time from (
                    select max(dist*weight) as dist2, id as mapid,
                           array_agg(distinct data) filter (where file_names) as matched_filenames
                    from (
                        select word_similarity($1, data) as dist, map as id, data, file_names,

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
                join chkdenorm on chkdenorm.chkblob = map.chkblob
                left join account on account.id = map.uploaded_by
                left join lateral ({rows}) filerow on true
                where (map.nsfw = false or ($2 = true and $21 = true)) and (map.outdated = false or $22 = true) and (map.unfinished = false or $23 = true) and (map.broken = false or $24 = true) and map.blackholed = false and
                    chkdenorm.scenario_name is not null and
                    chkdenorm.width >= $8 and chkdenorm.width <= $9 and
                    chkdenorm.height >= $10 and chkdenorm.height <= $11 and
                    chkdenorm.tileset = any($12) and
                    chkdenorm.human_players >= $13 and chkdenorm.human_players <= $14 and
                    chkdenorm.computer_players >= $15 and chkdenorm.computer_players <= $16 and
                    map.uploaded_time <= $17 and map.uploaded_time >= $18 and
                    ((filerow.modified_time <= $19 and filerow.modified_time >= $20) or filerow.modified_time is null) and
                    ($25 = '' or account.username = $25)
                order by {order_by}");

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
                &query_params.include_nsfw,
                &query_params.include_outdated,
                &query_params.include_unfinished,
                &query_params.include_broken,
                &query_params.uploaded_by,
            ],
        )
        .await?
        .into_iter()
        .map(|row| {
            anyhow::Ok(Map {
                id: bwcommon::get_web_id_from_db_id(row.try_get("id")?, crate::util::SEED_MAP_ID)?,
                scenario_name: row.try_get("scenario_name")?,
                filename: row.try_get("filename")?,
                last_modified: row
                    .try_get::<_, Option<i64>>("modified_time")?
                    .unwrap_or(-1),
                uploaded_time: row.try_get::<_, i64>("uploaded_time")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?
    };

    Ok(maps.into())
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

fn defaultempty() -> String {
    String::new()
}

fn defaultfalse() -> bool {
    false
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

    #[serde(default = "defaultempty")]
    pub(crate) uploaded_by: String,

    #[serde(default = "defaultfalse")]
    pub(crate) include_broken: bool,
    #[serde(default = "defaultfalse")]
    pub(crate) include_outdated: bool,
    #[serde(default = "defaultfalse")]
    pub(crate) include_unfinished: bool,
    #[serde(default = "defaultfalse")]
    pub(crate) include_nsfw: bool,
}

/// One search result row: a map as it is known under one particular filename.
/// The same map appears once per filename we know for it.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Map {
    pub(crate) id: String,
    scenario_name: String,
    /// The filename this row stands for, or null for a map we know no filename
    /// for at all.
    filename: Option<String>,
    /// The modified time of *this* filename, or -1 when we have none.
    last_modified: i64,
    uploaded_time: i64,
}

/// The total number of rows a search matched, and the rows themselves starting at
/// `query_params.offset`.
pub async fn search2(
    query: &str,
    allow_nsfw: bool,
    query_params: &SearchParams,
    pool: crate::webutil::Pool,
) -> Result<(usize, Vec<Map>), bwcommon::MyError> {
    let maps = search_cache(query, allow_nsfw, query_params, pool.clone()).await?;

    let offset: usize = query_params.offset.try_into()?;

    if maps.len() <= offset {
        return Ok((0, vec![]));
    }

    Ok((
        maps.len(),
        maps[offset..min(offset + 300, maps.len())].to_vec(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_params_defaults_from_empty_object() {
        // The search handlers deserialize SearchParams straight from the query
        // string; an omitted field must fall back to its documented default.
        let p: SearchParams = serde_json::from_str("{}").unwrap();

        assert_eq!(p.sort, "relevancy");

        // Every search-field toggle defaults on.
        assert!(p.unit_names && p.force_names && p.file_names);
        assert!(p.scenario_names && p.scenario_descriptions && p.provided_by);

        // Every tileset defaults on.
        assert!(p.tileset_badlands && p.tileset_space_platform && p.tileset_installation);
        assert!(p.tileset_ashworld && p.tileset_jungle && p.tileset_desert);
        assert!(p.tileset_ice && p.tileset_twilight);

        // Numeric bounds.
        assert_eq!(p.minimum_map_width, 0);
        assert_eq!(p.maximum_map_width, 256);
        assert_eq!(p.minimum_map_height, 0);
        assert_eq!(p.maximum_map_height, 256);
        assert_eq!(p.minimum_human_players, 0);
        assert_eq!(p.maximum_human_players, 256);
        assert_eq!(p.last_modified_after, 0);
        assert_eq!(p.last_modified_before, 2_524_608_000_000);
        assert_eq!(p.time_uploaded_before, 2_524_608_000_000);
        assert_eq!(p.offset, 0);

        // "include" flags and uploaded_by default off/empty.
        assert!(!p.include_broken && !p.include_outdated);
        assert!(!p.include_unfinished && !p.include_nsfw);
        assert_eq!(p.uploaded_by, "");
    }

    #[test]
    fn search_params_honors_overrides() {
        let p: SearchParams = serde_json::from_str(
            r#"{"sort":"scenario","tileset_jungle":false,"maximum_map_width":128,
                "include_nsfw":true,"uploaded_by":"neo","offset":300}"#,
        )
        .unwrap();

        assert_eq!(p.sort, "scenario");
        assert!(!p.tileset_jungle);
        assert!(
            p.tileset_badlands,
            "unspecified tilesets keep their default"
        );
        assert_eq!(p.maximum_map_width, 128);
        assert!(p.include_nsfw);
        assert_eq!(p.uploaded_by, "neo");
        assert_eq!(p.offset, 300);
    }
}
