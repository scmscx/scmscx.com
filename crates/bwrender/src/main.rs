mod config;
mod db;
mod render;

use anyhow::Result;
use tracing::{error, info, warn};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter, Layer};

use crate::config::Config;
use crate::db::{DbPool, UnrenderedMap};
use crate::render::RenderContext;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (same pattern as bwmapserver)
    LogTracer::init().expect("Failed to set logger");
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_file(true)
                .with_target(false)
                .with_line_number(true)
                .with_filter(EnvFilter::from_default_env()),
        ),
    )?;

    info!("Starting bwrender service");

    // Log relevant environment variables for debugging
    info!(
        display = %std::env::var("DISPLAY").unwrap_or_else(|_| "NOT SET".to_string()),
        libgl_always_software = %std::env::var("LIBGL_ALWAYS_SOFTWARE").unwrap_or_else(|_| "NOT SET".to_string()),
        xdg_runtime_dir = %std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "NOT SET".to_string()),
        "Environment variables"
    );

    // Load configuration
    let config = Config::from_env()?;
    info!(
        sc_data_path = %config.sc_data_path,
        render_skin = ?config.render_skin,
        batch_size = config.render_batch_size,
        poll_interval_secs = config.render_poll_interval_secs,
        webp_quality = config.render_webp_quality,
        "Configuration loaded"
    );

    // Setup database pool
    let pool = db::setup_pool(&config).await?;
    info!("Database pool initialized");

    // Setup HTTP client for GSFS
    let http_client = reqwest::Client::new();

    // Initialize rendering context (loads StarCraft data)
    info!(sc_data_path = %config.sc_data_path, "Loading StarCraft data...");
    let render_ctx = RenderContext::new(&config.sc_data_path, config.render_skin)?;
    info!("StarCraft data loaded successfully");

    // Create temp directory
    tokio::fs::create_dir_all(&config.temp_dir).await?;
    info!(temp_dir = %config.temp_dir, "Temp directory ready");

    // Main loop
    info!("Starting main render loop");
    loop {
        match process_batch(&config, &pool, &http_client, &render_ctx).await {
            Ok(processed) => {
                if processed == 0 {
                    info!(
                        poll_interval_secs = config.render_poll_interval_secs,
                        "No maps to render, sleeping"
                    );
                } else {
                    info!(processed = processed, "Batch completed");
                }
            }
            Err(e) => {
                error!(error = %e, "Error processing batch");
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(
            config.render_poll_interval_secs,
        ))
        .await;
    }
}

async fn process_batch(
    config: &Config,
    pool: &DbPool,
    client: &reqwest::Client,
    render_ctx: &RenderContext,
) -> Result<usize> {
    let maps = db::get_unrendered_maps(pool, config.render_batch_size).await?;

    if maps.is_empty() {
        return Ok(0);
    }

    info!(count = maps.len(), "Found unrendered maps");

    let mut processed = 0;

    for map in maps {
        match process_single_map(config, pool, client, render_ctx, &map).await {
            Ok(()) => {
                processed += 1;
                info!(chkblob_hash = %map.chkblob_hash, "Successfully rendered map");
            }
            Err(e) => {
                // Just log and continue - random ordering means we won't get stuck
                error!(
                    chkblob_hash = %map.chkblob_hash,
                    error = %e,
                    "Failed to render map"
                );
            }
        }
    }

    Ok(processed)
}

async fn process_single_map(
    config: &Config,
    pool: &DbPool,
    client: &reqwest::Client,
    render_ctx: &RenderContext,
    map: &UnrenderedMap,
) -> Result<()> {
    let start = std::time::Instant::now();

    // 1. Download map file from GSFS to temp file
    let temp_path = format!(
        "{}/{}.scx",
        config.temp_dir,
        uuid::Uuid::new_v4().as_simple()
    );

    info!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        "Downloading map from GSFS"
    );

    common::gsfs::gsfs_download_to_file(
        client,
        &config.gsfsfe_endpoint,
        &format!("/mapblob/{}", &map.mapblob_hash),
        &temp_path,
    )
    .await?;

    // 2. Render map to WebP
    info!(chkblob_hash = %map.chkblob_hash, "Rendering map");

    let render_result = render_ctx
        .render_map(
            &temp_path,
            config.render_anim_ticks,
            config.render_webp_quality,
        )
        .await;

    // Clean up temp file regardless of result
    if let Err(e) = tokio::fs::remove_file(&temp_path).await {
        warn!(path = %temp_path, error = %e, "Failed to remove temp file");
    }

    let webp_data = render_result?;

    let file_size = webp_data.len() as i64;
    info!(
        chkblob_hash = %map.chkblob_hash,
        file_size = file_size,
        "Render complete, uploading to GSFS"
    );

    // 3. Upload to GSFS
    let gsfs_path = format!("/img/{}.webp", map.chkblob_hash);

    common::gsfs::gsfs_put_bytes(client, &config.gsfsfe_endpoint, &gsfs_path, webp_data).await?;

    // 4. Mark as rendered
    db::mark_rendered(pool, &map.chkblob_hash).await?;

    let render_time_ms = start.elapsed().as_millis();
    info!(
        chkblob_hash = %map.chkblob_hash,
        render_time_ms = render_time_ms,
        file_size = file_size,
        gsfs_path = %gsfs_path,
        "Map render completed successfully"
    );

    Ok(())
}
