use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use bwcommon::TrackingAnalytics;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::webutil::realip;

/// Resolves a stable tracking-analytics id (from the `tac` cookie, or a salted
/// hash of client fingerprint headers) into the request extensions, and sets
/// the `tac` cookie on the response when the request didn't already carry one.
pub async fn tracking_analytics(mut req: Request, next: Next) -> Response {
    let jar = CookieJar::from_headers(req.headers());

    let (tracking_analytics_id, was_provided_by_request) = if let Some(cookie) = jar.get("tac") {
        (cookie.value().to_string(), true)
    } else {
        const SALT: &[u8] = b"this is a salt so that nobody can guess the content of these hashes";

        let header_or = |name: &str| -> String {
            req.headers()
                .get(name)
                .and_then(|x| x.to_str().ok())
                .unwrap_or("default")
                .to_owned()
        };

        let user_agent = header_or("user-agent");
        let accept_language = header_or("accept-language");
        let sec_ch_ua = header_or("sec-ch-ua");
        let sec_ch_mobile = header_or("sec-ch-mobile");
        let sec_ch_platform = header_or("sec-ch-ua-platform");

        let peer = req
            .extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0);
        let ip_address = realip(req.headers(), peer)
            .unwrap_or_else(|| "default:5000".to_owned())
            .split(':')
            .next()
            .unwrap_or("default")
            .to_owned();

        info!("tac created. user_agent: {user_agent:?}, accept_language: {accept_language:?}, sec_ch_ua: {sec_ch_ua:?}, sec_ch_mobile: {sec_ch_mobile:?}, sec_ch_platform: {sec_ch_platform:?}, ip_address: {ip_address:?}");

        let mut hasher = Sha256::new();
        hasher.update(SALT);
        hasher.update(ip_address);
        hasher.update(user_agent);
        hasher.update(accept_language);
        hasher.update(sec_ch_ua);
        hasher.update(sec_ch_mobile);
        hasher.update(sec_ch_platform);
        (format!("{:x}", hasher.finalize()), false)
    };

    req.extensions_mut().insert(TrackingAnalytics {
        tracking_analytics_id: tracking_analytics_id.clone(),
        was_provided_by_request,
    });

    info!("tac: {}", tracking_analytics_id);

    let mut res = next.run(req).await;

    if !was_provided_by_request {
        let cookie = Cookie::build(("tac", tracking_analytics_id))
            .path("/")
            .same_site(SameSite::Lax)
            .secure(true)
            .http_only(false)
            .permanent()
            .build();
        res.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie.to_string()).unwrap(),
        );
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::routing::get;
    use axum::{Extension, Router};
    use tower::ServiceExt;

    /// Reports "<id>|<was_provided_by_request>".
    async fn echo_tac(ta: Option<Extension<TrackingAnalytics>>) -> String {
        ta.map_or_else(
            || "none".to_string(),
            |Extension(t)| format!("{}|{}", t.tracking_analytics_id, t.was_provided_by_request),
        )
    }

    async fn run(headers: &[(&str, &str)]) -> Response {
        let app = Router::new()
            .route("/", get(echo_tac))
            .layer(axum::middleware::from_fn(tracking_analytics));
        let mut builder = axum::http::Request::builder().uri("/");
        for (k, v) in headers {
            builder = builder.header(*k, *v);
        }
        app.oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    fn set_cookies(res: &Response) -> Vec<String> {
        res.headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect()
    }

    async fn body(res: Response) -> String {
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn reuses_existing_tac_cookie_without_resetting() {
        let res = run(&[("cookie", "tac=existing-id-123")]).await;
        assert!(
            set_cookies(&res).is_empty(),
            "an already-provided tac must not be re-set"
        );
        assert_eq!(body(res).await, "existing-id-123|true");
    }

    #[tokio::test]
    async fn generates_tac_when_absent_and_sets_cookie() {
        let res = run(&[("user-agent", "UA/1.0")]).await;
        let cookies = set_cookies(&res);
        assert!(
            cookies.iter().any(|c| c.starts_with("tac=")),
            "a fresh tac cookie must be set, got {cookies:?}"
        );
        let out = body(res).await;
        let (id, provided) = out.split_once('|').unwrap();
        assert_eq!(provided, "false");
        // The generated id is a SHA-256 hex digest.
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn generated_tac_is_deterministic_for_same_fingerprint() {
        let hdrs = &[
            ("user-agent", "Mozilla/5.0"),
            ("accept-language", "en-US"),
            ("sec-ch-ua-platform", "\"Linux\""),
        ];
        let a = body(run(hdrs).await).await;
        let b = body(run(hdrs).await).await;
        assert_eq!(a, b, "same fingerprint headers must hash to the same tac");
    }

    #[tokio::test]
    async fn different_user_agent_yields_different_tac() {
        let a = body(run(&[("user-agent", "Mozilla/5.0")]).await).await;
        let b = body(run(&[("user-agent", "curl/8.0")]).await).await;
        assert_ne!(a, b, "distinct fingerprints must produce distinct tacs");
    }
}
