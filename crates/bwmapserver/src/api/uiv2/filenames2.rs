use axum::extract::{Extension, Path};
use axum::response::Response;
use axum::Json;
use bwcommon::with_logging_info;
use bwcommon::ApiSpecificInfoForLogging;
use bwcommon::MyError;

use crate::webutil::Pool;

pub async fn filenames2(
    Path((map_id,)): Path<(String,)>,
    Extension(pool): Extension<Pool>,
) -> Result<Response, MyError> {
    let map_id = crate::util::parse_map_id(&map_id)?;

    let pool = pool.clone();
    let con = pool.get().await?;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Filenames2 {
        filename: String,
        modified_time: i64,
    }

    let filetimes = con
        .query(
            "select filename.filename, extract(epoch from modified_time)::int8 as modified_time
            from filenames2
            join filename on filenames2.filename_id = filename.id
            where filenames2.map_id = $1
            order by modified_time asc",
            &[&map_id],
        )
        .await?
        .into_iter()
        .map(|row| {
            anyhow::Ok(Filenames2 {
                filename: row.try_get("filename")?,
                modified_time: row.try_get("modified_time")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let info = ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(with_logging_info(info, Json(filetimes)))
}
