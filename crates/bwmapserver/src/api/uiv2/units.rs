use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;
use bwmap::ParsedChk;
use serde_json::json;

use crate::access;
use crate::webutil::{MaybeUser, Pool, PoolExt};

/// Units carrying a custom name: entries left at default settings (`config == 0`)
/// whose `string_number` points at a string.
///
/// The modern UNIx section and the legacy UNIS one declare identical
/// `config`/`string_number` arrays, so they share this instead of each keeping a
/// copy of the filter. The UNIS copy had drifted — it ignored
/// `spoiler_unit_names` and served real names for spoiler-flagged maps — which no
/// test could see, because every fixture uses UNIx.
fn named_units(
    config: &[u8],
    string_number: &[u16],
    parsed_chk: &ParsedChk,
    spoiler_unit_names: bool,
) -> Vec<serde_json::Value> {
    let mut v = Vec::new();

    for unit_id in 0..config.len() {
        if config[unit_id] == 0 && string_number[unit_id] != 0 {
            v.push(json!({
                "unit_id": unit_id,
                "name": if spoiler_unit_names {
                    "SPOILER".to_owned()
                } else {
                    parsed_chk
                        .get_string(string_number[unit_id] as usize)
                        .unwrap_or_else(|_| "couldn't decode string".to_owned())
                },
            }));
        }
    }

    v
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
        named_units(&x.config, &x.string_number, &parsed_chk, spoiler_unit_names)
    } else if let Ok(x) = &parsed_chk.unis {
        named_units(&x.config, &x.string_number, &parsed_chk, spoiler_unit_names)
    } else {
        Vec::new()
    };

    Ok(Json(units).into_response())
}
