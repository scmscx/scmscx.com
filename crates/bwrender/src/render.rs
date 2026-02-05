use anyhow::{Context, Result};
use chkdraft_bindings::{init_logging, GfxUtil, RenderOptions, RenderSkin, Renderer};
use std::sync::mpsc;
use std::thread;
use tokio::sync::oneshot;
use tracing::info;

/// Rendering context that manages a dedicated render thread.
///
/// OpenGL contexts are thread-local, so we need to ensure all OpenGL operations
/// happen on the same thread. This struct spawns a dedicated thread for rendering
/// and communicates with it via channels.
///
/// The renderer is created once and reused for all render requests to avoid
/// creating multiple OpenGL contexts which can cause conflicts.
pub struct RenderContext {
    sender: mpsc::Sender<RenderRequest>,
}

struct RenderRequest {
    map_path: String,
    anim_ticks: u64,
    webp_quality: f32,
    response_tx: oneshot::Sender<Result<Vec<u8>, String>>,
}

impl RenderContext {
    /// Initialize the rendering context by spawning a dedicated render thread.
    ///
    /// The render thread will own the GfxUtil and Renderer and handle all OpenGL operations.
    pub fn new(sc_data_path: &str, skin: RenderSkin) -> Result<Self> {
        let (request_sender, request_receiver) = mpsc::channel::<RenderRequest>();
        let (init_tx, init_rx) = mpsc::channel::<Result<(), String>>();

        let sc_data_path = sc_data_path.to_string();

        // Spawn a dedicated thread for all OpenGL operations
        thread::spawn(move || {
            // Initialize C++ logging first
            info!("Render thread: initializing C++ logging");
            if init_logging() {
                info!("Render thread: C++ logging initialized successfully");
            } else {
                info!("Render thread: WARNING - C++ logging may not be working");
            }

            // Initialize GfxUtil on this thread - OpenGL context will be bound here
            info!("Render thread: initializing GfxUtil with skin {:?}", skin);
            let mut gfx = match GfxUtil::new() {
                Ok(g) => g,
                Err(e) => {
                    let _ = init_tx.send(Err(format!("Failed to create GfxUtil: {}", e)));
                    return;
                }
            };

            info!(
                "Render thread: loading StarCraft data from {}",
                sc_data_path
            );
            if let Err(e) = gfx.load_sc_data(&sc_data_path) {
                let _ = init_tx.send(Err(format!("Failed to load SC data: {}", e)));
                return;
            }
            info!("Render thread: StarCraft data loaded successfully");

            // Create the renderer once and reuse it for all renders
            // This avoids creating multiple OpenGL contexts which causes conflicts
            info!("Render thread: creating renderer with skin {:?}", skin);
            let renderer = match gfx.create_renderer(skin) {
                Ok(r) => r,
                Err(e) => {
                    let _ = init_tx.send(Err(format!("Failed to create renderer: {}", e)));
                    return;
                }
            };
            info!("Render thread: renderer created successfully");

            // Signal successful initialization
            let _ = init_tx.send(Ok(()));

            // Process render requests - reuse the same renderer for all
            while let Ok(request) = request_receiver.recv() {
                let result = render_map_internal(
                    &gfx,
                    &renderer,
                    &request.map_path,
                    request.anim_ticks,
                    request.webp_quality,
                );
                let _ = request.response_tx.send(result);
            }
            info!("Render thread: shutting down");
        });

        // Wait for initialization to complete
        init_rx
            .recv()
            .context("Render thread died during initialization")?
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(RenderContext {
            sender: request_sender,
        })
    }

    /// Render a map file to WebP bytes.
    ///
    /// This sends the request to the dedicated render thread and waits for the result.
    pub async fn render_map(
        &self,
        map_path: &str,
        anim_ticks: u64,
        webp_quality: f32,
    ) -> Result<Vec<u8>> {
        let (response_tx, response_rx) = oneshot::channel();

        self.sender
            .send(RenderRequest {
                map_path: map_path.to_string(),
                anim_ticks,
                webp_quality,
                response_tx,
            })
            .map_err(|_| anyhow::anyhow!("Render thread died"))?;

        response_rx
            .await
            .context("Render thread died")?
            .map_err(|e| anyhow::anyhow!(e))
    }
}

fn render_map_internal(
    gfx: &GfxUtil,
    renderer: &Renderer,
    map_path: &str,
    anim_ticks: u64,
    webp_quality: f32,
) -> Result<Vec<u8>, String> {
    info!("render_map_internal: loading map from {}", map_path);
    let mut map = gfx
        .load_map(map_path)
        .map_err(|e| format!("Failed to load map: {}", e))?;
    info!(
        "render_map_internal: map loaded, size {}x{}",
        map.tile_width(),
        map.tile_height()
    );

    // Simulate animation (extends siege tanks, etc.)
    info!(
        "render_map_internal: simulating {} animation ticks",
        anim_ticks
    );
    map.simulate_anim(anim_ticks);
    info!("render_map_internal: animation simulation complete");

    info!(
        "render_map_internal: calling get_webp with quality {}",
        webp_quality
    );
    let options = RenderOptions {
        webp_quality,
        ..RenderOptions::default()
    };
    let webp_data = renderer
        .get_webp(&map, &options)
        .map_err(|e| format!("Failed to render map to WebP: {}", e))?;
    info!(
        "render_map_internal: get_webp returned {} bytes",
        webp_data.len()
    );

    Ok(webp_data)
}

/// Standalone render function for when you don't want to maintain context
#[allow(dead_code)]
pub async fn render_map_standalone(
    sc_data_path: &str,
    map_path: &str,
    skin: RenderSkin,
    anim_ticks: u64,
    webp_quality: f32,
) -> Result<Vec<u8>> {
    let sc_data_path = sc_data_path.to_string();
    let map_path = map_path.to_string();

    tokio::task::spawn_blocking(move || {
        let mut gfx = GfxUtil::new().context("Failed to create GfxUtil")?;
        gfx.load_sc_data(&sc_data_path)
            .context("Failed to load StarCraft data")?;

        let renderer = gfx
            .create_renderer(skin)
            .context("Failed to create renderer")?;

        let mut map = gfx.load_map(&map_path).context("Failed to load map")?;

        map.simulate_anim(anim_ticks);

        let options = RenderOptions {
            webp_quality,
            ..RenderOptions::default()
        };
        let webp_data = renderer
            .get_webp(&map, &options)
            .context("Failed to render map to WebP")?;

        Ok(webp_data)
    })
    .await
    .context("Render task panicked")?
}
