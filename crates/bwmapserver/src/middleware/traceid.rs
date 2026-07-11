use std::{
    future::{ready, Ready},
    time::Instant,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::{future::LocalBoxFuture, FutureExt};
use tracing::{error, info, instrument, warn, Instrument};

#[derive(Clone, Debug)]
pub struct TraceID {
    pub id: String,
    pub start_time: Instant,
}

pub struct TraceIDTransformer;

impl<S, B> Transform<S, ServiceRequest> for TraceIDTransformer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TracewIDMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TracewIDMiddleware { service }))
    }
}

pub struct TracewIDMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TracewIDMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    #[instrument(skip_all, name = "traceid-middleware", fields(trace_id))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_owned();

        let trace_id: String = uuid::Uuid::new_v4()
            .as_simple()
            .to_string()
            .chars()
            .take(6)
            .collect();

        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("x.x.x.x")
            .to_owned();

        let user_agent = req.headers().get("user-agent").map_or_else(
            || "couldn't unwrap2".to_string(),
            |x| x.to_str().unwrap_or("couldn't unwrap").to_owned(),
        );

        req.extensions_mut().insert(TraceID {
            id: trace_id.clone(),
            start_time: Instant::now(),
        });

        tracing::Span::current().record("trace_id", trace_id.as_str());
        let fut = self.service.call(req);

        async move {
            match fut.await {
                Ok(x) => {
                    if x.status().is_success() {
                        info!(status=%x.status(), %path, %ip, %user_agent);
                    } else if x.status().is_redirection() {
                        info!(status=%x.status(), %path, %ip, %user_agent);
                    } else if x.status().is_client_error() {
                        warn!(status=%x.status(), %path, %ip, %user_agent);
                    } else if x.status().is_server_error() {
                        error!(status=%x.status(), %path, %ip, %user_agent);
                    } else {
                        warn!(status=%x.status(), %path, %ip, %user_agent);
                    }
                    Ok(x)
                }
                Err(x) => {
                    error!(%path, %ip, %user_agent, err=?x);
                    Err(x)
                }
            }
        }
        .instrument(tracing::Span::current())
        .boxed_local()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpMessage, HttpRequest, HttpResponse};

    /// Reports the trace id the middleware assigned (or "none").
    async fn echo_trace(req: HttpRequest) -> HttpResponse {
        let id = req.extensions().get::<TraceID>().map(|t| t.id.clone());
        HttpResponse::Ok().body(id.unwrap_or_else(|| "none".to_string()))
    }

    async fn trace_id_body() -> String {
        let app = test::init_service(
            App::new()
                .wrap(TraceIDTransformer)
                .default_service(web::to(echo_trace)),
        )
        .await;
        let resp = test::call_service(&app, test::TestRequest::get().to_request()).await;
        String::from_utf8(test::read_body(resp).await.to_vec()).unwrap()
    }

    #[actix_web::test]
    async fn assigns_six_char_trace_id_visible_to_handler() {
        let id = trace_id_body().await;
        assert_eq!(id.len(), 6, "trace id is truncated to 6 chars, got {id:?}");
        assert!(
            id.chars().all(|c| c.is_ascii_hexdigit()),
            "trace id should be hex, got {id:?}"
        );
    }

    #[actix_web::test]
    async fn trace_ids_are_unique_per_request() {
        assert_ne!(
            trace_id_body().await,
            trace_id_body().await,
            "each request must get a fresh trace id"
        );
    }

    #[actix_web::test]
    async fn preserves_downstream_status() {
        let app = test::init_service(
            App::new()
                .wrap(TraceIDTransformer)
                .default_service(web::to(|| async { HttpResponse::ImATeapot().finish() })),
        )
        .await;
        let resp = test::call_service(&app, test::TestRequest::get().to_request()).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::IM_A_TEAPOT);
    }
}
