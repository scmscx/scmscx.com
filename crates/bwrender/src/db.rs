use anyhow::{Context, Result};
use bb8_postgres::tokio_postgres::NoTls;
use bb8_postgres::{bb8::Pool, PostgresConnectionManager};

use crate::config::Config;

pub type DbPool = Pool<PostgresConnectionManager<NoTls>>;

/// Map data needed for rendering
#[derive(Debug, Clone)]
pub struct UnrenderedMap {
    pub chkblob_hash: String,
    pub mapblob_hash: String,
}

/// Setup the database connection pool
pub async fn setup_pool(config: &Config) -> Result<DbPool> {
    let manager = PostgresConnectionManager::new_from_stringlike(config.connection_string(), NoTls)
        .context("Failed to create connection manager")?;

    let pool = Pool::builder()
        .max_size(config.db_connections)
        .min_idle(Some(1))
        .max_lifetime(Some(std::time::Duration::from_secs(60)))
        .idle_timeout(Some(std::time::Duration::from_secs(30)))
        .test_on_check_out(true)
        .build(manager)
        .await
        .context("Failed to create connection pool")?;

    Ok(pool)
}

/// Get maps that haven't been rendered yet (in random order)
pub async fn get_unrendered_maps(pool: &DbPool, batch_size: i64) -> Result<Vec<UnrenderedMap>> {
    let conn = pool.get().await.context("Failed to get connection")?;

    let rows = conn
        .query(
            r#"
            SELECT chkblob, mapblob2 FROM (
                SELECT DISTINCT m.chkblob, m.mapblob2
                FROM map m
                WHERE m.chkblob IS NOT NULL
                  AND m.mapblob2 IS NOT NULL
                  AND m.blackholed = false
                  AND m.rendered = false
            ) sub
            ORDER BY RANDOM()
            LIMIT $1
            "#,
            &[&batch_size],
        )
        .await
        .context("Failed to query unrendered maps")?;

    let maps = rows
        .into_iter()
        .map(|row| UnrenderedMap {
            chkblob_hash: row.get("chkblob"),
            mapblob_hash: row.get("mapblob2"),
        })
        .collect();

    Ok(maps)
}

/// Mark a map as rendered
pub async fn mark_rendered(pool: &DbPool, chkblob_hash: &str) -> Result<()> {
    let conn = pool.get().await.context("Failed to get connection")?;

    conn.execute(
        "UPDATE map SET rendered = true WHERE chkblob = $1",
        &[&chkblob_hash],
    )
    .await
    .context("Failed to mark map as rendered")?;

    Ok(())
}
