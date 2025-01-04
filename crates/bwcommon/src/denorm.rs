use anyhow::anyhow;
use anyhow::Result;
use backblaze::api::b2_authorize_account;
use backblaze::api::b2_download_file_by_name;
use bb8_postgres::tokio_postgres::Transaction;
use bwmap::ParsedChk;
use reqwest::Client;
use tokio::io::AsyncWriteExt;

pub fn calculate_perceptual_hashes(minimap: &Vec<u8>) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    use image::ImageDecoder;

    let cursor = std::io::Cursor::new(minimap.as_slice());
    let png = image::codecs::png::PngDecoder::new(cursor)?;
    let (x, y) = png.dimensions();

    let mut image_data = vec![0; png.total_bytes() as usize];

    anyhow::ensure!(ImageDecoder::color_type(&png) == image::ColorType::Rgb8);

    png.read_image(image_data.as_mut_slice())?;

    let image: image::ImageBuffer<image::Rgb<u8>, _> =
        image::ImageBuffer::from_vec(x, y, image_data).unwrap();

    let ph8x8 = image::imageops::grayscale(&image::imageops::resize(
        &image,
        8,
        8,
        image::imageops::Lanczos3,
    ));
    let ph16x16 = image::imageops::grayscale(&image::imageops::resize(
        &image,
        16,
        16,
        image::imageops::Lanczos3,
    ));
    let ph32x32 = image::imageops::grayscale(&image::imageops::resize(
        &image,
        32,
        32,
        image::imageops::Lanczos3,
    ));

    let ph8x8_avg = (ph8x8.iter().fold(0, |acc, x| acc + (*x as usize)) / (8 * 8)) as u8;
    let ph16x16_avg = (ph16x16.iter().fold(0, |acc, x| acc + (*x as usize)) / (16 * 16)) as u8;
    let ph32x32_avg = (ph32x32.iter().fold(0, |acc, x| acc + (*x as usize)) / (32 * 32)) as u8;

    let ph8x8: Vec<_> = ph8x8
        .iter()
        .map(|x| if *x < ph8x8_avg { 0 } else { 1 })
        .collect::<Vec<u8>>()
        .chunks_exact(8)
        .map(|x| x.iter().fold(0u8, |acc, x| acc << 1 | *x))
        .collect();
    anyhow::ensure!(ph8x8.len() == 8 * 8 / 8);

    let ph16x16: Vec<_> = ph16x16
        .iter()
        .map(|x| if *x < ph16x16_avg { 0 } else { 1 })
        .collect::<Vec<u8>>()
        .chunks_exact(8)
        .map(|x| x.iter().fold(0u8, |acc, x| acc << 1 | *x))
        .collect();
    anyhow::ensure!(ph16x16.len() == 16 * 16 / 8);

    let ph32x32: Vec<_> = ph32x32
        .iter()
        .map(|x| if *x < ph32x32_avg { 0 } else { 1 })
        .collect::<Vec<u8>>()
        .chunks_exact(8)
        .map(|x| x.iter().fold(0u8, |acc, x| acc << 1 | *x))
        .collect();
    anyhow::ensure!(ph32x32.len() == 32 * 32 / 8);

    anyhow::Ok((ph8x8, ph16x16, ph32x32))
}

