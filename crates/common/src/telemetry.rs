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
/// `register_counter!("scmscx", http_requests, "help", method = m, status = s).inc();`
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
    let panics = register_counter!("scmscx", panics, "Total panics observed by the panic hook.");
    std::panic::set_hook(Box::new(move |info| {
        panics.inc();
        previous(info);
    }));
}

/// Interval between runtime-metric samples.
const RUNTIME_SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

/// Spawn a background task that periodically publishes process-level and tokio
/// runtime gauges. Must be called from within a tokio runtime.
///
/// With `--cfg tokio_unstable` (set in `.cargo/config.toml`) this reports the
/// full `tokio-metrics` `RuntimeMonitor` interval stats — worker busy time, task
/// polls, work-stealing, queue depths, blocking-pool state, etc. Without the cfg
/// it degrades to the small stable subset from `Handle::metrics()`.
///
/// Note: it monitors *the runtime it is spawned on*. In `bwrender` that is the
/// whole pipeline; in the actix web server that is the main/background runtime
/// (pumpers, reporters, scrape server) — actix serves requests on its own
/// per-worker runtimes, whose latency is already captured by the HTTP middleware
/// histogram.
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

    // Expose tokio's task poll-time distribution as a native Prometheus histogram.
    // It is a custom collector rather than a sampled gauge so the buckets are read
    // straight from the runtime at scrape time (see `PollTimeCollector`).
    #[cfg(tokio_unstable)]
    REGISTRY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .register_collector(Box::new(PollTimeCollector {
            handle: handle.clone(),
        }));

    tokio::spawn(async move {
        // The interval iterator is stateful (each `next()` yields the delta since
        // the previous sample), so it lives across loop iterations.
        #[cfg(tokio_unstable)]
        let monitor = tokio_metrics::RuntimeMonitor::new(&handle);
        #[cfg(tokio_unstable)]
        let mut intervals = monitor.intervals();

        loop {
            register_gauge!(
                "scmscx",
                process_uptime_seconds,
                "Seconds since this process started"
            )
            .set(start.elapsed().as_secs() as i64);

            #[cfg(tokio_unstable)]
            if let Some(interval) = intervals.next() {
                record_runtime_interval(&interval);
            }

            #[cfg(not(tokio_unstable))]
            {
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
            }

            tokio::time::sleep(RUNTIME_SAMPLE_INTERVAL).await;
        }
    });
}

