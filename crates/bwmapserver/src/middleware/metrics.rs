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
