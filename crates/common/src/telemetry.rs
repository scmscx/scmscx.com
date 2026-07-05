//! Prometheus telemetry shared between the web server (`bwmapserver`) and the
//! renderer (`bwrender`).
//!
//! This mirrors the approach used in the sibling `gsfs` project: a single
//! process-global [`REGISTRY`] and a set of `register_*!` macros used inline at
//! the point a metric is recorded.
//!
//! Unlike a naive macro, ours is safe to invoke for the *same metric name from
//! any number of call sites*: every metric family is created once and cached in
//! a global name-keyed table (see [`get_or_register_counter`] and friends), and
//! each call site additionally caches its handle in a private `OnceLock` so the
//! global table is touched only once per site. All call sites for a given name
//! therefore share one aggregated family. Labels use a uniform
//! `Vec<(&'static str, String)>` so the family type does not depend on which
//! labels a given site happens to use.

use lazy_static::lazy_static;
use prometheus_client::metrics::{
    counter::Counter, family::Family, gauge::Gauge, histogram::Histogram,
};
use prometheus_client::registry::Registry;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Mutex;

/// Uniform label set: an ordered list of `(name, value)` pairs. The `register_*!`
/// macros sort these, so label order at the call site does not matter.
pub type Labels = Vec<(&'static str, String)>;

pub type CounterFamily = Family<Labels, Counter>;
pub type GaugeFamily = Family<Labels, Gauge>;
pub type HistogramFamily = Family<Labels, Histogram>;

lazy_static! {
    /// Process-global metric registry. Everything registered here is exposed on
    /// the `/metrics` endpoint.
    pub static ref REGISTRY: Mutex<Registry> =
        Mutex::new(prometheus_client::registry::Registry::default());

    /// Name -> metric family cache, so a metric used from several call sites is
    /// registered once and shared. Values are `CounterFamily` / `GaugeFamily` /
    /// `HistogramFamily`, downcast on retrieval.
    static ref FAMILIES: Mutex<HashMap<&'static str, Box<dyn Any + Send + Sync>>> =
        Mutex::new(HashMap::new());
}

/// Fetch the metric family for `name`, registering it on first use (via `make`)
/// and caching it in the global table so later call sites for the same name
/// share one family. The three typed helpers below are thin instantiations.
fn get_or_register<M>(name: &'static str, help: &'static str, make: impl FnOnce() -> M) -> M
where
    M: prometheus_client::registry::Metric + Clone + Send + Sync + 'static,
{
    // Recover from a poisoned lock rather than propagating the panic: telemetry
    // is best-effort, and one thread panicking while registering a metric must
    // not permanently break registration for every other metric.
    let mut families = FAMILIES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(existing) = families.get(name) {
        return existing
            .downcast_ref::<M>()
            .expect("metric name reused with a different metric type")
            .clone();
    }
    let family = make();
    REGISTRY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .register(name, help, family.clone());
    families.insert(name, Box::new(family.clone()));
    family
}

/// Fetch the counter family for `name`, registering it on first use. Prefer the
/// [`register_counter!`] macro over calling this directly.
pub fn get_or_register_counter(name: &'static str, help: &'static str) -> CounterFamily {
    get_or_register(name, help, CounterFamily::default)
}

/// Fetch the gauge family for `name`, registering it on first use. Prefer the
/// [`register_gauge!`] macro.
pub fn get_or_register_gauge(name: &'static str, help: &'static str) -> GaugeFamily {
    get_or_register(name, help, GaugeFamily::default)
}

/// Fetch the histogram family for `name`, registering it on first use with the
/// given per-series constructor. Prefer the [`register_histogram!`] macro. The
/// constructor from the *first* call site wins; later sites reuse it.
pub fn get_or_register_histogram(
    name: &'static str,
    help: &'static str,
    constructor: fn() -> Histogram,
) -> HistogramFamily {
    get_or_register(name, help, || {
        HistogramFamily::new_with_constructor(constructor)
    })
}

/// Record on a counter, e.g.
/// `register_counter!("scmscx", http_requests_total, "help", method = m, status = s).inc();`
#[macro_export]
macro_rules! register_counter {
    ($prefix:literal, $id:ident, $help:literal $(, $key:ident = $val:expr)* $(,)?) => {{
        static CELL: std::sync::OnceLock<$crate::telemetry::CounterFamily> =
            std::sync::OnceLock::new();
        let family = CELL.get_or_init(|| {
            $crate::telemetry::get_or_register_counter(
                concat!($prefix, "_", stringify!($id)),
                $help,
            )
        });
        #[allow(unused_mut)]
        let mut labels: $crate::telemetry::Labels =
            vec![$((stringify!($key), ($val).to_string()),)*];
        labels.sort_unstable();
        // Bind to a local so the internal lock guard from `get_or_create` is
        // dropped here, before this block returns; otherwise the guard (which is
        // not `Send`) can be held across a later `.await` at the call site.
        let metric = family.get_or_create(&labels).clone();
        metric
    }};
}

/// Record on a gauge, e.g.
/// `register_gauge!("scmscx", queue_depth, "help", queue = q).set(n);`
#[macro_export]
macro_rules! register_gauge {
    ($prefix:literal, $id:ident, $help:literal $(, $key:ident = $val:expr)* $(,)?) => {{
        static CELL: std::sync::OnceLock<$crate::telemetry::GaugeFamily> =
            std::sync::OnceLock::new();
        let family = CELL.get_or_init(|| {
            $crate::telemetry::get_or_register_gauge(
                concat!($prefix, "_", stringify!($id)),
                $help,
            )
        });
        #[allow(unused_mut)]
        let mut labels: $crate::telemetry::Labels =
            vec![$((stringify!($key), ($val).to_string()),)*];
        labels.sort_unstable();
        // Bind to a local so the internal lock guard from `get_or_create` is
        // dropped here, before this block returns; otherwise the guard (which is
        // not `Send`) can be held across a later `.await` at the call site.
        let metric = family.get_or_create(&labels).clone();
        metric
    }};
}

/// Observe on a histogram, e.g.
/// `register_histogram!("scmscx", dur_seconds, "help", latency_buckets(), stage = s).observe(secs);`
///
/// The `$buckets` expression is evaluated once per created series inside a
/// non-capturing constructor, so it must be a free function call such as
/// `common::telemetry::latency_buckets()`.
#[macro_export]
macro_rules! register_histogram {
    ($prefix:literal, $id:ident, $help:literal, $buckets:expr $(, $key:ident = $val:expr)* $(,)?) => {{
        static CELL: std::sync::OnceLock<$crate::telemetry::HistogramFamily> =
            std::sync::OnceLock::new();
        let family = CELL.get_or_init(|| {
            $crate::telemetry::get_or_register_histogram(
                concat!($prefix, "_", stringify!($id)),
                $help,
                || prometheus_client::metrics::histogram::Histogram::new(($buckets).into_iter()),
            )
        });
        #[allow(unused_mut)]
        let mut labels: $crate::telemetry::Labels =
            vec![$((stringify!($key), ($val).to_string()),)*];
        labels.sort_unstable();
        // Bind to a local so the internal lock guard from `get_or_create` is
        // dropped here, before this block returns; otherwise the guard (which is
        // not `Send`) can be held across a later `.await` at the call site.
        let metric = family.get_or_create(&labels).clone();
        metric
    }};
}

/// Latency histogram buckets (seconds), ~30 buckets spanning 1ms to 60s.
pub fn latency_buckets() -> Vec<f64> {
    vec![
        0.001, 0.0015, 0.0025, 0.004, 0.005, 0.0075, 0.01, 0.015, 0.025, 0.04, 0.05, 0.075, 0.1,
        0.15, 0.25, 0.4, 0.5, 0.75, 1.0, 1.5, 2.5, 4.0, 5.0, 7.5, 10.0, 15.0, 20.0, 30.0, 45.0,
        60.0,
    ]
}

/// Byte-size histogram buckets, 19 buckets spanning 1KiB to 256MiB (powers of two).
pub fn size_buckets() -> Vec<f64> {
    vec![
        1024.0,
        2048.0,
        4096.0,
        8192.0,
        16384.0,
        32768.0,
        65536.0,
        131_072.0,
        262_144.0,
        524_288.0,
        1_048_576.0,
        2_097_152.0,
        4_194_304.0,
        8_388_608.0,
        16_777_216.0,
        33_554_432.0,
        67_108_864.0,
        134_217_728.0,
        268_435_456.0,
    ]
}

/// Encode the whole registry into the OpenMetrics text exposition format.
pub fn encode_metrics() -> String {
    let mut buffer = String::new();
    // Recover from poison so a scrape still succeeds after some other thread
    // panicked while holding the registry lock.
    let registry = REGISTRY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // Encoding into a String only fails on a formatter error, which does not
    // happen for String; fall back to whatever was written so far.
    let _ = prometheus_client::encoding::text::encode(&mut buffer, &registry);
    buffer
}

/// Install a panic hook that increments `scmscx_panics_total` on every panic and
/// then delegates to the previously installed hook (so logging/aborting is
/// preserved). A reset-to-zero of this counter is itself a useful restart signal.
///
/// The counter is resolved **once, here**, and the hook itself performs only a
/// lock-free atomic increment. This is deliberate: a panic can fire while the
/// panicking thread already holds the (non-reentrant) `FAMILIES`/`REGISTRY`
/// mutex used by metric registration, so a hook that registered a metric would
/// re-lock that mutex on the same thread and deadlock. It also carries no
/// `location` label — the panic location is already in the log line emitted by
/// the delegated hook, and a label-less counter needs no per-panic map lookup.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    let panics = register_counter!(
        "scmscx",
        panics_total,
        "Total panics observed by the panic hook."
    );
    std::panic::set_hook(Box::new(move |info| {
        panics.inc();
        previous(info);
    }));
}

