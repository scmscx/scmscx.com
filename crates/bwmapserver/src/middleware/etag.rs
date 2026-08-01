use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};

/// Adds a weak `ETag` to server-rendered HTML pages and answers conditional
/// requests: a matching `If-None-Match` yields `304 Not Modified` with no body, so
/// an unchanged page (same deploy, same underlying data) isn't re-sent. The pages
/// stay dynamic — `Cache-Control: no-cache` makes the browser revalidate on every
/// visit — but revalidation costs a small 304 instead of the full document.
///
/// Only `text/html` responses are touched; everything else (hashed assets, images,
/// JSON, streamed downloads) passes straight through, unbuffered. The ETag is a
/// hash of the rendered bytes, so it changes exactly when the page does: a new
/// deploy (fresh asset hashes in the manifest) or different per-URL data.
pub async fn etag(req: Request, next: Next) -> Response {
    // Read the request validator before the body is consumed downstream.
    let if_none_match = req
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let res = next.run(req).await;

    if !super::response_is_html(&res) {
        return res;
    }

    let (mut parts, body) = res.into_parts();

    // Server-rendered pages are small, fully-buffered strings; collecting is cheap.
    let Ok(bytes) = axum::body::to_bytes(body, 16 * 1024 * 1024).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let tag = weak_etag(&bytes);
    parts.headers.insert(
        header::ETAG,
        HeaderValue::from_str(&tag).expect("etag is ascii"),
    );
    // Make the browser revalidate every time so it actually sends If-None-Match;
    // don't override a handler that already declared its own policy.
    if !parts.headers.contains_key(header::CACHE_CONTROL) {
        parts
            .headers
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    }

    if if_none_match
        .as_deref()
        .is_some_and(|inm| if_none_match_matches(inm, &tag))
    {
        parts.status = StatusCode::NOT_MODIFIED;
        parts.headers.remove(header::CONTENT_TYPE);
        parts.headers.remove(header::CONTENT_LENGTH);
        return (parts, Body::empty()).into_response();
    }

    (parts, Body::from(bytes)).into_response()
}

/// A weak validator over the response bytes. 128 bits of SHA-256 is ample to tell
/// one page revision from another; weak (`W/`) because a downstream content-coding
/// (gzip/br) leaves the entity semantically equivalent but not byte-identical.
fn weak_etag(bytes: &Bytes) -> String {
    use std::fmt::Write;
    let digest = Sha256::digest(bytes);
    let mut tag = String::from("W/\"");
    for b in &digest[..16] {
        let _ = write!(tag, "{b:02x}");
    }
    tag.push('"');
    tag
}

/// RFC 7232 weak comparison against a possibly-multi-valued `If-None-Match`:
/// `*` matches anything, otherwise the opaque-tags must be equal ignoring any
/// `W/` weakness prefix.
fn if_none_match_matches(header_value: &str, our_tag: &str) -> bool {
    let ours = opaque_tag(our_tag);
    header_value
        .split(',')
        .map(str::trim)
        .any(|candidate| candidate == "*" || opaque_tag(candidate) == ours)
}

fn opaque_tag(tag: &str) -> &str {
    tag.trim().strip_prefix("W/").unwrap_or(tag).trim()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route(
                "/",
                get(|| async { ([(header::CONTENT_TYPE, "text/html")], "<h1>hi</h1>") }),
            )
            .route(
                "/json",
                get(|| async { ([(header::CONTENT_TYPE, "application/json")], "{}") }),
            )
            .route(
                "/fixed",
                get(|| async {
                    (
                        [
                            (header::CONTENT_TYPE, "text/html"),
                            (header::CACHE_CONTROL, "max-age=5"),
                        ],
                        "<h1>hi</h1>",
                    )
                }),
            )
            .layer(axum::middleware::from_fn(etag))
    }

    async fn get_path(path: &str, if_none_match: Option<&str>) -> Response {
        let mut builder = axum::http::Request::builder().uri(path);
        if let Some(inm) = if_none_match {
            builder = builder.header(header::IF_NONE_MATCH, inm);
        }
        app()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    fn etag_of(res: &Response) -> String {
        res.headers()
            .get(header::ETAG)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned()
    }

    #[tokio::test]
    async fn html_gets_a_weak_etag_and_no_cache() {
        let res = get_path("/", None).await;
        assert_eq!(res.status(), StatusCode::OK);
        let tag = etag_of(&res);
        assert!(tag.starts_with("W/\""), "expected a weak etag, got {tag:?}");
        assert_eq!(
            res.headers()
                .get(header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("no-cache"),
        );
    }

    #[tokio::test]
    async fn matching_if_none_match_yields_304_with_no_body() {
        let tag = etag_of(&get_path("/", None).await);
        let res = get_path("/", Some(&tag)).await;
        assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(body.is_empty(), "304 must have no body");
    }

    #[tokio::test]
    async fn stale_if_none_match_yields_200_with_body() {
        let res = get_path("/", Some("W/\"deadbeef\"")).await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"<h1>hi</h1>");
    }

    #[tokio::test]
    async fn star_if_none_match_yields_304() {
        assert_eq!(
            get_path("/", Some("*")).await.status(),
            StatusCode::NOT_MODIFIED
        );
    }

    #[tokio::test]
    async fn non_html_is_untouched() {
        let res = get_path("/json", None).await;
        assert!(
            res.headers().get(header::ETAG).is_none(),
            "non-html must not get an ETag"
        );
        assert!(res.headers().get(header::CACHE_CONTROL).is_none());
    }

    #[tokio::test]
    async fn does_not_override_a_handlers_cache_control() {
        let res = get_path("/fixed", None).await;
        assert_eq!(
            res.headers()
                .get(header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("max-age=5"),
            "a handler's own Cache-Control must win",
        );
        assert!(res.headers().get(header::ETAG).is_some());
    }
}