/// Publish one `tokio-metrics` runtime interval. Per-interval totals are recorded
/// as counters (incremented by the interval's delta so they accumulate) and
/// point-in-time samples as gauges.
#[cfg(tokio_unstable)]
fn record_runtime_interval(iv: &tokio_metrics::RuntimeMetrics) {
    fn micros(d: std::time::Duration) -> u64 {
        d.as_micros().min(u128::from(u64::MAX)) as u64
    }

    // Accumulating totals (interval deltas).
    register_counter!("scmscx", tokio_park, "Total worker park (idle) events")
        .inc_by(iv.total_park_count);
    register_counter!(
        "scmscx",
        tokio_busy_duration_micros,
        "Total time worker threads were busy, in microseconds"
    )
    .inc_by(micros(iv.total_busy_duration));
    register_counter!("scmscx", tokio_polls, "Total number of task polls")
        .inc_by(iv.total_polls_count);
    register_counter!("scmscx", tokio_noop, "Total no-op park wakeups").inc_by(iv.total_noop_count);
    register_counter!(
        "scmscx",
        tokio_steal_count,
        "Total tasks stolen between workers"
    )
    .inc_by(iv.total_steal_count);
    register_counter!(
        "scmscx",
        tokio_steal_operations,
        "Total work-stealing operations"
    )
    .inc_by(iv.total_steal_operations);
    register_counter!(
        "scmscx",
        tokio_local_schedule,
        "Total tasks scheduled onto a worker's local queue"
    )
    .inc_by(iv.total_local_schedule_count);
    register_counter!(
        "scmscx",
        tokio_remote_schedule,
        "Total tasks scheduled from outside the runtime"
    )
    .inc_by(iv.num_remote_schedules);
    register_counter!(
        "scmscx",
        tokio_overflow,
        "Total times a local queue overflowed into the global queue"
    )
    .inc_by(iv.total_overflow_count);
    register_counter!(
        "scmscx",
        tokio_budget_forced_yield,
        "Total tasks forced to yield because they exhausted their coop budget"
    )
    .inc_by(iv.budget_forced_yield_count);
    register_counter!(
        "scmscx",
        tokio_io_driver_ready,
        "Total readiness events processed by the IO driver"
    )
    .inc_by(iv.io_driver_ready_count);

    // Point-in-time samples.
    register_gauge!(
        "scmscx",
        tokio_workers,
        "Number of tokio runtime worker threads"
    )
    .set(iv.workers_count as i64);
    register_gauge!(
        "scmscx",
        tokio_global_queue_depth,
        "Number of tasks in the tokio global run queue"
    )
    .set(iv.global_queue_depth as i64);
    register_gauge!(
        "scmscx",
        tokio_local_queue_depth,
        "Total tasks sitting in per-worker local run queues"
    )
    .set(iv.total_local_queue_depth as i64);
    register_gauge!(
        "scmscx",
        tokio_blocking_queue_depth,
        "Tasks queued waiting for a blocking (spawn_blocking) thread"
    )
    .set(iv.blocking_queue_depth as i64);
    register_gauge!(
        "scmscx",
        tokio_live_tasks,
        "Number of alive tasks on the runtime"
    )
    .set(iv.live_tasks_count as i64);
    register_gauge!(
        "scmscx",
        tokio_blocking_threads,
        "Number of threads in the blocking pool"
    )
    .set(iv.blocking_threads_count as i64);
    register_gauge!(
        "scmscx",
        tokio_idle_blocking_threads,
        "Number of idle threads in the blocking pool"
    )
    .set(iv.idle_blocking_threads_count as i64);
    register_gauge!(
        "scmscx",
        tokio_interval_elapsed_micros,
        "Wall-clock duration covered by this metrics interval, in microseconds"
    )
    .set(micros(iv.elapsed) as i64);

    // Mean task poll duration. Populated only when the runtime was built with the
    // poll-time histogram enabled (see the binaries' runtime builders); zero
    // otherwise. This is the headline "are tasks blocking the executor?" signal.
    register_gauge!(
        "scmscx",
        tokio_mean_poll_duration_micros,
        "Mean task poll duration across the runtime, in microseconds"
    )
    .set(micros(iv.mean_poll_duration) as i64);
    register_gauge!(
        "scmscx",
        tokio_mean_poll_duration_worker_min_micros,
        "Smallest per-worker mean poll duration, in microseconds"
    )
    .set(micros(iv.mean_poll_duration_worker_min) as i64);
    register_gauge!(
        "scmscx",
        tokio_mean_poll_duration_worker_max_micros,
        "Largest per-worker mean poll duration, in microseconds"
    )
    .set(micros(iv.mean_poll_duration_worker_max) as i64);
}

/// Custom collector that exports tokio's per-worker task poll-time histogram as a
/// native Prometheus histogram (`scmscx_tokio_poll_time_seconds`) at scrape time.
///
/// tokio keeps cumulative per-bucket poll counts (since runtime start) per worker;
/// we sum them across workers and hand prometheus-client the per-bucket counts,
/// which it accumulates into the `le` series. This yields real
/// `histogram_quantile()`-able p50/p99 poll latencies — the signal a *mean* hides.
///
/// `_sum` is approximate: tokio does not expose the exact sum of poll durations,
/// so it is estimated from each bucket's lower bound. Quantiles (bucket-based) are
/// exact; only the derived average is an estimate.
#[cfg(tokio_unstable)]
#[derive(Debug)]
struct PollTimeCollector {
    handle: tokio::runtime::Handle,
}

#[cfg(tokio_unstable)]
impl prometheus_client::collector::Collector for PollTimeCollector {
    fn encode(
        &self,
        mut encoder: prometheus_client::encoding::DescriptorEncoder,
    ) -> Result<(), std::fmt::Error> {
        let m = self.handle.metrics();
        if !m.poll_time_histogram_enabled() {
            return Ok(());
        }

        let num_buckets = m.poll_time_histogram_num_buckets();
        let num_workers = m.num_workers();

        let mut buckets: Vec<(f64, u64)> = Vec::with_capacity(num_buckets);
        let mut total: u64 = 0;
        let mut sum_seconds: f64 = 0.0;
        for b in 0..num_buckets {
            let count: u64 = (0..num_workers)
                .map(|w| m.poll_time_histogram_bucket_count(w, b))
                .sum();
            let range = m.poll_time_histogram_bucket_range(b);
            // The final bucket is the open-ended overflow; `f64::MAX` is the
            // encoder's sentinel for the `le="+Inf"` label.
            let upper = if b == num_buckets - 1 {
                f64::MAX
            } else {
                range.end.as_secs_f64()
            };
            buckets.push((upper, count));
            total += count;
            sum_seconds += count as f64 * range.start.as_secs_f64();
        }

        let mut metric_encoder = encoder.encode_descriptor(
            "scmscx_tokio_poll_time_seconds",
            "Distribution of tokio task poll durations, in seconds",
            None,
            prometheus_client::metrics::MetricType::Histogram,
        )?;
        metric_encoder.encode_histogram::<Labels>(sum_seconds, total, &buckets, None)?;
        Ok(())
    }
}

