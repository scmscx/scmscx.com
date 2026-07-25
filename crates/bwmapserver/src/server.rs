//! The web layer: shared application state, the router, and the axum server
//! bootstrap.
//!
//! Request handlers live in `api` (and `uiv2`/`static_pages` for rendered
//! pages); this file only wires them to routes. Cross-cutting axum glue that
//! never belonged to a single handler (the `Pool` alias, the `MaybeUser`
//! extractor, cookie helpers, real-client-IP resolution) lives in `webutil`.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Extension, Request};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Router, ServiceExt};
use bwcommon::MyError;
use common::{register_counter, register_gauge};
use futures::lock::Mutex;
use handlebars::DirectorySourceOptions;
use serde::Deserialize;
use tower::Layer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::GovernorLayer;
use tower_http::compression::CompressionLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::services::ServeDir;
use tracing::info;

use crate::api::maps::{BackblazeAuth, BackblazeAuthState};
use crate::pumpers::{start_backblaze_pumper, start_gsfs_pumper};
use crate::ratelimit::UsernameLoginLimiter;
use crate::util::is_dev_mode;
use crate::webutil::Pool;
use crate::{api, middleware as mw, static_pages, uiv2};

// Shared, cheaply-cloneable handles injected into handlers as `Extension`s.
pub type Manifest = Arc<std::collections::HashMap<String, ManifestChunk>>;
pub type Handlebars = Arc<handlebars::Handlebars<'static>>;

#[derive(Clone, Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ManifestChunk {
    pub file: String,
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub src: String,
    #[allow(dead_code)]
    pub isEntry: Option<bool>,
    pub css: Option<Vec<String>>,
}

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
    // Read via the shared helper so the `state="max"` gauge can never drift from
    // the ceiling the pool was actually built with.
    let manager = bb8_postgres::PostgresConnectionManager::new(
        connection_string.parse()?,
        bb8_postgres::tokio_postgres::NoTls,
    );

    let pool = bb8_postgres::bb8::Pool::builder()
        .max_size(db_max_connections()?)
        .min_idle(Some(1))
        .max_lifetime(Some(std::time::Duration::from_mins(1)))
        .idle_timeout(Some(std::time::Duration::from_secs(30)))
        .test_on_check_out(true)
        .build(manager)
        .await?;

    anyhow::Ok(pool)
}

/// The configured pool ceiling, shared by [`setup_db`] and the telemetry reporter.
fn db_max_connections() -> Result<u32> {
    Ok(std::env::var("DB_CONNECTIONS")
        .unwrap_or_else(|_| "16".to_string())
        .parse::<u32>()?)
}

