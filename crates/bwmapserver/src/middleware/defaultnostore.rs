use axum::extract::Request;
use axum::http::header;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Stamps `Cache-Control: no-store` on any response that didn't set one, so the
/// origin never emits a header-less response. This makes edge caching "default
/// deny": anything meant to be cached opts in explicitly (`immutable` assets,
/// `public` minimaps/images, the `no-cache` on SSR HTML), and everything else —
/// dynamic JSON, per-session endpoints like `is_session_valid`, redirects, errors
/// — is `no-store` rather than left to a CDN's guesswork.
///
/// This is what makes a "respect origin, cache everything eligible" Cloudflare rule
/// safe: with every response carrying an explicit directive, there is no "no header
/// → Cloudflare decides" case left.
///
/// Placed outermost so it observes the final `Cache-Control` set by the handler and
/// every inner layer (`immutable_assets`, `etag`), and only fills the gap.
pub async fn default_no_store(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;

    if !res.headers().contains_key(header::CACHE_CONTROL) {
        res.headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    async fn cache_control(res: Response) -> Option<String> {
        res.headers()
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    }

    async fn oneshot(handler: Router) -> Response {
        handler
            .layer(axum::middleware::from_fn(default_no_store))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn fills_in_no_store_when_absent() {
        // A handler that sets no Cache-Control gets no-store.
        let app = Router::new().route("/", get(|| async { "hi" }));
        assert_eq!(
            cache_control(oneshot(app).await).await.as_deref(),
            Some("no-store"),
        );
    }

    #[tokio::test]
    async fn leaves_an_explicit_cache_control_untouched() {
        // A handler that opted into caching keeps its own header — no clobbering.
        let app = Router::new().route(
            "/",
            get(|| async {
                (
                    [(header::CACHE_CONTROL, "public, max-age=31536000, immutable")],
                    "asset",
                )
            }),
        );
        assert_eq!(
            cache_control(oneshot(app).await).await.as_deref(),
            Some("public, max-age=31536000, immutable"),
        );
    }

    #[tokio::test]
    async fn does_not_downgrade_no_cache() {
        // The SSR-HTML `no-cache` (set by the etag layer) must survive, not become
        // no-store — the page should still revalidate, not be un-storable.
        let app = Router::new().route(
            "/",
            get(|| async { ([(header::CACHE_CONTROL, "no-cache")], "html").into_response() }),
        );
        assert_eq!(
            cache_control(oneshot(app).await).await.as_deref(),
            Some("no-cache"),
        );
    }
}
