use axum::extract::Request;
use axum::http::header::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;

/// Adds a long immutable `Cache-Control` to static asset responses (js/wasm/
/// webm/css), except for the dev-mode `css.css` / `lib.js` bundles.
pub async fn cache_html(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();

    let mut res = next.run(req).await;

    let mut should_cache = false;
    if let Some(content_type) = res.headers().get(HeaderName::from_static("content-type")) {
        if let Ok(s) = content_type.to_str() {
            should_cache = s.contains("application/javascript")
                || s.contains("application/wasm")
                || s.contains("video/webm")
                || s.contains("text/css");
        }
    }

    if path.contains("css.css") || path.contains("lib.js") {
        should_cache = false;
    }

    if should_cache {
        res.headers_mut().append(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::response::IntoResponse;
    use axum::Router;
    use tower::ServiceExt;

    /// Fallback handler that echoes the caller-supplied content-type header
    /// (`x-test-ct`) as the response's Content-Type.
    async fn handler(req: Request) -> Response {
        let ct = req
            .headers()
            .get("x-test-ct")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/plain")
            .to_string();
        ([(axum::http::header::CONTENT_TYPE, ct)], "body".to_string()).into_response()
    }

    async fn run(path: &str, content_type: &str) -> Response {
        let app = Router::new()
            .fallback(handler)
            .layer(axum::middleware::from_fn(cache_html));
        let req = axum::http::Request::builder()
            .uri(path)
            .header("x-test-ct", content_type)
            .body(Body::empty())
            .unwrap();
        app.oneshot(req).await.unwrap()
    }

    fn is_cached(res: &Response) -> bool {
        res.headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.contains("immutable"))
    }

    #[tokio::test]
    async fn caches_static_asset_content_types() {
        for ct in [
            "application/javascript",
            "application/wasm",
            "video/webm",
            "text/css",
        ] {
            let res = run("/assets/app.hash.js", ct).await;
            assert!(is_cached(&res), "content-type {ct} should be cached");
        }
    }

    #[tokio::test]
    async fn caches_content_type_with_charset_suffix() {
        // The match is substring-based, so a `; charset=` suffix still caches.
        let res = run("/assets/app.js", "application/javascript; charset=utf-8").await;
        assert!(is_cached(&res));
    }

    #[tokio::test]
    async fn does_not_cache_html_or_other_types() {
        for ct in ["text/html", "application/json", "image/png"] {
            let res = run("/index.html", ct).await;
            assert!(!is_cached(&res), "content-type {ct} should NOT be cached");
        }
    }

    #[tokio::test]
    async fn dev_bundles_are_never_cached() {
        // css.css and lib.js are the hot-reloaded dev bundles; even with a
        // cacheable content-type they must stay uncached.
        let res = run("/dist/css.css", "text/css").await;
        assert!(!is_cached(&res), "css.css must not be cached");

        let res = run("/dist/lib.js", "application/javascript").await;
        assert!(!is_cached(&res), "lib.js must not be cached");
    }
}
