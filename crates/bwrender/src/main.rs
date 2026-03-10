mod config;
mod db;
mod render;

use anyhow::Result;
use backblaze::api::{b2_authorize_account, b2_download_file_by_name, B2AuthorizeAccount, B2Error};
use futures_util::StreamExt;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter, Layer};

use crate::config::Config;
use crate::db::{DbPool, UnrenderedMap};
use crate::render::RenderContext;

struct B2Auth {
    auth: Option<B2AuthorizeAccount>,
}

impl B2Auth {
    fn new() -> Self {
        Self { auth: None }
    }

    fn invalidate(&mut self) {
        self.auth = None;
    }
}

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

    // Cached Backblaze B2 auth (fallback for GSFS failures)
    let b2_auth = Mutex::new(B2Auth::new());

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
        match process_batch(&config, &pool, &http_client, &render_ctx, &b2_auth).await {
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
    b2_auth: &Mutex<B2Auth>,
) -> Result<usize> {
    let maps = db::get_unrendered_maps(pool, config.render_batch_size).await?;

    if maps.is_empty() {
        return Ok(0);
    }

    info!(count = maps.len(), "Found unrendered maps");

    let mut processed = 0;

    for map in maps {
        match process_single_map(config, pool, client, render_ctx, b2_auth, &map).await {
            Ok(()) => {
                processed += 1;
            }
            Err(e) => {
                error!(
                    chkblob_hash = %map.chkblob_hash,
                    mapblob_hash = %map.mapblob_hash,
                    error = %e,
                    "Failed to render map"
                );
            }
        }
    }

    Ok(processed)
}

const MAPBLOB_BUCKET_NAME: &str = "seventyseven-mapblob";

async fn get_or_refresh_b2_auth(
    config: &Config,
    client: &reqwest::Client,
    b2_auth: &Mutex<B2Auth>,
) -> Result<B2AuthorizeAccount> {
    let mut lock = b2_auth.lock().await;
    if let Some(ref auth) = lock.auth {
        return Ok(auth.clone());
    }
    info!("Authenticating with Backblaze B2");
    let auth = b2_authorize_account(
        client,
        &config.backblaze_key_id,
        &config.backblaze_application_key,
    )
    .await?;
    lock.auth = Some(auth.clone());
    Ok(auth)
}

async fn download_from_b2_and_upload_to_gsfs(
    config: &Config,
    client: &reqwest::Client,
    b2_auth: &Mutex<B2Auth>,
    map: &UnrenderedMap,
    dest_path: &str,
) -> Result<()> {
    let api_info = get_or_refresh_b2_auth(config, client, b2_auth).await?;

    info!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        bucket = MAPBLOB_BUCKET_NAME,
        "Downloading mapblob from Backblaze B2"
    );

    let download_result =
        b2_download_file_by_name(client, &api_info, MAPBLOB_BUCKET_NAME, &map.mapblob_hash).await;

    let mut stream = match download_result {
        Ok(stream) => stream,
        Err(B2Error::BadAuthToken(_) | B2Error::ExpiredAuthToken(_)) => {
            warn!(
                chkblob_hash = %map.chkblob_hash,
                mapblob_hash = %map.mapblob_hash,
                "B2 auth token expired, re-authenticating"
            );
            b2_auth.lock().await.invalidate();
            let api_info = get_or_refresh_b2_auth(config, client, b2_auth).await?;
            b2_download_file_by_name(client, &api_info, MAPBLOB_BUCKET_NAME, &map.mapblob_hash)
                .await?
        }
        Err(e) => return Err(e.into()),
    };

    // Download to temp file
    let mut file = tokio::fs::File::create(dest_path).await?;
    let mut all_bytes = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        all_bytes.extend_from_slice(&chunk);
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }

    tokio::io::AsyncWriteExt::flush(&mut file).await?;
    drop(file);

    info!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        size = all_bytes.len(),
        "Downloaded from B2, uploading to GSFS"
    );

    info!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        "Uploading mapblob to GSFS for future cache"
    );

    // Upload to GSFS so future fetches don't need B2
    if let Err(e) = common::gsfs::gsfs_put_bytes(
        client,
        &config.gsfsfe_endpoint,
        &format!("/mapblob/{}", &map.mapblob_hash),
        all_bytes,
    )
    .await
    {
        warn!(
            mapblob_hash = %map.mapblob_hash,
            error = %e,
            "Failed to upload to GSFS after B2 download, continuing with render"
        );
    }

    Ok(())
}

async fn process_single_map(
    config: &Config,
    pool: &DbPool,
    client: &reqwest::Client,
    render_ctx: &RenderContext,
    b2_auth: &Mutex<B2Auth>,
    map: &UnrenderedMap,
) -> Result<()> {
    let start = std::time::Instant::now();

    // 1. Download map file from GSFS to temp file, falling back to Backblaze B2
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

    let gsfs_result = common::gsfs::gsfs_download_to_file(
        client,
        &config.gsfsfe_endpoint,
        &format!("/mapblob/{}", &map.mapblob_hash),
        &temp_path,
    )
    .await;

    if let Err(e) = gsfs_result {
        warn!(
            chkblob_hash = %map.chkblob_hash,
            mapblob_hash = %map.mapblob_hash,
            error = %e,
            "GSFS download failed, falling back to Backblaze B2"
        );

        download_from_b2_and_upload_to_gsfs(config, client, b2_auth, map, &temp_path).await?;
    } else {
        info!(
            chkblob_hash = %map.chkblob_hash,
            mapblob_hash = %map.mapblob_hash,
            "Downloaded mapblob from GSFS"
        );
    }

    // 2. Render map to WebP
    info!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        "Rendering map to WebP"
    );

    let render_result = render_ctx
        .render_map(
            &temp_path,
            config.render_anim_ticks,
            config.render_webp_quality,
        )
        .await;

    // Clean up temp file regardless of result
    if let Err(e) = tokio::fs::remove_file(&temp_path).await {
        warn!(
            chkblob_hash = %map.chkblob_hash,
            mapblob_hash = %map.mapblob_hash,
            path = %temp_path,
            error = %e,
            "Failed to remove temp file"
        );
    }

    let webp_data = render_result?;

    let file_size = webp_data.len() as i64;

    // 3. Upload rendered image to GSFS
    let gsfs_path = format!("/img/{}.webp", map.chkblob_hash);
    info!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        file_size = file_size,
        gsfs_path = %gsfs_path,
        "Uploading rendered image to GSFS"
    );

    common::gsfs::gsfs_put_bytes(client, &config.gsfsfe_endpoint, &gsfs_path, webp_data).await?;

    // 4. Mark as rendered in database
    info!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        "Marking map as rendered in database"
    );
    db::mark_rendered(pool, &map.chkblob_hash).await?;

    let render_time_ms = start.elapsed().as_millis();
    info!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        render_time_ms = render_time_ms,
        file_size = file_size,
        gsfs_path = %gsfs_path,
        "Map render pipeline completed"
    );

    Ok(())
}
