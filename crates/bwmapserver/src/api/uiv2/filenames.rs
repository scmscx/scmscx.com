use axum::extract::{Extension, Path};
use axum::response::Response;
use axum::Json;
use bwcommon::with_logging_info;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;

use crate::webutil::Pool;

pub async fn filenames(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let pool = pool.clone();
    let con = pool.get().await?;
    let filenames: Vec<String> = con
        .query(
            "select filename.filename
            from mapfilename
            join filename on mapfilename.filename = filename.id
            where mapfilename.map = $1",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| anyhow::Ok(row.try_get(0)?))
        .collect::<Result<Vec<_>, _>>()?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(with_logging_info(info, Json(filenames)))
}