/// Spawn a background task that periodically publishes process-level and tokio
/// runtime gauges. Must be called from within a tokio runtime.
///
/// Uses only tokio's *stable* runtime metrics so it works without the
/// `tokio_unstable` cfg. Also publishes `scmscx_build_info` (always 1, carrying
/// the crate version) and process uptime.
pub fn spawn_runtime_metrics_reporter(version: &'static str) {
    let start = std::time::Instant::now();
    let handle = tokio::runtime::Handle::current();

    register_gauge!(
        "scmscx",
        build_info,
        "Always 1; the version label carries the running build version",
        version = version
    )
    .set(1);

    if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        register_gauge!(
            "scmscx",
            process_start_time_seconds,
            "Unix time (seconds) at which the process started"
        )
        .set(now.as_secs() as i64);
    }

    tokio::spawn(async move {
        loop {
            let m = handle.metrics();

            register_gauge!(
                "scmscx",
                tokio_workers,
                "Number of tokio runtime worker threads"
            )
            .set(m.num_workers() as i64);

            register_gauge!(
                "scmscx",
                tokio_alive_tasks,
                "Number of currently alive tokio tasks"
            )
            .set(m.num_alive_tasks() as i64);

            register_gauge!(
                "scmscx",
                tokio_global_queue_depth,
                "Number of tasks in the tokio global run queue"
            )
            .set(m.global_queue_depth() as i64);

            register_gauge!(
                "scmscx",
                process_uptime_seconds,
                "Seconds since this process started"
            )
            .set(start.elapsed().as_secs() as i64);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}

/// One-call telemetry bootstrap for a binary. Installs the panic hook, starts
/// the Prometheus scrape server on the address in `PROMETHEUS_ENDPOINT`
/// (required — panics if unset or unparseable, by design), and spawns the
/// runtime/process gauge reporter tagged with `version`. Must be called from
/// within a tokio runtime.
pub async fn init(version: &'static str) {
    install_panic_hook();
    let endpoint: std::net::SocketAddr = std::env::var("PROMETHEUS_ENDPOINT")
        .expect("PROMETHEUS_ENDPOINT not set")
        .parse()
        .expect("PROMETHEUS_ENDPOINT must be a socket address, e.g. 0.0.0.0:9101");
    init_prometheus_server(endpoint).await;
    spawn_runtime_metrics_reporter(version);
}

/// Start the Prometheus scrape server on `addr`, serving `/metrics` (and a
/// trivial `/health`). Runs in a spawned task; a bind failure is logged but does
/// not bring the service down — metrics are best-effort.
pub async fn init_prometheus_server(addr: std::net::SocketAddr) {
    use axum::routing::get;

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("failed to bind prometheus metrics server on {addr}: {e}");
            return;
        }
    };

    tracing::info!("prometheus metrics server listening on http://{addr}/metrics");

    let router = axum::Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(|| async { "ok" }));

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            tracing::error!("prometheus metrics server exited: {e}");
        }
    });
}

async fn metrics_handler() -> impl axum::response::IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )],
        encode_metrics(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_metric_from_two_sites_aggregates() {
        // Two distinct call sites for the same counter name must share one series.
        register_counter!(
            "scmscx",
            telemetry_test_shared,
            "A shared test counter",
            kind = "x"
        )
        .inc();
        register_counter!(
            "scmscx",
            telemetry_test_shared,
            "A shared test counter",
            kind = "x"
        )
        .inc();

        register_gauge!("scmscx", telemetry_test_gauge, "A test gauge").set(7);

        register_histogram!(
            "scmscx",
            telemetry_test_histogram,
            "A test histogram",
            latency_buckets(),
            stage = "s"
        )
        .observe(0.42);

        let output = encode_metrics();

        assert!(
            output.contains("scmscx_telemetry_test_shared_total"),
            "{output}"
        );
        assert!(output.contains("scmscx_telemetry_test_gauge"), "{output}");
        assert!(
            output.contains("scmscx_telemetry_test_histogram"),
            "{output}"
        );
        // Both increments land on the one shared series.
        assert!(
            output.contains("scmscx_telemetry_test_shared_total{kind=\"x\"} 2"),
            "{output}"
        );
        assert!(output.contains("# EOF"), "{output}");
    }
}