fn register_handlebars() -> Result<Handlebars> {
    let mut registry = handlebars::Handlebars::new();

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

    // Telemetry: start the background reporter for the DB pool gauges and the
    // cumulative pool statistics bb8 keeps internally.
    {
        let pool = db_pool.clone();
        let max_size = db_max_connections()?;
        tokio::spawn(async move {
            // bb8's `Statistics` are cumulative totals; Prometheus counters are
            // fed deltas, so remember the previous reading and advance by the
            // difference.
            let mut prev = [0u64; 8];
            loop {
                let state = pool.state();
                let s = &state.statistics;

                for (label, value) in [
                    ("total", state.connections),
                    ("idle", state.idle_connections),
                    ("max", max_size),
                ] {
                    register_gauge!(
                        "scmscx",
                        db_pool_connections,
                        "bb8 database pool connection count, by state",
                        state = label
                    )
                    .set(i64::from(value));
                }

                let cur = [
                    s.get_direct,
                    s.get_waited,
                    s.get_timed_out,
                    s.connections_created,
                    s.connections_closed_broken,
                    s.connections_closed_invalid,
                    s.connections_closed_max_lifetime,
                    s.connections_closed_idle_timeout,
                ];
                // Saturating throughout: bb8 never resets these, but a wrap would
                // otherwise panic in debug and poison the series.
                for (i, outcome) in ["direct", "waited", "timed_out"].iter().enumerate() {
                    register_counter!(
                        "scmscx",
                        db_pool_gets,
                        "Connection checkouts from the bb8 pool, by whether they had to wait",
                        outcome = outcome
                    )
                    .inc_by(cur[i].saturating_sub(prev[i]));
                }
                register_counter!(
                    "scmscx",
                    db_pool_connections_created,
                    "Postgres connections opened by the bb8 pool"
                )
                .inc_by(cur[3].saturating_sub(prev[3]));
                for (i, reason) in ["broken", "invalid", "max_lifetime", "idle_timeout"]
                    .iter()
                    .enumerate()
                {
                    register_counter!(
                        "scmscx",
                        db_pool_connections_closed,
                        "Postgres connections retired by the bb8 pool, by reason",
                        reason = reason
                    )
                    .inc_by(cur[4 + i].saturating_sub(prev[4 + i]));
                }

                prev = cur;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    let handlebars = register_handlebars()?;

    let manifest: Manifest = Arc::new(serde_json::from_str::<
        std::collections::HashMap<String, ManifestChunk>,
    >(
        tokio::fs::read_to_string("./dist/.vite/manifest.json")
            .await?
            .as_str(),
    )?);

    let backblaze_auth: BackblazeAuthState = Arc::new(Mutex::new(BackblazeAuth::default()));

    // Pump files up to backblaze
    let reqwest_client = reqwest::Client::new();

    start_backblaze_pumper(reqwest_client.clone()).await?;
    start_gsfs_pumper(reqwest_client.clone()).await?;

    let username_limiter = Arc::new(UsernameLoginLimiter::new());

    // Per-IP rate limits for the auth endpoints.
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
        .route("/api/maps/{mapblob_hash}", get(api::maps::get_map))
        .route("/api/replays/{replay_id}", get(api::replays::get_replay))
        .route(
            "/api/recent_activity",
            get(api::recent_activity::recent_activity),
        )
        .route("/api/minimap/{chk_id}", get(api::minimap::get_minimap))
        .route(
            "/api/search_result_popup/{map_id}",
            get(api::search_result_popup::get_search_result_popup),
        )
        .route(
            "/api/minimap_resized/{chk_id}",
            get(api::minimap::get_minimap_resized),
        )
        .route(
            "/api/get_selection_of_random_maps",
            get(api::selection::get_selection_of_random_maps),
        )
        .route(
            "/api/get_selection_of_random_nsfw_maps",
            get(api::selection::get_selection_of_random_nsfw_maps),
        )
        .route(
            "/api/tags/{map_id}",
            get(api::tags::get_tags).post(api::tags::set_tags),
        )
        .route("/api/addtags/{map_id}", post(api::tags::add_tags))
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
        .route("/api/chk/{chk_hash}/map_img", get(api::chk::get_map_img))
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
        .route(
            "/api/denormalize/{map_id}",
            get(api::denormalize::denormalize),
        )
        .route(
            "/api/denormalize_all",
            get(api::denormalize::denormalize_all),
        )
        // static pages
        .route("/map", get(static_pages::redirect_map))
        .route("/replay", get(static_pages::redirect_replay))
        // static assets
        .nest_service("/assets", ServeDir::new("./dist/assets"))
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
            // Outermost timing layer, so it spans every middleware below it —
            // including `user_session` and `postgres_logging`, which hit the
            // database and are invisible to the `metrics` histogram further down.
            .layer(axum::middleware::from_fn(mw::end_to_end_duration))
            // Runs last on the response, after every inner
            // layer and the handler have set their headers, and defaults any
            // response still missing a Cache-Control to `no-store` (edge caching is
            // opt-in — see `default_no_store`).
            .layer(axum::middleware::from_fn(mw::default_no_store))
            .layer(axum::middleware::from_fn(mw::trace_id))
            .layer(axum::middleware::from_fn(mw::tracking_analytics))
            .layer(axum::middleware::from_fn(mw::language))
            .layer(axum::middleware::from_fn({
                let pool = db_pool.clone();
                move |req, next| mw::user_session(pool.clone(), req, next)
            }))
            .layer(axum::middleware::from_fn({
                let pool = db_pool.clone();
                move |req, next| mw::postgres_logging(pool.clone(), req, next)
            }))
            .layer(axum::middleware::from_fn(mw::immutable_assets))
            .layer(axum::middleware::from_fn(mw::metrics))
            .layer(CompressionLayer::new())
            // Innermost so it runs first on the response: it hashes the handler's
            // uncompressed HTML for the ETag and can short-circuit to 304 before
            // compression does any work.
            .layer(axum::middleware::from_fn(mw::etag)),
    );

    // NormalizePath must wrap the router (before routing) so trailing slashes
    // are trimmed prior to route matching.
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    // Prefer an inherited, already-bound listener (`BIND_FD`) over binding
    // `BIND_ADDR`. The E2E harness hands the socket down this way so the port is
    // chosen and held race-free — no ephemeral-port grab/close/re-bind window.
    // See `common::telemetry::take_listener_from_env`.
    let listener = if let Some(std_listener) = common::telemetry::take_listener_from_env("BIND_FD")
    {
        std_listener.set_nonblocking(true)?;
        tokio::net::TcpListener::from_std(std_listener)?
    } else {
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        tokio::net::TcpListener::bind(&bind_addr).await?
    };
    info!("listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    )
    .await?;

    anyhow::Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_chunk_deserializes_from_vite_manifest() {
        // A realistic slice of dist/.vite/manifest.json.
        let json = r#"{
            "index.html": {
                "file": "assets/index-abc123.js",
                "name": "index",
                "src": "index.html",
                "isEntry": true,
                "css": ["assets/index-def456.css"]
            }
        }"#;

        let map: std::collections::HashMap<String, ManifestChunk> =
            serde_json::from_str(json).unwrap();
        let chunk = &map["index.html"];
        assert_eq!(chunk.file, "assets/index-abc123.js");
        assert_eq!(chunk.name.as_deref(), Some("index"));
        assert_eq!(chunk.isEntry, Some(true));
        assert_eq!(
            chunk.css.as_deref(),
            Some(&["assets/index-def456.css".to_string()][..])
        );
    }

    #[test]
    fn manifest_chunk_tolerates_missing_optional_fields() {
        // Non-entry chunks omit isEntry/css.
        let json = r#"{ "file": "assets/x.js", "src": "x.ts" }"#;
        let chunk: ManifestChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.file, "assets/x.js");
        assert!(chunk.name.is_none());
        assert!(chunk.isEntry.is_none());
        assert!(chunk.css.is_none());
    }

    #[test]
    fn backblaze_auth_default_is_empty() {
        let auth = BackblazeAuth::default();
        assert_eq!(auth.version, 0);
        assert!(auth.auth.is_none());
    }
}

#[cfg(test)]
mod router_tests {
    //! Wiring tests for the router-level tower layers. `start()` itself needs a
    //! DB/manifest/env and can't run here, but the `NormalizePathLayer` wrapping
    //! the router (see the comment above) is a pure routing concern we can lock
    //! down against version bumps of axum/tower-http.

    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use tower::{Layer, ServiceExt};
    use tower_http::normalize_path::NormalizePathLayer;

    async fn status(uri: &str) -> StatusCode {
        let router = Router::new()
            .route("/about", get(|| async { "about" }))
            .route("/map/{id}", get(|| async { "map" }));
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        app.oneshot(HttpRequest::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn trailing_slash_is_trimmed_before_routing() {
        // The whole reason NormalizePath wraps (rather than layers inside) the
        // router: `/about/` must match the `/about` route.
        assert_eq!(status("/about").await, StatusCode::OK);
        assert_eq!(status("/about/").await, StatusCode::OK);
        assert_eq!(status("/map/5/").await, StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_is_404() {
        assert_eq!(status("/does-not-exist").await, StatusCode::NOT_FOUND);
    }
}
