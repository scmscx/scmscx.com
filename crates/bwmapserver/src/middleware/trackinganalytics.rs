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