/// Build the tokio runtime the binary should run on. Equivalent to
/// `#[tokio::main]` (multi-threaded, all drivers enabled) plus, under
/// `--cfg tokio_unstable`, the poll-time histogram, which is what populates the
/// `scmscx_tokio_mean_poll_duration_*` metrics. Call `.block_on(async { … })` on
/// the returned runtime.
pub fn build_runtime() -> std::io::Result<tokio::runtime::Runtime> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    #[cfg(tokio_unstable)]
    {
        // Log-scale poll-time histogram spanning ~1µs–10s. A rare
        // several-hundred-ms blocking poll then lands in its own bucket; the
        // default (linear, low-µs) config buckets everything above ~1ms together
        // and would hide exactly the stalls we care about. `precision_exact(1)`
        // gives two sub-buckets per octave (~46 buckets over the range) — enough
        // resolution for quantiles without excessive series cardinality.
        let histogram = tokio::runtime::LogHistogram::builder()
            .min_value(std::time::Duration::from_micros(1))
            .max_value(std::time::Duration::from_secs(10))
            .precision_exact(1)
            .build();
        builder.metrics_poll_time_histogram_configuration(
            tokio::runtime::HistogramConfiguration::log(histogram),
        );
        builder.enable_metrics_poll_time_histogram();
    }
    builder.build()
}

/// One-call telemetry bootstrap for a binary. Installs the panic hook, starts
/// the Prometheus scrape server on the address in `PROMETHEUS_ENDPOINT`
/// (required — panics if unset or unparseable, by design), and spawns the
/// runtime/process gauge reporter tagged with `version`. Must be called from
/// within a tokio runtime.
pub async fn init(version: &'static str) {
    install_panic_hook();
    // Prefer an inherited, already-bound listener (`PROMETHEUS_FD`) over binding
    // an address (`PROMETHEUS_ENDPOINT`). The E2E harness hands the socket down
    // this way so the scrape port is chosen and held race-free — see
    // [`take_listener_from_env`].
    if let Some(std_listener) = take_listener_from_env("PROMETHEUS_FD") {
        std_listener
            .set_nonblocking(true)
            .expect("set inherited prometheus listener non-blocking");
        let listener = tokio::net::TcpListener::from_std(std_listener)
            .expect("adopt inherited prometheus listener");
        tracing::info!("prometheus metrics server listening on inherited fd");
        serve_prometheus(listener);
    } else {
        let endpoint: std::net::SocketAddr = std::env::var("PROMETHEUS_ENDPOINT")
            .expect("PROMETHEUS_ENDPOINT not set")
            .parse()
            .expect("PROMETHEUS_ENDPOINT must be a socket address, e.g. 0.0.0.0:9101");
        init_prometheus_server(endpoint).await;
    }
    spawn_runtime_metrics_reporter(version);
}

/// Adopt an already-bound, listening TCP socket from the file-descriptor number
/// held in `env_var` (if set and parseable); otherwise `None`, and the caller
/// binds an address itself.
///
/// This lets a parent process pick a port, bind it, and hand the *live* socket
/// down to this process — the port is held continuously, eliminating the classic
/// "grab an ephemeral port, close it, then race to re-bind it" flaw. The E2E
/// harness uses it for both the HTTP server and this scrape server; it is also
/// the shape systemd / `systemfd` socket activation uses.
///
/// SAFETY: the parent's contract is that the fd is an owned, listening socket
/// handed off for our exclusive use; we take sole ownership of it here.
pub fn take_listener_from_env(env_var: &str) -> Option<std::net::TcpListener> {
    use std::os::fd::FromRawFd;
    let fd: std::os::fd::RawFd = std::env::var(env_var).ok()?.trim().parse().ok()?;
    Some(unsafe { std::net::TcpListener::from_raw_fd(fd) })
}

/// Start the Prometheus scrape server on `addr`, serving `/metrics` (and a
/// trivial `/health`). Runs in a spawned task; a bind failure is logged but does
/// not bring the service down — metrics are best-effort.
pub async fn init_prometheus_server(addr: std::net::SocketAddr) {
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("failed to bind prometheus metrics server on {addr}: {e}");
            return;
        }
    };

    tracing::info!("prometheus metrics server listening on http://{addr}/metrics");

    serve_prometheus(listener);
}

/// Spawn the Prometheus scrape server (`/metrics`, plus a trivial `/health`) on
/// an already-bound listener, in a background task. Serving is best-effort.
fn serve_prometheus(listener: tokio::net::TcpListener) {
    use axum::routing::get;

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
