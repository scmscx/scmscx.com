//! axum server bootstrap and router (replaces the old actix `start()`).

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Extension, Request};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use axum::ServiceExt;
use bwcommon::MyError;
use common::register_gauge;
use futures::lock::Mutex;
use handlebars::{DirectorySourceOptions, Handlebars};
use tower::Layer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::GovernorLayer;
use tower_http::compression::CompressionLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::services::ServeDir;
use tracing::info;

use crate::api;
use crate::middleware as mw;
use crate::pumpers::{start_backblaze_pumper, start_gsfs_pumper};
use crate::ratelimit::UsernameLoginLimiter;
use crate::state::{BackblazeAuth, ManifestChunk};
use crate::uiv2;
use crate::util::is_dev_mode;
use crate::webutil::Pool;
use crate::{hacks, static_pages};

async fn setup_db() -> Result<Pool> {
    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        std::env::var("DB_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .as_str(),
        std::env::var("DB_PORT").unwrap().as_str(),
        std::env::var("DB_USER").unwrap().as_str(),
        std::env::var("DB_PASSWORD").unwrap().as_str(),
        std::env::var("DB_DATABASE")
            .unwrap_or_else(|_| std::env::var("DB_USER").unwrap())
            .as_str(),
    );
    let manager = bb8_postgres::PostgresConnectionManager::new(
        connection_string.parse()?,
        bb8_postgres::tokio_postgres::NoTls,
    );

    let pool = bb8_postgres::bb8::Pool::builder()
        .max_size(
            std::env::var("DB_CONNECTIONS")
                .unwrap_or_else(|_| "16".to_string())
                .parse::<u32>()?,
        )
        .min_idle(Some(1))
        .max_lifetime(Some(std::time::Duration::from_mins(1)))
        .idle_timeout(Some(std::time::Duration::from_secs(30)))
        .test_on_check_out(true)
        .build(manager)
        .await?;

    anyhow::Ok(pool)
}

fn register_handlebars() -> Result<Arc<Handlebars<'static>>> {
    let mut registry = Handlebars::new();

    registry.set_strict_mode(true);

    if is_dev_mode() {
        info!("DEV_MODE activated, template hot reloading");
        registry.set_dev_mode(true);
    }

    let mut options = DirectorySourceOptions::default();
    ".hbs".clone_into(&mut options.tpl_extension);
    registry
        .register_templates_directory(
            std::path::Path::new(std::env::var("ROOT_DIR").unwrap().as_str()).join("uiv2"),
            options,
        )
        .unwrap();

    registry
        .register_partial(
            "header.hbs",
            std::fs::read_to_string(
                std::path::Path::new(std::env::var("ROOT_DIR").unwrap().as_str())
                    .join("uiv2/header.hbs"),
            )?
            .as_str(),
        )
        .map_err(|err| anyhow::anyhow!("failed to unwrap. err: {:?}", err))?;

    registry
        .register_partial(
            "body.hbs",
            std::fs::read_to_string(
                std::path::Path::new(std::env::var("ROOT_DIR").unwrap().as_str())
                    .join("uiv2/body.hbs"),
            )?
            .as_str(),
        )
        .map_err(|err| anyhow::anyhow!("failed to unwrap. err: {:?}", err))?;

    Ok(Arc::new(registry))
}

/// Dev-mode fallback: proxy any unmatched request to the local vite server.
async fn dev_proxy(
    Extension(client): Extension<reqwest::Client>,
    req: Request,
) -> Result<Response, MyError> {
    let path_query = req
        .uri()
        .path_and_query()
        .map_or_else(|| req.uri().path(), |pq| pq.as_str());

    let url = format!("http://localhost:3000{path_query}");
    info!("proxying to {}", url);

    let upstream = client.get(&url).send().await?;

    let mut builder = Response::builder().status(upstream.status());
    for (name, value) in upstream.headers() {
        builder = builder.header(name, value);
    }
    Ok(builder
        .body(Body::from_stream(upstream.bytes_stream()))
        .unwrap())
}

