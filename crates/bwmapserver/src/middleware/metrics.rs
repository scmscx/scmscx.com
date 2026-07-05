//! axum middleware that records Prometheus metrics for every HTTP request:
//! total count (by method / route / status), latency histogram, and an
//! in-flight gauge.
//!
//! The route label uses axum's `MatchedPath` (e.g. `/api/maps/{hash}`) rather
//! than the raw path, keeping cardinality bounded. Requests that match no route
//! (static files, the dev proxy, 404s) are bucketed under `"<other>"`.

use std::time::Instant;

use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;
use common::{register_counter, register_gauge, register_histogram};

pub async fn metrics(req: Request, next: Next) -> Response {
    let method = req.method().to_string();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "<other>".to_string(), |m| m.as_str().to_string());
    let start = Instant::now();

    // The gauge is label-less, so cache the resolved handle in a OnceLock to
    // skip the family lookup on every request.
    static IN_FLIGHT: std::sync::OnceLock<prometheus_client::metrics::gauge::Gauge> =
        std::sync::OnceLock::new();
    let in_flight = IN_FLIGHT
        .get_or_init(|| {
            register_gauge!(
                "scmscx",
                http_requests_in_flight,
                "Number of HTTP requests currently being processed"
            )
        })
        .clone();

    in_flight.inc();
    let _guard = InFlightGuard(in_flight);

    let res = next.run(req).await;
    let elapsed = start.elapsed().as_secs_f64();
    let status = res.status().as_u16();

    register_counter!(
        "scmscx",
        http_requests,
        "Total HTTP requests handled, by method, route pattern and status",
        method = method,
        route = route,
        status = status
    )
    .inc();
    register_histogram!(
        "scmscx",
        http_request_duration_seconds,
        "HTTP request handling latency in seconds, by method and route pattern",
        common::telemetry::latency_buckets(),
        method = method,
        route = route
    )
    .observe(elapsed);

    // Drop of _guard here decrements the in-flight gauge.
    res
}

/// Decrements the in-flight gauge when dropped, so cancelled requests are counted
/// out correctly.
struct InFlightGuard(prometheus_client::metrics::gauge::Gauge);

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.0.dec();
    }
}
