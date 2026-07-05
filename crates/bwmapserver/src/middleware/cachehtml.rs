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
