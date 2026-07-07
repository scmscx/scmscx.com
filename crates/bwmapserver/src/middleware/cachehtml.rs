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

            if let Some(content_type) = res
                .response()
                .headers()
                .get(HeaderName::from_static("content-type"))
            {
                if let Ok(s) = content_type.to_str() {
                    should_cache = s.contains("application/javascript")
                        || s.contains("application/wasm")
                        || s.contains("video/webm")
                        || s.contains("text/css");
                }
            }

            if res.request().path().contains("css.css") || res.request().path().contains("lib.js") {
                should_cache = false;
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::CONTENT_TYPE;
    use actix_web::{test, web, App, HttpRequest, HttpResponse};

    /// Handler that echoes the caller-supplied content-type (`x-test-ct`) as the
    /// response's Content-Type, so we can drive the cache decision per request.
    async fn echo_ct(req: HttpRequest) -> HttpResponse {
        let ct = req
            .headers()
            .get("x-test-ct")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/plain")
            .to_string();
        HttpResponse::Ok()
            .insert_header((CONTENT_TYPE, ct))
            .body("body")
    }

    async fn is_cached(path: &str, content_type: &str) -> bool {
        let app = test::init_service(
            App::new()
                .wrap(CacheHtmlTransformer)
                .default_service(web::to(echo_ct)),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(path)
            .insert_header(("x-test-ct", content_type))
            .to_request();
        let resp = test::call_service(&app, req).await;
        resp.headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.contains("immutable"))
    }

    #[actix_web::test]
    async fn caches_static_asset_content_types() {
        for ct in [
            "application/javascript",
            "application/wasm",
            "video/webm",
            "text/css",
        ] {
            assert!(
                is_cached("/assets/app.hash.js", ct).await,
                "content-type {ct} should be cached"
            );
        }
    }

    #[actix_web::test]
    async fn caches_content_type_with_charset_suffix() {
        assert!(is_cached("/assets/app.js", "application/javascript; charset=utf-8").await);
    }

    #[actix_web::test]
    async fn does_not_cache_html_or_other_types() {
        for ct in ["text/html", "application/json", "image/png"] {
            assert!(
                !is_cached("/index.html", ct).await,
                "content-type {ct} should NOT be cached"
            );
        }
    }

    #[actix_web::test]
    async fn dev_bundles_are_never_cached() {
        // css.css and lib.js are the hot-reloaded dev bundles; even with a
        // cacheable content-type they must stay uncached.
        assert!(!is_cached("/dist/css.css", "text/css").await);
        assert!(!is_cached("/dist/lib.js", "application/javascript").await);
    }
}
