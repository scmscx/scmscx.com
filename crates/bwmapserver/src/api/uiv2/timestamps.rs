use axum::extract::{Extension, Path};
use axum::response::Response;
use axum::Json;
use bwcommon::with_logging_info;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;

use crate::webutil::Pool;

pub async fn timestamps(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let pool = pool.clone();
    let con = pool.get().await?;

    let filetimes: Vec<i64> = con
        .query(
            "select distinct modified_time
            from map
            join filetime on filetime.map = map.id
            where map.id = $1
            order by modified_time",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| anyhow::Ok(row.try_get("modified_time")?))
        .collect::<Result<Vec<_>, _>>()?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(with_logging_info(info, Json(filetimes)))
}
