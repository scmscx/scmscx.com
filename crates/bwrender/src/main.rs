mod config;
mod db;
mod encode;
mod render;

use anyhow::Result;
use backblaze::api::{b2_authorize_account, b2_download_file_by_name, B2AuthorizeAccount, B2Error};
use futures_util::StreamExt;
use render::RenderResult;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
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

/// Timing data accumulated as a job moves through the pipeline.
struct PipelineTimings {
    download_ms: u128,
    download_source: &'static str,
    render_ms: u128,
    encode_ms: u128,
    map_size: u64,
}

/// Job passed from download workers to the render thread.
struct RenderJob {
    temp_path: String,
    map_id: i64,
    chkblob_hash: String,
    mapblob_hash: String,
    download_ms: u128,
    download_source: &'static str,
    map_size: u64,
}

/// Job passed from the render thread to encode workers.
struct EncodeJob {
    render_result: RenderResult,
    map_id: i64,
    chkblob_hash: String,
    mapblob_hash: String,
    download_ms: u128,
    download_source: &'static str,
    render_ms: u128,
    map_size: u64,
}

/// Job passed from encode workers to upload workers.
struct UploadJob {
    webp_data: Vec<u8>,
    minimap_png: Vec<u8>,
    map_id: i64,
    chkblob_hash: String,
    mapblob_hash: String,
    timings: PipelineTimings,
}

/// Snapshot of all queue lengths for logging.
struct QueueLengths {
    download: usize,
    render: usize,
    encode: usize,
    upload: usize,
}

/// Shared references to all queue receivers/senders for snapshotting queue lengths.
struct QueueRefs {
    download_rx: async_channel::Receiver<UnrenderedMap>,
    render_rx: async_channel::Receiver<RenderJob>,
    encode_rx: async_channel::Receiver<EncodeJob>,
    upload_rx: async_channel::Receiver<UploadJob>,
}

