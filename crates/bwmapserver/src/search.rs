use crate::util::{process_iter_async_concurrent, sanitize_sc_string_preserve_newlines};
use actix_web::web;
use anyhow::Result;
use bwcommon::ApproximateSet;
use bwmap::ParsedChk;
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::Mutex};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SearchDatabase {
    pub scenario_name: ApproximateSet,
    pub scenario_description: ApproximateSet,
    pub unit_names: ApproximateSet,
    pub force_names: ApproximateSet,
    pub file_names: ApproximateSet,
}

impl SearchDatabase {
    pub fn _new() -> SearchDatabase {
        SearchDatabase {
            scenario_name: ApproximateSet::new(10_000_000),
            scenario_description: ApproximateSet::new(10_000_000),
            unit_names: ApproximateSet::new(10_000_000),
            force_names: ApproximateSet::new(10_000_000),
            file_names: ApproximateSet::new(10_000_000),
        }
    }
}

pub(crate) async fn _populate_search_database(
    set: web::Data<Mutex<SearchDatabase>>,
    pool: bb8_postgres::bb8::Pool<
        bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
    >,
) -> Result<()> {
    let mut con = pool.get().await?;

    // try to load stringdb from db
    let serialized_db_row = con
        .query_one("select data, max_map_id from cache", &[])
        .await;

    let max_map_id = if let Ok(serialized_db_row) = serialized_db_row {
        let serialized_string_db = serialized_db_row.try_get::<_, Vec<u8>>("data")?;
        let serialized_string_db_max_map_id = serialized_db_row.try_get::<_, i64>("max_map_id")?;

        let cursor = Cursor::new(serialized_string_db);
        let serialized_string_db = zstd::decode_all(cursor)?;

        let deserialized_set =
            bincode::deserialize::<SearchDatabase>(serialized_string_db.as_slice());

        if let Ok(deserialized_set) = deserialized_set {
            let x = set.clone();
            let mut lock = x.lock().unwrap();
            *lock = deserialized_set;
            serialized_string_db_max_map_id
        } else {
            0
        }
    } else {
        0
    };

    let map_ids = con
        .query(
            "Select map.id from map where chkblob is not null and map.id > $1",
            &[&max_map_id],
        )
        .await?
        .into_iter()
        .map(|x| anyhow::Ok(x.try_get::<_, i64>(0)?))
        .collect::<Result<Vec<_>>>()?;

    async fn process(
        (pool, set): (
            bb8_postgres::bb8::Pool<
                bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
            >,
            web::Data<Mutex<SearchDatabase>>,
        ),
        map_id: &i64,
    ) -> Result<()> {
        let con = pool.get().await?;

        let chk_hash: String = con
            .query_one("select chkblob from map where map.id = $1", &[&map_id])
            .await?
            .try_get(0)?;

        let chk_blob = {
            let row = con
                .query_one(
                    "
                    select length, ver, data
                    from chkblob
                    where hash = $1",
                    &[&chk_hash],
                )
                .await?;

            let length = row.try_get::<_, i64>("length")? as usize;
            let ver = row.try_get::<_, i64>("ver")?;
            let data = row.try_get::<_, Vec<u8>>("data")?;

            anyhow::ensure!(ver == 1);
            zstd::bulk::decompress(data.as_slice(), length)?
        };

        let parsed_chk = ParsedChk::from_bytes(chk_blob.as_slice());

        let unit_names = if let Ok(x) = &parsed_chk.unix {
            let mut v = Vec::new();

            for unit_id in 0..x.config.len() {
                if x.config[unit_id] == 0 && x.string_number[unit_id] != 0 {
                    if let Ok(x) = parsed_chk.get_string(x.string_number[unit_id] as usize) {
                        v.push(x);
                    }
                }
            }

            Some(v)
        } else if let Ok(x) = &parsed_chk.unis {
            let mut v = Vec::new();

            for unit_id in 0..x.config.len() {
                if x.config[unit_id] == 0 && x.string_number[unit_id] != 0 {
                    if let Ok(x) = parsed_chk.get_string(x.string_number[unit_id] as usize) {
                        v.push(x);
                    }
                }
            }

            Some(v)
        } else {
            None
        };

        let force_names = if let Ok(x) = &parsed_chk.forc {
            let mut v = Vec::new();

            for string_number in x.force_name {
                if *string_number != 0 {
                    if let Ok(string) = parsed_chk.get_string(*string_number as usize) {
                        if string == "Force 1"
                            || string == "Force 2"
                            || string == "Force 3"
                            || string == "Force 4"
                        {
                            continue;
                        }

                        v.push(sanitize_sc_string_preserve_newlines(string.as_str()));
                    }
                }
            }

            Some(v)
        } else {
            None
        };

        let (scenario_name, scenario_description) = if let Ok(x) = &parsed_chk.sprp {
            let scenario_string = if *x.scenario_name_string_number == 0 {
                None
            } else {
                if let Ok(s) = parsed_chk.get_string(*x.scenario_name_string_number as usize) {
                    Some(sanitize_sc_string_preserve_newlines(s.as_str()))
                } else {
                    None
                }
            };

            let scenario_description_string = if *x.description_string_number == 0 {
                None
            } else {
                if let Ok(s) = parsed_chk.get_string(*x.description_string_number as usize) {
                    Some(sanitize_sc_string_preserve_newlines(s.as_str()))
                } else {
                    None
                }
            };

            (scenario_string, scenario_description_string)
        } else {
            (None, None)
        };

        // get filenames for this map
        let file_names: Vec<String> = con
            .query(
                "
            select filename.filename from map
            join mapfilename on mapfilename.map = map.id
            join filename on filename.id = mapfilename.filename
            where map.id = $1",
                &[&map_id],
            )
            .await?
            .into_iter()
            .map(|x| anyhow::Ok(x.try_get::<_, String>(0)?))
            .collect::<Result<_>>()?;

        fn create_search_vector(s: &str) -> Vec<String> {
            let mut vec = Vec::new();

            fn to_lower(c: char) -> char {
                match c {
                    'A' => 'a',
                    'B' => 'b',
                    'C' => 'c',
                    'D' => 'd',
                    'E' => 'e',
                    'F' => 'f',
                    'G' => 'g',
                    'H' => 'h',
                    'I' => 'i',
                    'J' => 'j',
                    'K' => 'k',
                    'L' => 'l',
                    'M' => 'm',
                    'N' => 'n',
                    'O' => 'o',
                    'P' => 'p',
                    'Q' => 'q',
                    'R' => 'r',
                    'S' => 's',
                    'T' => 't',
                    'U' => 'u',
                    'V' => 'v',
                    'W' => 'w',
                    'X' => 'x',
                    'Y' => 'y',
                    'Z' => 'z',
                    _ => c,
                }
            }

            let x = s.chars().collect::<Vec<_>>();

            // for len in 1..=std::cmp::min(x.len(), 10) {
            for len in 1..=std::cmp::min(x.len(), 8) {
                for start in 0..=(x.len() - len) {
                    let s = x[start..start + len]
                        .into_iter()
                        .cloned()
                        .map(to_lower)
                        .collect::<String>();

                    if s.contains(" ") {
                        continue;
                    }

                    if s.contains("\n") {
                        continue;
                    }

                    vec.push(
                        x[start..start + len]
                            .into_iter()
                            .cloned()
                            .map(to_lower)
                            .collect::<String>(),
                    );
                }
            }

            vec
        }

        let scenario_name = scenario_name.map(|x| create_search_vector(x.as_str()));
        let scenario_description = scenario_description.map(|x| create_search_vector(x.as_str()));
        let unit_names = unit_names.map(|x| {
            x.into_iter()
                .map(|x| create_search_vector(x.as_str()))
                .flatten()
                .collect::<Vec<_>>()
        });
        let force_names = force_names.map(|x| {
            x.into_iter()
                .map(|x| create_search_vector(x.as_str()))
                .flatten()
                .collect::<Vec<_>>()
        });
        let file_names = Some(
            file_names
                .into_iter()
                .map(|x| create_search_vector(x.as_str()))
                .flatten()
                .collect::<Vec<_>>(),
        );

        if let Some(scenario_name) = scenario_name {
            let mut set = set.lock().unwrap();
            for s in scenario_name {
                set.scenario_name.insert(s.as_str(), *map_id);
            }
        }

        if let Some(scenario_description) = scenario_description {
            let mut set = set.lock().unwrap();
            for s in scenario_description {
                set.scenario_description.insert(s.as_str(), *map_id);
            }
        }

        if let Some(unit_names) = unit_names {
            let mut set = set.lock().unwrap();
            for s in unit_names {
                set.unit_names.insert(s.as_str(), *map_id);
            }
        }

        if let Some(force_names) = force_names {
            let mut set = set.lock().unwrap();
            for s in force_names {
                set.force_names.insert(s.as_str(), *map_id);
            }
        }

        if let Some(file_names) = file_names {
            let mut set = set.lock().unwrap();
            for s in file_names {
                set.file_names.insert(s.as_str(), *map_id);
            }
        }

        anyhow::Ok(())
    }

    process_iter_async_concurrent(
        map_ids.iter(),
        || (pool.clone(), set.clone()),
        256,
        |x, y| info!("Completed: {}, ret: {:?}", x, y),
        process,
    )
    .await;

    // find max map id
    let max = map_ids.iter().max();

    if let Some(max) = max {
        let serialized_db = {
            let lock = set.lock().unwrap();

            bincode::serialize(&*lock)
        };

        if let Ok(serialized_db) = serialized_db {
            info!("serialized_db size: {}", serialized_db.len());

            let cursor = Cursor::new(serialized_db);
            // compress
            let encoded_result = web::block(|| zstd::encode_all(cursor, 5)).await??;

            info!("encoded_db size: {}", encoded_result.len());

            let tx = con.transaction().await?;

            tx.execute("delete from cache", &[]).await?;
            tx.execute(
                "insert into cache (data, max_map_id) values($1, $2)",
                &[&encoded_result, &max],
            )
            .await?;

            tx.commit().await?;
        }
    }

    anyhow::Ok(())
}
