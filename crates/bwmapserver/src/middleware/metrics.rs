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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    async fn drive(app: Router, uri: &str) -> Response {
        app.oneshot(
            axum::http::Request::builder()
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn records_series_labelled_by_matched_route() {
        // A route pattern unique to this test keeps the assertion robust against
        // metrics written by other tests sharing the process-global registry.
        let app = Router::new()
            .route("/metrics_probe_route", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(metrics));

        let res = drive(app, "/metrics_probe_route").await;
        assert_eq!(res.status(), StatusCode::OK);

        let scraped = common::telemetry::encode_metrics();
        assert!(
            scraped.contains("route=\"/metrics_probe_route\""),
            "expected the request counter to be labelled with the matched route"
        );
        // The duration histogram is registered for the same route.
        assert!(scraped.contains("scmscx_http_request_duration_seconds"));
        assert!(scraped.contains("scmscx_http_requests_in_flight"));
    }

    /// Read the current value of the label-less `scmscx_http_requests_in_flight`
    /// gauge out of a scrape. The value line is the metric name followed by the
    /// value (prometheus_client renders an empty label set with or without `{}`);
    /// the `# HELP`/`# TYPE` lines start with `#` and are skipped. The gauge is
    /// registered lazily on the first request through the middleware, so before any
    /// request it is simply absent — which is a value of 0 (nothing in flight).
    fn in_flight_gauge() -> i64 {
        let scraped = common::telemetry::encode_metrics();
        for line in scraped.lines() {
            if let Some(rest) = line.strip_prefix("scmscx_http_requests_in_flight") {
                let v = rest.trim_start_matches("{}").trim();
                if let Ok(n) = v.parse::<f64>() {
                    return n as i64;
                }
            }
        }
        0
    }

    #[tokio::test]
    async fn in_flight_gauge_returns_to_baseline_after_requests() {
        // The InFlightGuard's Drop decrements the gauge when each request's future
        // completes. Fire a batch through the middleware and assert the gauge does
        // not climb: every `inc()` must be paired with the guard's `dec()`. Under a
        // mutant that drops the decrement, each completed request leaks +1 and the
        // gauge grows by the batch size. The gauge is process-global (shared with
        // the sibling metrics tests), so measure the delta around our own batch and
        // tolerate a little concurrent noise rather than asserting an absolute.
        let app = Router::new()
            .route("/in_flight_probe", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(metrics));

        const N: i64 = 20;
        let before = in_flight_gauge();
        for _ in 0..N {
            let res = drive(app.clone(), "/in_flight_probe").await;
            assert_eq!(res.status(), StatusCode::OK);
        }
        let after = in_flight_gauge();

        // Correct code nets zero (each +1 matched by the guard's -1), so
        // `after == before`. The mutant leaves `after == before + N`. A half-batch
        // margin absorbs a sibling test having a request in flight across a read.
        assert!(
            after - before < N / 2,
            "in-flight gauge grew by {} over {N} requests (before={before}, after={after}); \
             the InFlightGuard decrement did not run",
            after - before,
        );
    }

    #[tokio::test]
    async fn unmatched_requests_bucket_under_other() {
        // No route matches → fallback → MatchedPath absent → "<other>" bucket,
        // keeping label cardinality bounded.
        let app = Router::new()
            .route("/known", get(|| async { "ok" }))
            .fallback(|| async { StatusCode::NOT_FOUND })
            .layer(axum::middleware::from_fn(metrics));

        let res = drive(app, "/this/path/does/not/match").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        assert!(
            common::telemetry::encode_metrics().contains("route=\"<other>\""),
            "unmatched requests should be bucketed under <other>"
        );
    }

    #[tokio::test]
    async fn preserves_downstream_status() {
        let app = Router::new()
            .route("/", get(|| async { StatusCode::IM_A_TEAPOT }))
            .layer(axum::middleware::from_fn(metrics));
        let res = drive(app, "/").await;
        assert_eq!(res.status(), StatusCode::IM_A_TEAPOT);
    }
}