impl QueueRefs {
    fn snapshot(&self) -> QueueLengths {
        QueueLengths {
            download: self.download_rx.len(),
            render: self.render_rx.len(),
            encode: self.encode_rx.len(),
            upload: self.upload_rx.len(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
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

    info!(
        display = %std::env::var("DISPLAY").unwrap_or_else(|_| "NOT SET".to_string()),
        libgl_always_software = %std::env::var("LIBGL_ALWAYS_SOFTWARE").unwrap_or_else(|_| "NOT SET".to_string()),
        xdg_runtime_dir = %std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "NOT SET".to_string()),
        "Environment variables"
    );

    let config = Arc::new(Config::from_env()?);

    let render_channel_bound = config.max_concurrent_downloads * 2;
    let encode_channel_bound = config.max_concurrent_encodes * 2;
    let upload_channel_bound = config.max_concurrent_uploads * 2;

    info!(
        sc_data_path = %config.sc_data_path,
        render_skin = ?config.render_skin,
        poll_interval_secs = config.render_poll_interval_secs,
        webp_quality = config.render_webp_quality,
        max_concurrent_downloads = config.max_concurrent_downloads,
        max_concurrent_renders = config.max_concurrent_renders,
        max_concurrent_encodes = config.max_concurrent_encodes,
        max_concurrent_uploads = config.max_concurrent_uploads,
        render_channel_bound = render_channel_bound,
        encode_channel_bound = encode_channel_bound,
        upload_channel_bound = upload_channel_bound,
        "Configuration loaded"
    );

    // Setup database pool
    let pool = db::setup_pool(&config).await?;
    info!("Database pool initialized");

    let http_client = reqwest::Client::new();
    let b2_auth = Arc::new(Mutex::new(B2Auth::new()));

    // Create temp directory
    tokio::fs::create_dir_all(&config.temp_dir).await?;
    info!(temp_dir = %config.temp_dir, "Temp directory ready");

    // Stage 1: DB fetcher -> download_tx (unbounded)
    let (download_tx, download_rx) = async_channel::unbounded::<UnrenderedMap>();

    // Stage 2: download workers -> render_tx
    let (render_tx, render_rx) = async_channel::bounded::<RenderJob>(render_channel_bound);

    // Stage 3: render thread -> encode_tx
    let (encode_tx, encode_rx) = async_channel::bounded::<EncodeJob>(encode_channel_bound);

    // Stage 4: encode workers -> upload_tx
    let (upload_tx, upload_rx) = async_channel::bounded::<UploadJob>(upload_channel_bound);

    // Queue refs for snapshotting lengths from upload workers
    let queue_refs = Arc::new(QueueRefs {
        download_rx: download_rx.clone(),
        render_rx: render_rx.clone(),
        encode_rx: encode_rx.clone(),
        upload_rx: upload_rx.clone(),
    });

    // Spawn download workers
    info!(
        num_workers = config.max_concurrent_downloads,
        "Spawning download workers"
    );
    for worker_id in 0..config.max_concurrent_downloads {
        let rx = download_rx.clone();
        let tx = render_tx.clone();
        let config = config.clone();
        let client = http_client.clone();
        let b2_auth = b2_auth.clone();
        tokio::spawn(async move {
            download_worker(worker_id, rx, tx, &config, &client, &b2_auth).await;
        });
    }
    drop(download_rx);
    drop(render_tx);

    // Spawn render workers (each with its own OpenGL context on a dedicated thread)
    info!(
        num_workers = config.max_concurrent_renders,
        sc_data_path = %config.sc_data_path,
        "Spawning render workers and loading StarCraft data"
    );
    for worker_id in 0..config.max_concurrent_renders {
        let render_ctx = RenderContext::new(&config.sc_data_path, config.render_skin)?;
        info!(worker_id = worker_id, "Render context initialized");
        let rx = render_rx.clone();
        let tx = encode_tx.clone();
        let config = config.clone();
        tokio::spawn(async move {
            render_worker(worker_id, rx, tx, &config, &render_ctx).await;
        });
    }
    drop(render_rx);
    drop(encode_tx);

    // Spawn encode workers
    info!(
        num_workers = config.max_concurrent_encodes,
        "Spawning encode workers"
    );
    for worker_id in 0..config.max_concurrent_encodes {
        let rx = encode_rx.clone();
        let tx = upload_tx.clone();
        let config = config.clone();
        tokio::spawn(async move {
            encode_worker(worker_id, rx, tx, &config).await;
        });
    }
    drop(encode_rx);
    drop(upload_tx);

    // Spawn upload workers
    info!(
        num_workers = config.max_concurrent_uploads,
        "Spawning upload workers"
    );
    for worker_id in 0..config.max_concurrent_uploads {
        let rx = upload_rx.clone();
        let pool = pool.clone();
        let client = http_client.clone();
        let config = config.clone();
        let queue_refs = queue_refs.clone();
        tokio::spawn(async move {
            upload_worker(worker_id, rx, &config, &pool, &client, &queue_refs).await;
        });
    }
    drop(upload_rx);

    // DB fetcher loop: refill download queue when empty
    info!("Starting DB fetcher loop");
    loop {
        // Wait until the download queue is drained before refilling
        if !download_tx.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        let maps = match db::get_unrendered_maps(&pool).await {
            Ok(maps) => maps,
            Err(e) => {
                error!(error = ?e, "Failed to fetch unrendered maps from DB");
                tokio::time::sleep(std::time::Duration::from_secs(
                    config.render_poll_interval_secs,
                ))
                .await;
                continue;
            }
        };

        if maps.is_empty() {
            info!(
                poll_interval_secs = config.render_poll_interval_secs,
                "No maps to render, sleeping"
            );
            tokio::time::sleep(std::time::Duration::from_secs(
                config.render_poll_interval_secs,
            ))
            .await;
            continue;
        }

        info!(
            count = maps.len(),
            "Found unrendered maps, filling download queue"
        );

        for map in maps {
            download_tx
                .send(map)
                .await
                .map_err(|_| anyhow::anyhow!("Download channel closed"))?;
        }
    }
}

/// Download worker: fetches map files from GSFS/B2 and sends them to the render queue.
async fn download_worker(
    worker_id: usize,
    rx: async_channel::Receiver<UnrenderedMap>,
    tx: async_channel::Sender<RenderJob>,
    config: &Config,
    client: &reqwest::Client,
    b2_auth: &Mutex<B2Auth>,
) {
    info!(worker_id = worker_id, "Download worker started");

    while let Ok(map) = rx.recv().await {
        let start = std::time::Instant::now();

        let temp_path = format!(
            "{}/{}.scx",
            config.temp_dir,
            uuid::Uuid::new_v4().as_simple()
        );

        let gsfs_result = common::gsfs::gsfs_download_mapblob_to_file(
            client,
            &config.gsfsfe_endpoint,
            &map.mapblob_hash,
            &temp_path,
        )
        .await;

        let download_source = if let Err(e) = gsfs_result {
            info!(
                worker_id = worker_id,
                chkblob_hash = %map.chkblob_hash,
                mapblob_hash = %map.mapblob_hash,
                error = %e,
                "GSFS download failed, falling back to Backblaze B2"
            );

            if let Err(e) =
                download_from_b2_and_upload_to_gsfs(config, client, b2_auth, &map, &temp_path).await
            {
                error!(
                    worker_id = worker_id,
                    chkblob_hash = %map.chkblob_hash,
                    mapblob_hash = %map.mapblob_hash,
                    error = %e,
                    "Failed to download map from B2"
                );
                continue;
            }
            "b2"
        } else {
            "gsfs"
        };

        // Extract CHK and upload to GSFS
        {
            let chkblob_hash = map.chkblob_hash.clone();
            let temp_path_clone = temp_path.clone();
            match tokio::task::spawn_blocking(move || {
                bwmpq::get_chk_from_mpq_filename(&temp_path_clone)
            })
            .await
            {
                Ok(Ok(chkblob)) => {
                    if let Err(e) = common::gsfs::gsfs_put_chkblob(
                        client,
                        &config.gsfsfe_endpoint,
                        &chkblob_hash,
                        chkblob,
                    )
                    .await
                    {
                        warn!(
                            worker_id = worker_id,
                            chkblob_hash = %chkblob_hash,
                            error = %e,
                            "Failed to upload chkblob to GSFS"
                        );
                    }
                }
                Ok(Err(e)) => {
                    warn!(
                        worker_id = worker_id,
                        chkblob_hash = %map.chkblob_hash,
                        error = %e,
                        "Failed to extract CHK from map"
                    );
                }
                Err(e) => {
                    warn!(
                        worker_id = worker_id,
                        chkblob_hash = %map.chkblob_hash,
                        error = %e,
                        "CHK extraction task panicked"
                    );
                }
            }
        }

        let download_ms = start.elapsed().as_millis();
        let map_size = tokio::fs::metadata(&temp_path).await.map_or(0, |m| m.len());

        if tx
            .send(RenderJob {
                temp_path,
                map_id: map.map_id,
                chkblob_hash: map.chkblob_hash.clone(),
                mapblob_hash: map.mapblob_hash.clone(),
                download_ms,
                download_source,
                map_size,
            })
            .await
            .is_err()
        {
            error!(worker_id = worker_id, chkblob_hash = %map.chkblob_hash, "Render channel closed");
            break;
        }
    }

    info!(worker_id = worker_id, "Download worker shutting down");
}

/// Render worker: renders maps via OpenGL and sends raw images to encode queue.
/// Each worker has its own dedicated render thread with an independent OpenGL context.
async fn render_worker(
    worker_id: usize,
    rx: async_channel::Receiver<RenderJob>,
    tx: async_channel::Sender<EncodeJob>,
    config: &Config,
    render_ctx: &RenderContext,
) {
    info!(worker_id = worker_id, "Render worker started");

    while let Ok(job) = rx.recv().await {
        let start = std::time::Instant::now();

        let render_result = render_ctx
            .render_map(&job.temp_path, config.render_anim_ticks)
            .await;
        let render_ms = start.elapsed().as_millis();

        // Clean up temp file
        if let Err(e) = tokio::fs::remove_file(&job.temp_path).await {
            warn!(
                chkblob_hash = %job.chkblob_hash,
                path = %job.temp_path,
                error = %e,
                "Failed to remove temp file"
            );
        }

        let render_result = match render_result {
            Ok(result) => result,
            Err(e) => {
                error!(
                    chkblob_hash = %job.chkblob_hash,
                    mapblob_hash = %job.mapblob_hash,
                    error = %e,
                    "Failed to render map"
                );
                continue;
            }
        };

        if tx
            .send(EncodeJob {
                render_result,
                map_id: job.map_id,
                chkblob_hash: job.chkblob_hash.clone(),
                mapblob_hash: job.mapblob_hash.clone(),
                download_ms: job.download_ms,
                download_source: job.download_source,
                render_ms,
                map_size: job.map_size,
            })
            .await
            .is_err()
        {
            error!(chkblob_hash = %job.chkblob_hash, "Encode channel closed");
            break;
        }
    }

    info!(worker_id = worker_id, "Render worker shutting down");
}

/// Encode worker: encodes raw images to WebP and sends to upload queue.
async fn encode_worker(
    worker_id: usize,
    rx: async_channel::Receiver<EncodeJob>,
    tx: async_channel::Sender<UploadJob>,
    config: &Config,
) {
    info!(worker_id = worker_id, "Encode worker started");

    while let Ok(job) = rx.recv().await {
        let start = std::time::Instant::now();
        let chkblob_hash = job.chkblob_hash;
        let mapblob_hash = job.mapblob_hash;

        let webp_quality = config.render_webp_quality;
        let render_result = job.render_result;

        let encode_result = tokio::task::spawn_blocking(move || {
            let webp_data = encode::encode_rgb_to_webp(
                &render_result.map_image.rgb_data,
                render_result.map_image.width,
                render_result.map_image.height,
                webp_quality,
            )?;
            let minimap_png = encode::encode_rgb_to_png(
                &render_result.minimap_image.rgb_data,
                render_result.minimap_image.width,
                render_result.minimap_image.height,
            )?;
            anyhow::Ok((webp_data, minimap_png))
        })
        .await;

        let (webp_data, minimap_png) = match encode_result {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => {
                error!(worker_id = worker_id, chkblob_hash = %chkblob_hash, error = %e, "Encoding failed");
                continue;
            }
            Err(e) => {
                error!(worker_id = worker_id, chkblob_hash = %chkblob_hash, error = %e, "Encoding task panicked");
                continue;
            }
        };

        let encode_ms = start.elapsed().as_millis();

        if tx
            .send(UploadJob {
                webp_data,
                minimap_png,
                map_id: job.map_id,
                chkblob_hash: chkblob_hash.clone(),
                mapblob_hash: mapblob_hash.clone(),
                timings: PipelineTimings {
                    download_ms: job.download_ms,
                    download_source: job.download_source,
                    render_ms: job.render_ms,
                    encode_ms,
                    map_size: job.map_size,
                },
            })
            .await
            .is_err()
        {
            error!(worker_id = worker_id, chkblob_hash = %chkblob_hash, "Upload channel closed");
            break;
        }
    }

    info!(worker_id = worker_id, "Encode worker shutting down");
}

/// Upload worker: uploads WebP images to GSFS, marks DB, logs final pipeline summary.
async fn upload_worker(
    worker_id: usize,
    rx: async_channel::Receiver<UploadJob>,
    config: &Config,
    pool: &DbPool,
    client: &reqwest::Client,
    queue_refs: &QueueRefs,
) {
    info!(worker_id = worker_id, "Upload worker started");

    while let Ok(job) = rx.recv().await {
        let start = std::time::Instant::now();
        let webp_size = job.webp_data.len() as u64;

        if let Err(e) = common::gsfs::gsfs_put_map_image(
            client,
            &config.gsfsfe_endpoint,
            &job.chkblob_hash,
            job.webp_data,
        )
        .await
        {
            error!(
                worker_id = worker_id,
                chkblob_hash = %job.chkblob_hash,
                error = %e,
                "Failed to upload rendered image to GSFS"
            );
            continue;
        }

        if let Err(e) = common::gsfs::gsfs_put_minimap(
            client,
            &config.gsfsfe_endpoint,
            &job.chkblob_hash,
            job.minimap_png,
        )
        .await
        {
            error!(
                worker_id = worker_id,
                chkblob_hash = %job.chkblob_hash,
                error = %e,
                "Failed to upload minimap image to GSFS"
            );
            continue;
        }

        let upload_ms = start.elapsed().as_millis();

        if let Err(e) = db::mark_rendered(pool, &job.chkblob_hash).await {
            error!(
                worker_id = worker_id,
                chkblob_hash = %job.chkblob_hash,
                error = %e,
                "Failed to mark map as rendered in database"
            );
            continue;
        }

        let queues = queue_refs.snapshot();

        warn!(
            url = %format_args!("https://scmscx.com/map/{}", job.map_id),
            chkblob_hash = %job.chkblob_hash,
            mapblob_hash = %job.mapblob_hash,
            map_size = job.timings.map_size,
            webp_size = webp_size,
            download_src = job.timings.download_source,
            download_ms = job.timings.download_ms,
            render_ms = job.timings.render_ms,
            encode_ms = job.timings.encode_ms,
            upload_ms = upload_ms,
            q_download = queues.download,
            q_render = queues.render,
            q_encode = queues.encode,
            q_upload = queues.upload,
            "Pipeline complete"
        );
    }

    info!(worker_id = worker_id, "Upload worker shutting down");
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
    debug!("Authenticating with Backblaze B2");
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

    debug!(
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

    let mut file = tokio::fs::File::create(dest_path).await?;
    let mut all_bytes = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        all_bytes.extend_from_slice(&chunk);
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }

    tokio::io::AsyncWriteExt::flush(&mut file).await?;
    drop(file);

    debug!(
        chkblob_hash = %map.chkblob_hash,
        mapblob_hash = %map.mapblob_hash,
        size = all_bytes.len(),
        "Downloaded from B2, uploading to GSFS"
    );

    if let Err(e) = common::gsfs::gsfs_put_mapblob(
        client,
        &config.gsfsfe_endpoint,
        &map.mapblob_hash,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Integration test that renders a map through the full encode pipeline.
    ///
    /// Requires external files:
    /// - SC_DATA_PATH: Path to StarCraft installation with CASC data
    /// - TEST_MAP_URL: URL to download a .scx or .scm map file
    ///
    /// Optional:
    /// - OUTPUT_PATH: Path to save the rendered WebP (for visual inspection)
    ///
    /// Run with:
    /// ```bash
    /// xvfb-run -a -s "-screen 0 4096x4096x24" bash -c \
    ///   'SC_DATA_PATH=/starcraft TEST_MAP_URL=https://scmscx.com/api/maps/HASH \
    ///     cargo test -p bwrender test_render_pipeline -- --ignored --nocapture'
    /// ```
    const ALL_SKINS: &[(chkdraft_bindings::RenderSkin, &str)] = &[
        // (chkdraft_bindings::RenderSkin::Classic, "classic"),
        // (chkdraft_bindings::RenderSkin::RemasteredSd, "remastered_sd"),
        (
            chkdraft_bindings::RenderSkin::RemasteredHd2,
            "remastered_hd2",
        ),
        // (chkdraft_bindings::RenderSkin::RemasteredHd, "remastered_hd"),
        // (chkdraft_bindings::RenderSkin::CarbotHd2, "carbot_hd2"),
        // (chkdraft_bindings::RenderSkin::CarbotHd, "carbot_hd"),
    ];

    #[tokio::test]
    #[ignore]
    async fn test_render_pipeline() {
        use chkdraft_bindings::{GfxUtil, RenderOptions};

        let sc_data_path =
            std::env::var("SC_DATA_PATH").expect("SC_DATA_PATH environment variable must be set");
        let map_url =
            std::env::var("TEST_MAP_URL").expect("TEST_MAP_URL environment variable must be set");
        let output_dir =
            std::env::var("OUTPUT_DIR").unwrap_or_else(|_| "/tmp/test_render".to_string());

        std::fs::create_dir_all(&output_dir).expect("Failed to create output dir");

        // Download the map to a temp file
        println!("Downloading map from: {map_url}");
        let map_data = reqwest::get(&map_url)
            .await
            .expect("Failed to fetch map")
            .bytes()
            .await
            .expect("Failed to read map bytes");
        println!("Downloaded {} bytes", map_data.len());

        let temp_dir = std::env::temp_dir();
        let map_path = temp_dir.join("test_map.scx");
        std::fs::write(&map_path, &map_data).expect("Failed to write temp map file");

        // Create one GfxUtil/Renderer and reuse across skins.
        // Creating multiple GL contexts in the same process causes issues with
        // Mesa's software renderer (stale texture IDs across glfwTerminate/glfwInit cycles).
        let mut gfx = GfxUtil::new().expect("Failed to create GfxUtil");
        gfx.load_sc_data(&sc_data_path)
            .expect("Failed to load StarCraft data");

        let renderer = gfx
            .create_renderer(chkdraft_bindings::RenderSkin::Classic)
            .expect("Failed to create renderer");

        for &(skin, skin_name) in ALL_SKINS {
            println!("\n=== Rendering with skin: {skin_name} ===");

            renderer.set_skin(skin);

            let mut map = gfx
                .load_map(map_path.to_str().unwrap())
                .expect("Failed to load map");
            println!(
                "Map loaded: {}x{} tiles",
                map.tile_width(),
                map.tile_height()
            );

            map.simulate_anim(52);

            // Render map image
            let options = RenderOptions::default();
            let raw_image = renderer
                .get_raw_image(&map, &options)
                .expect("Failed to render map");
            println!(
                "Raw map image: {}x{} ({} bytes)",
                raw_image.width,
                raw_image.height,
                raw_image.rgb_data.len()
            );

            // Encode and write
            let webp_data = encode::encode_rgb_to_webp(
                &raw_image.rgb_data,
                raw_image.width,
                raw_image.height,
                70.0,
            )
            .expect("Failed to encode WebP");

            let decoder = webp::Decoder::new(&webp_data);
            let decoded = decoder.decode().expect("Failed to decode WebP");
            println!(
                "WebP: {} bytes, {}x{} pixels (lossy q90)",
                webp_data.len(),
                decoded.width(),
                decoded.height()
            );

            let webp_path = format!("{output_dir}/{skin_name}.webp");
            std::fs::write(&webp_path, &webp_data).expect("Failed to write WebP file");
            println!("Map saved to: {webp_path}");
        }

        // Render minimap once (skin-independent)
        let map = gfx
            .load_map(map_path.to_str().unwrap())
            .expect("Failed to load map");
        let minimap = renderer
            .get_raw_minimap(&map)
            .expect("Failed to render minimap");
        println!(
            "\nRaw minimap: {}x{} ({} bytes)",
            minimap.width,
            minimap.height,
            minimap.rgb_data.len()
        );

        let minimap_png =
            encode::encode_rgb_to_png(&minimap.rgb_data, minimap.width, minimap.height)
                .expect("Failed to encode minimap PNG");
        println!("Minimap PNG: {} bytes", minimap_png.len());

        let minimap_path = format!("{output_dir}/minimap.png");
        std::fs::write(&minimap_path, &minimap_png).expect("Failed to write minimap PNG");
        println!("Minimap saved to: {minimap_path}");

        // Clean up
        let _ = std::fs::remove_file(&map_path);

        println!("\nAll skins rendered successfully!");
    }
}