async fn update_strings(
    tx: &mut Transaction<'_>,
    map_id: i64,
    parsed_chk: &ParsedChk<'_>,
) -> Result<()> {
    pub(crate) fn sanitize_sc_string(s: &str) -> String {
        // split string by left or right marks

        let mut strings: Vec<_> = s.split(|x| x == '\u{0012}' || x == '\u{0013}').collect();

        strings.sort_by_key(|x| std::cmp::Reverse(x.len()));

        if strings.len() == 0 {
            String::new()
        } else {
            strings[0].chars().filter(|&x| x >= ' ').collect()
        }
    }

    pub(crate) fn sanitize_sc_string_preserve_newlines(s: &str) -> String {
        s.split('\n')
            .map(sanitize_sc_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

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
            if string_number != 0 {
                if let Ok(string) = parsed_chk.get_string(string_number as usize) {
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
    let filenames: Vec<String> = tx
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

    tx.execute("delete from stringmap2 where map = $1", &[&map_id])
        .await?;

    // scenario name
    if let Some(scenario_name) = scenario_name {
        tx.execute(
            "call insert_string2($1, $2, $3)",
            &[&map_id, &"scenario_name", &vec![scenario_name.as_str()]],
        )
        .await?;
    }

    // scenario description
    if let Some(scenario_description) = scenario_description {
        tx.execute(
            "call insert_string2($1, $2, $3)",
            &[
                &map_id,
                &"scenario_description",
                &vec![scenario_description.as_str()],
            ],
        )
        .await?;
    };

    // unit names
    if let Some(unit_names) = unit_names {
        tx.execute(
            "call insert_string2($1, $2, $3)",
            &[&map_id, &"unit_names", &unit_names],
        )
        .await?;
    }

    // force names
    if let Some(force_names) = force_names {
        tx.execute(
            "call insert_string2($1, $2, $3)",
            &[&map_id, &"force_names", &force_names],
        )
        .await?;
    }

    // file names
    {
        tx.execute(
            "call insert_string2($1, $2, $3)",
            &[&map_id, &"file_names", &filenames],
        )
        .await?;
    }

    anyhow::Ok(())
}

async fn update_minimap(
    tx: &mut Transaction<'_>,
    chk_hash: &str,
    parsed_chk: &ParsedChk<'_>,
) -> Result<()> {
    let mtxm_section = if let Ok(x) = &parsed_chk.mtxm {
        anyhow::Ok(x)
    } else {
        Err(anyhow!("Could not get MTXM section"))
    }?;

    let dim_section = if let Ok(x) = &parsed_chk.dim {
        anyhow::Ok(x)
    } else {
        Err(anyhow!("Could not get DIM section"))
    }?;

    let era_section = if let Ok(x) = &parsed_chk.era {
        anyhow::Ok(x)
    } else {
        Err(anyhow!("Could not get ERA section"))
    }?;

    let minimap = bwminimaprender::render_minimap(
        mtxm_section.data.as_slice(),
        *dim_section.width as usize,
        *dim_section.height as usize,
        *era_section.tileset,
    )?;

    let (ph8x8, ph16x16, ph32x32) = calculate_perceptual_hashes(&minimap)?;

    tx.execute("delete from minimap where chkhash = $1", &[&chk_hash])
        .await?;

    let binstring = ph16x16
        .iter()
        .map(|x| format!("{x:08b}"))
        .collect::<String>();

    tracing::info!("binstring: {}", binstring);

    tx.execute(
        "INSERT INTO minimap
            (chkhash, width, height, minimap, ph8x8, ph16x16, ph32x32, vector) VALUES
            ($1, $2, $3, $4, $5, $6, $7, ($8::text)::bit(256)) ON CONFLICT DO NOTHING",
        &[
            &chk_hash,
            &(*dim_section.width as i32),
            &(*dim_section.height as i32),
            &minimap,
            &ph8x8,
            &ph16x16,
            &ph32x32,
            &binstring,
        ],
    )
    .await?;

    anyhow::Ok(())
}

async fn update_chkdenorm(
    transaction: &mut Transaction<'_>,
    chk_hash: &str,
    parsed_chk: &ParsedChk<'_>,
) -> Result<()> {
    let (width, height) = if let Ok(x) = &parsed_chk.dim {
        (Some(*x.width as i64), Some(*x.height as i64))
    } else {
        (None, None)
    };

    let tileset = if let Ok(x) = &parsed_chk.era {
        Some((x.tileset % 8) as i64)
    } else {
        None
    };

    let (human_players, computer_players) = if let Ok(x) = &parsed_chk.ownr {
        (
            Some(x.player_owner.iter().filter(|&&x| x == 6).count() as i64),
            Some(x.player_owner.iter().filter(|&&x| x == 5).count() as i64),
        )
    } else {
        (None, None)
    };

    let doodads = if let Ok(x) = &parsed_chk.dd2 {
        Some(x.doodads.len() as i64)
    } else {
        None
    };

    let sprites = if let Ok(x) = &parsed_chk.thg2 {
        Some(x.sprites.len() as i64)
    } else {
        None
    };

    let triggers = if let Ok(x) = &parsed_chk.trig {
        Some(x.triggers.len() as i64)
    } else {
        None
    };

    let briefing_triggers = if let Ok(x) = &parsed_chk.mbrf {
        Some(x.triggers.len() as i64)
    } else {
        None
    };

    let locations = if let Ok(x) = &parsed_chk.mrgn {
        Some(
            x.locations
                .iter()
                .filter(|&&x| !(x.left == x.right || x.top == x.bottom))
                .count() as i64,
        )
    } else {
        None
    };

    let units = if let Ok(x) = &parsed_chk.unit {
        Some(x.units.len() as i64)
    } else {
        None
    };

    let eups = if let Ok(x) = &parsed_chk.unit {
        let mut eups: i64 = 0;
        for unit in &x.units {
            if unit.unit_id > 227 || unit.owner > 27 {
                eups += 1;
            }
        }

        Some(eups)
    } else {
        None
    };

    let (scenario_name, scenario_description) = if let Ok(x) = &parsed_chk.sprp {
        let scenario_string = if *x.scenario_name_string_number == 0 {
            None
        } else {
            if let Ok(s) = parsed_chk.get_string(*x.scenario_name_string_number as usize) {
                Some(s)
            } else {
                None
            }
        };

        let scenario_description_string = if *x.description_string_number == 0 {
            None
        } else {
            if let Ok(s) = parsed_chk.get_string(*x.description_string_number as usize) {
                Some(s)
            } else {
                None
            }
        };

        (scenario_string, scenario_description_string)
    } else {
        (None, None)
    };

    transaction
        .execute("delete from chkdenorm where chkblob = $1", &[&chk_hash])
        .await?;
    transaction
        .execute("insert into chkdenorm (width, height, tileset, human_players, computer_players, sprites, triggers, briefing_triggers, locations, units, scenario_name, get_deaths_euds_or_epds, set_deaths_euds_or_epds, eups, strings, chkblob, doodads, scenario_description
        ) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18) ON CONFLICT DO NOTHING", &[
            &width,
            &height,
            &tileset,
            &human_players,
            &computer_players,
            &sprites,
            &triggers,
            &briefing_triggers,
            &locations,
            &units,
            &scenario_name,
            &Option::<i64>::None,
            &Option::<i64>::None,
            &eups,
            &Option::<i64>::None,
            &chk_hash,
            &doodads,
            &scenario_description,
        ]).await?;

    anyhow::Ok(())
}

pub async fn denormalize_map_tx(map_id: i64, tx: &mut Transaction<'_>) -> Result<()> {
    let chk_hash: Option<String> = tx
        .query_one("select chkblob from map where map.id = $1", &[&map_id])
        .await?
        .try_get(0)?;

    let chk_hash = match chk_hash {
        Some(chk_hash) => chk_hash,
        None => {
            // hash didn't exist in db, download the map, try to extract it, and so on.
            let mapblob_hash: String = tx
                .query_one("select mapblob2 from map where map.id = $1", &[&map_id])
                .await?
                .try_get(0)?;

            const MAPBLOB_BUCKET_NAME: &'static str = "seventyseven-mapblob";
            let client = Client::new();

            let api_info = b2_authorize_account(
                &client,
                &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
                &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
            )
            .await?;

            let mut stream = b2_download_file_by_name(
                &client,
                &api_info,
                MAPBLOB_BUCKET_NAME,
                mapblob_hash.as_str(),
            )
            .await?;

            let tmp_filename = format!("/tmp/{}.scx", uuid::Uuid::new_v4().as_simple().to_string());
            let mut file = tokio::fs::File::create(&tmp_filename).await?;

            use futures_util::stream::StreamExt;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk).await?;
            }

            file.shutdown().await?;

            drop(file);

            let chk_blob = bwmpq::get_chk_from_mpq_filename(tmp_filename)?;

            let chk_blob_hash = {
                use sha2::Digest;
                let mut hasher = sha2::Sha256::new();
                hasher.update(&chk_blob);
                format!("{:x}", hasher.finalize())
            };

            let chk_blob_compressed = zstd::bulk::compress(chk_blob.as_slice(), 15)?;

            tx.execute("INSERT INTO chkblob (hash, ver, length, data) VALUES ($1, 1, $2, $3) ON CONFLICT DO NOTHING RETURNING Cast((xmax = 0) as boolean) AS inserted",
        &[&chk_blob_hash, &(chk_blob.len() as i64), &chk_blob_compressed]).await?;

            tx.execute(
                "update map set chkblob = $1 where map.id = $2",
                &[&chk_blob_hash, &map_id],
            )
            .await?;

            chk_blob_hash
        }
    };

    let chk_blob = {
        let row = tx
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

    update_chkdenorm(tx, chk_hash.as_str(), &parsed_chk).await?;
    update_minimap(tx, chk_hash.as_str(), &parsed_chk).await?;
    update_strings(tx, map_id, &parsed_chk).await?;
    // update_strings3(tx, map_id, &parsed_chunks).await?;
    //update_strings5(tx, map_id, &parsed_chunks).await?;

    anyhow::Ok(())
}
