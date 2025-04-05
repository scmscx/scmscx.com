use crate::middleware::UserSession;
use actix_web::get;
use actix_web::web;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use bwcommon::insert_extension;
use bwcommon::ApiSpecificInfoForLogging;
use bwmap::ParsedChk;
use serde::Serialize;
use serde_json::json;

#[get("/api/uiv2/map_info/{map_id}")]
async fn map_info(
    req: HttpRequest,
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

    let (
        chkblob,
        chkhash,
        chk_size,
        mpq_hash,
        mpq_size,
        uploaded_by,
        uploaded_by_username,
        uploaded_time,
        last_viewed,
        last_downloaded,
        views,
        downloads,
        nsfw,
        blackholed,
    ) = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select
                    length,
                    ver,
                    data,
                    map.chkblob,
                    map.mapblob2,
                    map.mapblob_size,
                    uploaded_time,
                    uploaded_by,
                    account.username,
                    last_viewed,
                    last_downloaded,
                    views,
                    downloads,
                    nsfw,
                    blackholed
                from map
                join chkblob on chkblob.hash = map.chkblob
                left join account on map.uploaded_by = account.id
                where map.id = $1
                ",
                &[&map_id],
            )
            .await?;

        // Downloads, Views, Last Downloaded, Last Viewed

        let length = row.try_get::<_, i64>("length")? as usize;
        let ver = row.try_get::<_, i64>("ver")?;
        let data = row.try_get::<_, Vec<u8>>("data")?;

        bwcommon::ensure!(ver == 1);

        (
            zstd::bulk::decompress(data.as_slice(), length)?,
            row.try_get::<_, String>("chkblob")?,
            length,
            row.try_get::<_, String>("mapblob2")?,
            row.try_get::<_, i64>("mapblob_size")?,
            row.try_get::<_, i64>("uploaded_by")?,
            row.try_get::<_, String>("username")?,
            row.try_get::<_, i64>("uploaded_time")?,
            row.try_get::<_, Option<i64>>("last_viewed")?,
            row.try_get::<_, Option<i64>>("last_downloaded")?,
            row.try_get::<_, i64>("views")?,
            row.try_get::<_, i64>("downloads")?,
            row.try_get::<_, bool>("nsfw")?,
            row.try_get::<_, bool>("blackholed")?,
        )
    };

    let user_id = req.extensions().get::<UserSession>().map(|x| x.id);

    if nsfw && user_id == None {
        return Ok(HttpResponse::Forbidden().finish().customize());
    }

    if blackholed && user_id != Some(uploaded_by) && user_id != Some(4) {
        return Ok(HttpResponse::NotFound().finish().customize());
    }

    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let ver = if let Ok(x) = &parsed_chk.ver {
        Some(*x.file_format_version)
    } else {
        None
    };

    let (width, height) = if let Ok(x) = &parsed_chk.dim {
        (Some(*x.width as i64), Some(*x.height as i64))
    } else {
        (None, None)
    };

    let tileset = if let Ok(x) = &parsed_chk.era {
        Some(*x.tileset as i64)
    } else {
        None
    };

    let player_owners = if let Ok(x) = &parsed_chk.ownr {
        Some(x.player_owner.clone())
    } else {
        None
    };

    let player_side = if let Ok(x) = &parsed_chk.side {
        Some(x.player_side.clone())
    } else {
        None
    };

    #[derive(Debug, Serialize)]
    struct Force {
        name: String,
        prop_random_start_location: bool,
        prop_allies: bool,
        prop_allied_victory: bool,
        prop_shared_vision: bool,
        player_ids: Vec<usize>,
    }

    let forces = if let Ok(x) = &parsed_chk.forc {
        let mut forces = Vec::new();

        for (force_id, (force_name, force_properties)) in
            std::iter::zip(x.force_name, x.force_properties).enumerate()
        {
            let name = if force_name == 0 {
                format!("Force {}", force_id + 1) // TODO: the "Force 1" strings are localized so needs to be done by FE...
            } else {
                parsed_chk
                    .get_string(force_name as usize)
                    .unwrap_or(format!(
                        "<<couldn't get force name: str_num: {force_name}>>"
                    ))
            };

            let prop_random_start_location = (force_properties & (1 << 0)) > 0;
            let prop_allies = (force_properties & (1 << 1)) > 0;
            let prop_allied_victory = (force_properties & (1 << 2)) > 0;
            let prop_shared_vision = (force_properties & (1 << 3)) > 0;

            let mut player_ids = Vec::new();
            for (player_id, player_force) in x.player_forces.iter().enumerate() {
                if *player_force as usize == force_id {
                    player_ids.push(player_id);
                }
            }

            forces.push(Force {
                name,
                prop_random_start_location,
                prop_allies,
                prop_allied_victory,
                prop_shared_vision,
                player_ids,
            });
        }

        forces
    } else {
        Vec::new()
    };

    let doodads = (if let Ok(x) = &parsed_chk.dd2 {
        Some(x.doodads.len() as i64)
    } else {
        None
    })
    .unwrap_or_default();

    let sprites = (if let Ok(x) = &parsed_chk.thg2 {
        Some(x.sprites.len() as i64)
    } else {
        None
    })
    .unwrap_or_default();

    let triggers = (if let Ok(x) = &parsed_chk.trig {
        Some(x.triggers.len() as i64)
    } else {
        None
    })
    .unwrap_or_default();

    let briefing_triggers = (if let Ok(x) = &parsed_chk.mbrf {
        Some(x.triggers.len() as i64)
    } else {
        None
    })
    .unwrap_or_default();

    let locations = (if let Ok(x) = &parsed_chk.mrgn {
        Some(
            x.locations
                .iter()
                .filter(|&&x| !(x.left == x.right || x.top == x.bottom))
                .count() as i64,
        )
    } else {
        None
    })
    .unwrap_or_default();

    let units = (if let Ok(x) = &parsed_chk.unit {
        Some(x.units.len() as i64)
    } else {
        None
    })
    .unwrap_or_default();

    let unique_terrain_tiles = (if let Ok(x) = &parsed_chk.mtxm {
        let hash_set: std::collections::HashSet<u16> = x.data.iter().cloned().collect();
        Some(hash_set.len())
    } else {
        None
    })
    .unwrap_or_default();

    let eups = (if let Ok(x) = &parsed_chk.unit {
        let mut eups: i64 = 0;
        for unit in &x.units {
            if unit.unit_id > 227 || unit.owner > 27 {
                eups += 1;
            }
        }

        Some(eups)
    } else {
        None
    })
    .unwrap_or_default();

    let parsed_triggers = bwmap::parse_triggers(&parsed_chk);

    let mission_briefings = bwmap::parse_mission_briefing(&parsed_chk);

    let mut trigger_list_reads = 0;
    let mut trigger_list_writes = 0;
    let mut get_death_euds = 0;
    let mut set_death_euds = 0;

    let wavs = {
        let mut set = std::collections::HashSet::new();

        for trigger in &parsed_triggers {
            for condition in &trigger.conditions {
                match condition {
                    bwmap::Condition::Deaths {
                        player: _,
                        comparison: _,
                        unit_type: _,
                        number: _,
                        eud_offset,
                    } => {
                        if *eud_offset < 0x58A364 || *eud_offset >= 0x58CE24 {
                            get_death_euds += 1;
                        }

                        if *eud_offset >= 0x51A280 && *eud_offset < 0x51A2E0 {
                            trigger_list_reads += 1;
                        }
                    }
                    _ => {}
                }
            }
            for action in &trigger.actions {
                match action {
                    bwmap::Action::SetDeaths {
                        player: _,
                        unit_type: _,
                        number: _,
                        modifier: _,
                        eud_offset,
                    } => {
                        if *eud_offset < 0x58A364 || *eud_offset >= 0x58CE24 {
                            set_death_euds += 1;
                        }

                        if *eud_offset >= 0x51A280 && *eud_offset < 0x51A2E0 {
                            trigger_list_writes += 1;
                        }
                    }
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

        Vec::from_iter(set.drain().cloned())
    };

    let (scenario_name, scenario_description) = if let Ok(x) = &parsed_chk.sprp {
        let scenario_string = if *x.scenario_name_string_number == 0 {
            None
        } else {
            parsed_chk
                .get_string(*x.scenario_name_string_number as usize)
                .ok()
        };

        let scenario_description_string = if *x.description_string_number == 0 {
            None
        } else {
            parsed_chk
                .get_string(*x.description_string_number as usize)
                .ok()
        };

        (scenario_string, scenario_description_string)
    } else {
        (None, None)
    };

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

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(insert_extension(HttpResponse::Ok(), info)
        .content_type("application/json")
        .body(
            serde_json::to_string(&json!({
                "scenario": scenario_name,
                "scenario_description": scenario_description,
                "player_owners": player_owners,
                "player_side": player_side,
                "forces": forces,
                "internal_id": map_id,
                "properties": {
                    "ver": ver,
                    "width": width,
                    "height": height,
                    "tileset": tileset,
                    "sprites": sprites,
                    "doodads": doodads,
                    "triggers": triggers,
                    "briefing_triggers": briefing_triggers,
                    "locations": locations,
                    "units": units,
                    "unique_terrain_tiles": unique_terrain_tiles,
                    "eups": eups,
                    "get_death_euds": get_death_euds,
                    "set_death_euds": set_death_euds,
                    "trigger_list_reads": trigger_list_reads,
                    "trigger_list_writes": trigger_list_writes,
                },
                "meta": {
                    "chkhash": chkhash,
                    "chk_size": chk_size,
                    "mpq_hash": mpq_hash,
                    "mpq_size": mpq_size,
                    "uploaded_by": uploaded_by_username,
                    "uploaded_time": uploaded_time,
                    "last_viewed": last_viewed,
                    "last_downloaded": last_downloaded,
                    "views": views,
                    "downloads": downloads,
                },
                "wavs": wavs,
            }))
            .unwrap(),
        )
        .customize())
}

// #[post("/api/uiv2/map_info/{map_id}")]
