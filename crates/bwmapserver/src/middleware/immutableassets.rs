use axum::extract::Request;
use axum::http::header;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Tags Vite's content-hashed static assets — served from the `/assets` mount —
/// with a long immutable `Cache-Control`. Every filename there carries a content
/// hash, so a given URL's bytes never change and it's safe to cache for a year.
/// Keying on the mount path (rather than the response's content-type) covers every
/// asset kind uniformly: js, css, wasm, fonts, and the hashed images/icons a
/// content-type allowlist used to miss.
///
/// Note this touches *assets only*, never HTML — server-rendered pages stay dynamic
/// and get a revalidating `ETag` + `no-cache` from the [`etag`](super::etag) layer.
///
/// The dev-mode hot-reload bundles (`css.css` / `lib.js`) keep stable names across
/// rebuilds, so they're excluded. A handler that already set its own
/// `Cache-Control` is left untouched.
pub async fn immutable_assets(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();

    let mut res = next.run(req).await;

    let is_hashed_asset =
        path.starts_with("/assets/") && !path.contains("css.css") && !path.contains("lib.js");

    if is_hashed_asset && !res.headers().contains_key(header::CACHE_CONTROL) {
        res.headers_mut().insert(
            header::CACHE_CONTROL,
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

    /// Fallback handler. Echoes an optional caller-supplied Cache-Control
    /// (`x-test-cc` header) so a test can check the middleware doesn't clobber a
    /// handler's own policy.
    async fn handler(req: Request) -> Response {
        if let Some(cc) = req.headers().get("x-test-cc").and_then(|v| v.to_str().ok()) {
            return ([(header::CACHE_CONTROL, cc.to_owned())], "body").into_response();
        }
        "body".into_response()
    }

    async fn oneshot(path: &str, cc: Option<&str>) -> Response {
        let app = Router::new()
            .fallback(handler)
            .layer(axum::middleware::from_fn(immutable_assets));
        let mut builder = axum::http::Request::builder().uri(path);
        if let Some(cc) = cc {
            builder = builder.header("x-test-cc", cc);
        }
        app.oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    fn cache_control(res: &Response) -> Option<String> {
        res.headers()
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    }

    fn is_immutable(res: &Response) -> bool {
        cache_control(res).is_some_and(|v| v.contains("immutable"))
    }

    #[tokio::test]
    async fn caches_hashed_assets_of_any_type() {
        // Every asset kind — including the images/icons a content-type allowlist
        // missed — is cached.
        for path in [
            "/assets/XJ5a_hVD.js",
            "/assets/Bz6ASMmM.css",
            "/assets/CofwWhv-.svg",
            "/assets/BuYe8tgW.png",
            "/assets/Ds4liTQf.ico",
        ] {
            assert!(
                is_immutable(&oneshot(path, None).await),
                "{path} should be cached immutable"
            );
        }
    }

    #[tokio::test]
    async fn does_not_cache_non_asset_paths() {
        for path in ["/", "/index.html", "/api/uiv2/minimap/5", "/favicon.ico"] {
            assert!(
                cache_control(&oneshot(path, None).await).is_none(),
                "{path} should not be tagged"
            );
        }
    }

    #[tokio::test]
    async fn dev_bundles_are_never_cached() {
        // css.css and lib.js are the hot-reloaded dev bundles; even under the asset
        // mount they must stay uncached.
        assert!(cache_control(&oneshot("/assets/css.css", None).await).is_none());
        assert!(cache_control(&oneshot("/assets/lib.js", None).await).is_none());
    }

    #[tokio::test]
    async fn does_not_override_a_handlers_own_cache_control() {
        let res = oneshot("/assets/whatever.js", Some("no-store")).await;
        assert_eq!(cache_control(&res).as_deref(), Some("no-store"));
    }
}