pub(crate) async fn start() -> Result<()> {
    // Telemetry: install the panic hook, bring up the Prometheus scrape server
    // (PROMETHEUS_ENDPOINT, required), and start the runtime/process reporter.
    common::telemetry::init(env!("CARGO_PKG_VERSION")).await;

    // Fail loud on a Backblaze misconfiguration rather than silently not
    // uploading maps. Backblaze must be turned off explicitly, never by accident.
    if std::env::var("BACKBLAZE_DISABLED").as_deref() != Ok("true") {
        for var in [
            "BACKBLAZE_KEY_ID",
            "BACKBLAZE_APPLICATION_KEY",
            "BACKBLAZE_MAPBLOB_BUCKET",
        ] {
            std::env::var(var).map_err(|_| {
                anyhow::anyhow!(
                    "{var} not set (set BACKBLAZE_DISABLED=true to run without Backblaze)"
                )
            })?;
        }
    }

    let db_pool = setup_db().await?;

    // Telemetry: start the background reporter for the DB pool gauges.
    {
        let pool = db_pool.clone();
        tokio::spawn(async move {
            loop {
                let state = pool.state();
                for (label, value) in [
                    ("total", state.connections),
                    ("idle", state.idle_connections),
                ] {
                    register_gauge!(
                        "scmscx",
                        db_pool_connections,
                        "bb8 database pool connection count, by state",
                        state = label
                    )
                    .set(i64::from(value));
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    let handlebars = register_handlebars()?;

    let tx = bwcommon::create_mixpanel_channel().await;

    let manifest: crate::state::Manifest = Arc::new(serde_json::from_str::<
        std::collections::HashMap<String, ManifestChunk>,
    >(
        tokio::fs::read_to_string("./dist/.vite/manifest.json")
            .await?
            .as_str(),
    )?);

    let backblaze_auth: crate::state::BackblazeAuthState =
        Arc::new(Mutex::new(BackblazeAuth::default()));

    // Pump files up to backblaze
    let reqwest_client = reqwest::Client::new();

    start_backblaze_pumper(reqwest_client.clone()).await?;
    start_gsfs_pumper(reqwest_client.clone()).await?;

    let username_limiter = Arc::new(UsernameLoginLimiter::new());

    // Per-IP rate limits for the auth endpoints (was actix-governor).
    let login_governor = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .period(std::time::Duration::from_secs(3))
            .burst_size(20)
            .finish()
            .expect("valid governor config"),
    );
    let register_governor = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .period(std::time::Duration::from_mins(20))
            .burst_size(3)
            .finish()
            .expect("valid governor config"),
    );

    // DashMap state stores grow per distinct key with no built-in eviction.
    // Periodically drop entries whose GCRA cell has fully refilled.
    {
        let ip_login = login_governor.limiter().clone();
        let ip_register = register_governor.limiter().clone();
        let username = username_limiter.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_mins(1));
            tick.tick().await;
            loop {
                tick.tick().await;
                ip_login.retain_recent();
                ip_login.shrink_to_fit();
                ip_register.retain_recent();
                ip_register.shrink_to_fit();
                username.retain_recent();
            }
        });
    }

    let router = Router::new()
        // core (formerly inline in actix.rs)
        .route("/api/maps/{mapblob_hash}", get(api::core::get_map))
        .route("/api/replays/{replay_id}", get(api::core::get_replay))
        .route("/api/recent_activity", get(api::core::recent_activity))
        .route("/api/minimap/{chk_id}", get(api::core::get_minimap))
        .route(
            "/api/search_result_popup/{map_id}",
            get(api::core::get_search_result_popup),
        )
        .route(
            "/api/minimap_resized/{chk_id}",
            get(api::core::get_minimap_resized),
        )
        .route(
            "/api/get_selection_of_random_maps",
            get(api::core::get_selection_of_random_maps),
        )
        .route(
            "/api/get_selection_of_random_nsfw_maps",
            get(api::core::get_selection_of_random_nsfw_maps),
        )
        .route(
            "/api/tags/{map_id}",
            get(api::core::get_tags).post(api::core::set_tags),
        )
        .route("/api/addtags/{map_id}", post(api::core::add_tags))
        // api
        .route(
            "/api/flags/{map_id}/{flag}",
            get(api::flags::get_flag).post(api::flags::set_flag),
        )
        .route(
            "/api/change-password",
            post(api::change_password::post_handler),
        )
        .route(
            "/api/change-username",
            post(api::change_username::post_handler),
        )
        .route(
            "/api/login",
            post(api::login::post_handler).layer(GovernorLayer::new(login_governor)),
        )
        .route(
            "/api/register",
            post(api::register::post_handler).layer(GovernorLayer::new(register_governor)),
        )
        .route("/api/logout", get(api::logout::handler))
        .route("/sitemap.txt", get(api::sitemap::handler))
        .route("/a.txt", get(api::sitemap::handlera))
        .route("/b.txt", get(api::sitemap::handlerb))
        .route("/c.txt", get(api::sitemap::handlerc))
        .route("/api/chk/strings/{map_id}", get(api::chk::get_chk_strings))
        .route(
            "/api/chk/riff_chunks/{map_id}",
            get(api::chk::get_chk_riff_chunks),
        )
        .route("/api/chk/json/{map_id}", get(api::chk::get_chk_json))
        .route("/api/chk/trig/{map_id}", get(api::chk::get_chk_trig_json))
        .route("/api/chk/mbrf/{map_id}", get(api::chk::get_chk_mbrf_json))
        .route("/api/chk/eups/{map_id}", get(api::chk::get_eups))
        .route("/api/chk/{chk_hash}", get(api::chk::download_chk))
        .route(
            "/api/similar_maps/{map_id}",
            get(api::similar_maps::handler),
        )
        // uiv2 api
        .route("/api/uiv2/featured_maps", get(api::uiv2::featured_maps))
        .route(
            "/api/uiv2/last_viewed_maps",
            get(api::uiv2::last_viewed_maps),
        )
        .route(
            "/api/uiv2/last_downloaded_maps",
            get(api::uiv2::last_downloaded_maps),
        )
        .route(
            "/api/uiv2/last_uploaded_maps",
            get(api::uiv2::last_uploaded_maps),
        )
        .route(
            "/api/uiv2/last_uploaded_replays",
            get(api::uiv2::last_uploaded_replays),
        )
        .route(
            "/api/uiv2/most_viewed_maps",
            get(api::uiv2::most_viewed_maps),
        )
        .route(
            "/api/uiv2/most_downloaded_maps",
            get(api::uiv2::most_downloaded_maps),
        )
        .route("/api/uiv2/minimap/{map_id}", get(api::uiv2::get_minimap))
        .route(
            "/api/uiv2/is_session_valid",
            post(api::uiv2::is_session_valid),
        )
        .route(
            "/api/uiv2/map_info/{map_id}",
            get(api::uiv2::map_info::map_info),
        )
        .route(
            "/api/uiv2/filenames/{map_id}",
            get(api::uiv2::filenames::filenames),
        )
        .route(
            "/api/uiv2/timestamps/{map_id}",
            get(api::uiv2::timestamps::timestamps),
        )
        .route(
            "/api/uiv2/filenames2/{map_id}",
            get(api::uiv2::filenames2::filenames2),
        )
        .route(
            "/api/uiv2/replays/{map_id}",
            get(api::uiv2::replays::replays),
        )
        .route("/api/uiv2/units/{map_id}", get(api::uiv2::units::units))
        .route(
            "/api/uiv2/search/{query}",
            get(api::uiv2::search::search_query),
        )
        .route("/api/uiv2/search", get(api::uiv2::search::search))
        .route("/api/uiv2/img/{map_id}", get(api::uiv2::get_map_image))
        .route("/api/uiv2/random/{query}", get(api::random::handler))
        .route("/api/uiv2/random", get(api::random::handler_noquery))
        .route("/api/uiv2/upload-map", post(api::uiv2::upload::upload_map))
        .route("/api/uiv2/logout", get(api::uiv2::logout::logout2))
        // uiv2 ssr
        .route("/", get(uiv2::index::index))
        .route("/search", get(uiv2::index::search_no_query))
        .route("/search/{query}", get(uiv2::index::search_query))
        .route("/map/{map_id}", get(uiv2::index::map))
        .route("/upload", get(uiv2::index::upload))
        .route("/about", get(uiv2::index::about))
        .route("/user/{username}", get(uiv2::index::user))
        .route("/login", get(uiv2::index::login))
        .route("/moderation", get(uiv2::index::moderation))
        .route("/site.webmanifest", get(uiv2::index::webmanifest))
        // hacks
        .route("/api/denormalize/{map_id}", get(hacks::denormalize))
        .route("/api/denormalize_all", get(hacks::denormalize_all))
        // static pages
        .route("/map", get(static_pages::redirect_map))
        .route("/replay", get(static_pages::redirect_replay))
        // static assets
        .nest_service("/assets", ServeDir::new("./dist/assets"))
        .nest_service("/uiv2/assets", ServeDir::new("./dist/assets"))
        // Upload can be large; lift axum's 2 MB default body cap.
        .layer(DefaultBodyLimit::disable());

    // Fallback: dev proxy to the vite server, or the public/ static dir in prod.
    let router = if is_dev_mode() {
        info!("dev mode active, adding local proxy to localhost:3000");
        router.fallback(dev_proxy)
    } else {
        router.fallback_service(ServeDir::new("./public").append_index_html_on_directories(true))
    };

    // Shared state as request extensions, plus the middleware stack. Order is
    // outermost-first: TraceID sees every request first and logs the final
    // status last; UserSession/TrackingAnalytics run before PostgresLogging so
    // their extensions are populated when it captures them.
    let router = router.layer(
        tower::ServiceBuilder::new()
            .layer(Extension(db_pool.clone()))
            .layer(Extension(reqwest_client.clone()))
            .layer(Extension(handlebars))
            .layer(Extension(manifest))
            .layer(Extension(backblaze_auth))
            .layer(Extension(username_limiter))
            .layer(axum::middleware::from_fn(mw::trace_id))
            .layer(axum::middleware::from_fn(mw::tracking_analytics))
            .layer(axum::middleware::from_fn(mw::language))
            .layer(axum::middleware::from_fn({
                let pool = db_pool.clone();
                move |req, next| mw::user_session(pool.clone(), req, next)
            }))
            .layer(axum::middleware::from_fn({
                let pool = db_pool.clone();
                let tx = tx.clone();
                move |req, next| mw::postgres_logging(pool.clone(), tx.clone(), req, next)
            }))
            .layer(axum::middleware::from_fn(mw::cache_html))
            .layer(axum::middleware::from_fn(mw::metrics))
            .layer(CompressionLayer::new()),
    );

    // NormalizePath must wrap the router (before routing) so trailing slashes
    // are trimmed prior to route matching.
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("listening on {bind_addr}");

    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    )
    .await?;

    anyhow::Ok(())
}
