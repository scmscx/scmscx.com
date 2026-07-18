use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use axum::Json;
use bwcommon::MyError;
use bwmap::ParsedChk;
use serde_json::json;

use crate::webutil::Pool;

pub async fn units(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let (chkblob, spoiler_unit_names) = {
        let con = pool.get().await?;
        let row = con
            .query_one(
                "select length, ver, data, spoiler_unit_names
                from map
                join chkblob on chkblob.hash = map.chkblob
                where map.id = $1
                ",
                &[&map_id],
            )
            .await?;

        let length = row.try_get::<_, i64>("length")? as usize;
        let ver = row.try_get::<_, i64>("ver")?;
        let data = row.try_get::<_, Vec<u8>>("data")?;

        bwcommon::ensure!(ver == 1);
        (
            zstd::bulk::decompress(data.as_slice(), length)?,
            row.try_get::<_, bool>("spoiler_unit_names")?,
        )
    };

    let parsed_chk = ParsedChk::from_bytes(chkblob.as_slice());

    let units = if let Ok(x) = &parsed_chk.unix {
        let mut v = Vec::new();

        for unit_id in 0..x.config.len() {
            if x.config[unit_id] == 0 && x.string_number[unit_id] != 0 {
                v.push(json!({
                    "unit_id": unit_id,
                    "name": if spoiler_unit_names { "SPOILER".to_owned()  } else {
                        parsed_chk.get_string(x.string_number[unit_id] as usize)
                        .unwrap_or_else(|_| "couldn't decode string".to_owned())
                        },
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
                        .unwrap_or_else(|_| "couldn't decode string".to_owned()),
                }));
            }
        }

        v
    } else {
        Vec::new()
    };

    Ok(Json(units).into_response())
}
