use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;
use bwmap::ParsedChk;
use serde::Serialize;

use crate::access;
use crate::webutil::{MaybeUser, Pool, PoolExt};

#[derive(Debug, Serialize)]
struct UnitSettings {
    unit_id: usize,
    /// `None` when the unit keeps its default name (`string_number == 0`).
    name: Option<String>,
    /// In whole hit points. The section stores 1/256ths; the low byte is a
    /// fractional HP the game never displays, but it is kept rather than rounded.
    hit_points: f64,
    shield_points: u16,
    armor_points: u8,
    build_time: u16,
    mineral_cost: u16,
    gas_cost: u16,
}

/// Units whose "use default settings" flag is cleared (`config == 0`), with the
/// name and stats the map overrides them to. The section stores every unit's
/// settings either way, but the game only reads them for these units.
///
/// The modern UNIx section and the legacy UNIS one declare identical per-unit
/// arrays (they differ only in their weapon arrays), so they share this instead
/// of each keeping a copy of the filter.
#[allow(clippy::too_many_arguments)]
fn overridden_units(
    config: &[u8],
    hit_points: &[u32],
    shield_points: &[u16],
    armor_points: &[u8],
    build_time: &[u16],
    mineral_cost: &[u16],
    gas_cost: &[u16],
    string_number: &[u16],
    parsed_chk: &ParsedChk,
    spoiler_unit_names: bool,
) -> Vec<UnitSettings> {
    (0..config.len())
        .filter(|&unit_id| config[unit_id] == 0)
        .map(|unit_id| UnitSettings {
            unit_id,
            name: match string_number[unit_id] {
                0 => None,
                _ if spoiler_unit_names => Some("SPOILER".to_owned()),
                string_number => Some(
                    parsed_chk
                        .get_string(string_number as usize)
                        .unwrap_or_else(|_| "couldn't decode string".to_owned()),
                ),
            },
            hit_points: hit_points[unit_id] as f64 / 256.0,
            shield_points: shield_points[unit_id],
            armor_points: armor_points[unit_id],
            build_time: build_time[unit_id],
            mineral_cost: mineral_cost[unit_id],
            gas_cost: gas_cost[unit_id],
        })
        .collect()
}

pub async fn units(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
    user: MaybeUser,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    // `blackholed` rides along on the query this handler already runs against the
    // map row, rather than costing a second checkout via `access::map_is_hidden`.
    let (chkblob, spoiler_unit_names) = {
        let con = pool.acquire().await?;
        let Some(row) = con
            .query_opt(
                "select length, ver, data, spoiler_unit_names, blackholed
                from map
                -- LEFT, not inner: an unprocessed map has no chkblob yet and
                -- should report no units, not 404 as though it did not exist.
                left join chkblob on chkblob.hash = map.chkblob
                where map.id = $1
                ",
                &[&map_id],
            )
            .await?
        else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

        if access::blackholed_is_hidden_from(row.try_get("blackholed")?, user.session()) {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }

        // NULL together until the map has been processed; an empty chk parses
        // to no unit section, so the handler answers with an empty list.
        let chk = match (
            row.try_get::<_, Option<i64>>("length")?,
            row.try_get::<_, Option<i64>>("ver")?,
            row.try_get::<_, Option<Vec<u8>>>("data")?,
        ) {
            (Some(length), Some(ver), Some(data)) => {
                bwcommon::ensure!(ver == 1);
                zstd::bulk::decompress(data.as_slice(), length as usize)?
            }
            _ => Vec::new(),
        };
        (chk, row.try_get::<_, bool>("spoiler_unit_names")?)
    };

    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let units = if let Ok(x) = &parsed_chk.unix {
        overridden_units(
            &x.config,
            &x.hit_points,
            &x.shield_points,
            &x.armor_points,
            &x.build_time,
            &x.mineral_cost,
            &x.gas_cost,
            &x.string_number,
            &parsed_chk,
            spoiler_unit_names,
        )
    } else if let Ok(x) = &parsed_chk.unis {
        overridden_units(
            &x.config,
            &x.hit_points,
            &x.shield_points,
            &x.armor_points,
            &x.build_time,
            &x.mineral_cost,
            &x.gas_cost,
            &x.string_number,
            &parsed_chk,
            spoiler_unit_names,
        )
    } else {
        Vec::new()
    };

    Ok(Json(units).into_response())
}
