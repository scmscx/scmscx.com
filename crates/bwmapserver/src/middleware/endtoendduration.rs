//! Outermost axum middleware recording the full server-side cost of a request.
//!
//! `scmscx_http_request_duration_seconds` (see [`super::metrics`]) is registered
//! part-way down the stack, so it times the handler, compression and the ETag
//! layer — but *not* `user_session`, `postgres_logging`, `tracking_analytics`,
//! `language`, `trace_id` or `default_no_store`, all of which sit outside it and
//! several of which touch the database. Latency added there is real, user-facing
//! and completely absent from that histogram.
//!
//! This layer wraps everything, so subtracting the two answers "is the time in the
//! handler, or in the middleware around it?":
//!
//! ```promql
//! sum(rate(scmscx_http_request_end_to_end_duration_seconds_sum[5m]))
//!   - sum(rate(scmscx_http_request_duration_seconds_sum[5m]))
//! ```
//!
//! Both use the same `method`/`route` labels so they subtract cleanly per route.
//! Keeping them as two series rather than moving the existing one preserves the
//! continuity of a histogram that already has history behind it.

use std::time::Instant;

use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;
use common::register_histogram;

pub async fn end_to_end_duration(req: Request, next: Next) -> Response {
    let method = req.method().to_string();
    // Same bounded-cardinality route label as the inner histogram, so the two are
    // directly comparable; unmatched requests share the `<other>` bucket.
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "<other>".to_string(), |m| m.as_str().to_string());
    let start = Instant::now();

    let res = next.run(req).await;

    register_histogram!(
        "scmscx",
        http_request_end_to_end_duration_seconds,
        "Full server-side HTTP latency including every middleware layer, by method and route pattern",
        common::telemetry::latency_buckets(),
        method = method,
        route = route
    )
    .observe(start.elapsed().as_secs_f64());

    res
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
            .route("/e2e_probe_route", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(end_to_end_duration));

        let res = drive(app, "/e2e_probe_route").await;
        assert_eq!(res.status(), StatusCode::OK);

        let scraped = common::telemetry::encode_metrics();
        assert!(
            scraped.contains("scmscx_http_request_end_to_end_duration_seconds"),
            "the end-to-end histogram should be registered on first request",
        );
        assert!(
            scraped.contains("route=\"/e2e_probe_route\""),
            "expected the histogram to be labelled with the matched route",
        );
    }

    #[tokio::test]
    async fn unmatched_requests_bucket_under_other() {
        // No route matches → fallback → MatchedPath absent → "<other>", matching
        // how `metrics` labels the same request so the two series subtract.
        let app = Router::new()
            .route("/known", get(|| async { "ok" }))
            .fallback(|| async { StatusCode::NOT_FOUND })
            .layer(axum::middleware::from_fn(end_to_end_duration));

        let res = drive(app, "/no/such/path").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        assert!(
            common::telemetry::encode_metrics().contains("route=\"<other>\""),
            "unmatched requests should be bucketed under <other>",
        );
    }

    #[tokio::test]
    async fn observes_time_spent_in_inner_layers() {
        // The point of this layer is that it sees latency the inner `metrics`
        // histogram cannot. Sleep inside a nested middleware — i.e. between this
        // layer and the handler — and assert the observation lands in a bucket
        // above the sleep, not in the sub-millisecond ones. A mutant that starts
        // the timer after `next.run` (or drops the observation) records ~0.
        async fn slow(req: Request, next: Next) -> Response {
            tokio::time::sleep(std::time::Duration::from_millis(60)).await;
            next.run(req).await
        }

        let app = Router::new()
            .route("/e2e_slow_probe", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(slow))
            .layer(axum::middleware::from_fn(end_to_end_duration));

        assert_eq!(drive(app, "/e2e_slow_probe").await.status(), StatusCode::OK);

        // Find this route's cumulative bucket counts and assert nothing landed
        // below the sleep duration: with 60ms of inner latency, the `le="0.05"`
        // bucket for this route must still be empty.
        let scraped = common::telemetry::encode_metrics();
        let under_50ms = scraped.lines().find(|l| {
            l.starts_with("scmscx_http_request_end_to_end_duration_seconds_bucket")
                && l.contains("route=\"/e2e_slow_probe\"")
                && l.contains("le=\"0.05\"")
        });
        let count = under_50ms
            .and_then(|l| l.rsplit(' ').next())
            .and_then(|v| v.trim().parse::<f64>().ok())
            .expect("expected an le=\"0.05\" bucket for this route");
        assert!(
            count < 1.0,
            "a 60ms request must not be counted under 50ms; the timer is not \
             spanning the inner layers (line: {under_50ms:?})",
        );
    }
}
