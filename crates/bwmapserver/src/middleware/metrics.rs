//! Actix middleware that records Prometheus metrics for every HTTP request:
//! total count (by method / route / status), latency histogram, and an
//! in-flight gauge.
//!
//! The route label uses actix's *matched pattern* (e.g. `/api/maps/{hash}`)
//! rather than the raw path, keeping cardinality bounded. Requests that match no
//! route (static files, the dev proxy, 404s) are bucketed under `"<other>"`.

use std::{
    future::{ready, Ready},
    time::Instant,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use common::{register_counter, register_gauge, register_histogram};
use futures_util::{future::LocalBoxFuture, FutureExt};

pub struct MetricsTransformer;

impl<S, B> Transform<S, ServiceRequest> for MetricsTransformer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddleware { service }))
    }
}

pub struct MetricsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().to_string();
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

        let fut = self.service.call(req);

        async move {
            // Increment and build the guard *inside* the future so it is owned by
            // it: `async move` only captures names used in the body, so a guard
            // bound outside the block would be dropped when `call` returns rather
            // than when the request completes. The guard's Drop decrements even if
            // the future is cancelled.
            in_flight.inc();
            let _guard = InFlightGuard(in_flight);

            let res = fut.await;
            let elapsed = start.elapsed().as_secs_f64();

            // Resolve the route label once. Successful responses carry the
            // matched route pattern (falling back to "<other>" for unmatched
            // paths); an error short-circuits before a route is matched, so it
            // is bucketed under "<error>" with the error's status.
            let (route, status) = match &res {
                Ok(res) => (
                    res.request()
                        .match_pattern()
                        .unwrap_or_else(|| "<other>".to_string()),
                    res.status().as_u16(),
                ),
                Err(err) => (
                    "<error>".to_string(),
                    err.as_response_error().status_code().as_u16(),
                ),
            };

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
        .boxed_local()
    }
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
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn records_series_labelled_by_matched_route() {
        // A route pattern unique to this test keeps the assertion robust against
        // metrics written by other tests sharing the process-global registry.
        let app = test::init_service(
            App::new()
                .wrap(MetricsTransformer)
                .route("/metrics_probe_route", web::get().to(|| async { "ok" })),
        )
        .await;

        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/metrics_probe_route")
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let scraped = common::telemetry::encode_metrics();
        assert!(
            scraped.contains("route=\"/metrics_probe_route\""),
            "expected the request counter to be labelled with the matched route"
        );
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

    #[actix_web::test]
    async fn in_flight_gauge_returns_to_baseline_after_requests() {
        // The InFlightGuard's Drop decrements the gauge when each request's future
        // completes. Fire a batch through the middleware and assert the gauge does
        // not climb: every `inc()` must be paired with the guard's `dec()`. Under a
        // mutant that drops the decrement, each completed request leaks +1 and the
        // gauge grows by the batch size. The gauge is process-global (shared with
        // the sibling metrics tests), so measure the delta around our own batch and
        // tolerate a little concurrent noise rather than asserting an absolute.
        let app = test::init_service(
            App::new()
                .wrap(MetricsTransformer)
                .route("/in_flight_probe", web::get().to(|| async { "ok" })),
        )
        .await;

        const N: i64 = 20;
        let before = in_flight_gauge();
        for _ in 0..N {
            let resp = test::call_service(
                &app,
                test::TestRequest::get()
                    .uri("/in_flight_probe")
                    .to_request(),
            )
            .await;
            assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
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

    #[actix_web::test]
    async fn unmatched_requests_bucket_under_other() {
        // No route matches → MatchedPattern absent → "<other>" bucket, keeping
        // label cardinality bounded.
        let app = test::init_service(
            App::new()
                .wrap(MetricsTransformer)
                .route("/known", web::get().to(|| async { "ok" }))
                .default_service(web::to(HttpResponse::NotFound)),
        )
        .await;

        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/this/path/does/not/match")
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);

        assert!(
            common::telemetry::encode_metrics().contains("route=\"<other>\""),
            "unmatched requests should be bucketed under <other>"
        );
    }
}
