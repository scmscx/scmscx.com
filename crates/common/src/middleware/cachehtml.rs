use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
    Error,
};
use futures_util::{future::LocalBoxFuture, FutureExt};

pub struct CacheHtmlTransformer;

impl<S, B> Transform<S, ServiceRequest> for CacheHtmlTransformer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CacheHtmlMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CacheHtmlMiddleware { service }))
    }
}

pub struct CacheHtmlMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CacheHtmlMiddleware<S>
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
        let fut = self.service.call(req);

        async move {
            let mut res = fut.await?;

            let mut should_cache = false;

            // if let Some(content_type) = res
            //     .response()
            //     .headers()
            //     .get(HeaderName::from_static("content-type"))
            // {
            //     if let Ok(s) = content_type.to_str() {
            //         should_cache = s.contains("application/javascript") || s.contains("text/css")
            //     }
            // }

            if res.request().path().starts_with("/static/") {
                should_cache = true;
            }

            if should_cache {
                let headers = res.headers_mut();
                headers.append(
                    HeaderName::from_static("cache-control"),
                    HeaderValue::from_static("public, max-age=31536000, immutable"),
                );
            }

            Ok(res)
        }
        .boxed_local()
    }
}
