//! Shared axum helpers: real-client-IP resolution and request extractors that
//! read data stashed in the request extensions by our middleware.

use std::net::SocketAddr;

use axum::extract::FromRequestParts;
use axum::response::Response;
use axum_extra::extract::cookie::{Cookie, SameSite};
use http::request::Parts;
use http::HeaderMap;

use crate::middleware::UserSession;

/// The bb8 Postgres connection pool type, spelled out once instead of at every
/// handler and helper signature.
pub type Pool = bb8_postgres::bb8::Pool<
    bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
>;

/// Best-effort real client IP. Behind Cloudflare the true client is carried in
/// `CF-Connecting-IP`, so it wins; otherwise fall back (mirroring actix's
/// `realip_remote_addr()`) to the left-most `X-Forwarded-For` entry, then the
/// `Forwarded` header's `for=`, then the raw peer address. Requests over the
/// WireGuard backdoor (CI/e2e) carry no `CF-Connecting-IP` and resolve via the
/// fallbacks.
///
/// Trusting `CF-Connecting-IP` is sound only because the origin is reachable
/// solely via the Cloudflare Tunnel and the WireGuard subnet (see the Cloudflare
/// migration runbook) — a publicly-reachable origin would let clients forge it.
///
/// The returned string may or may not include a port; callers that need a bare
/// address (e.g. as a stable per-client key) should use [`ip_only`], which is
/// IPv6-safe — do NOT hand-strip with `split(':')`, which mangles IPv6.
pub fn realip(headers: &HeaderMap, peer: Option<SocketAddr>) -> Option<String> {
    if let Some(cf) = headers
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
    {
        let cf = cf.trim();
        if !cf.is_empty() {
            return Some(cf.to_owned());
        }
    }

    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = xff.split(',').next() {
            let first = first.trim();
            if !first.is_empty() {
                return Some(first.to_owned());
            }
        }
    }

    if let Some(fwd) = headers.get("forwarded").and_then(|v| v.to_str().ok()) {
        // e.g. `Forwarded: for=192.0.2.60;proto=http;by=203.0.113.43`
        for part in fwd.split(';') {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("for=") {
                let rest = rest.trim_matches('"');
                if !rest.is_empty() {
                    return Some(rest.to_owned());
                }
            }
        }
    }

    peer.map(|addr| addr.to_string())
}

/// The bare IP from a [`realip`] string that may be `ip`, `ip:port`, or
/// `[ipv6]:port`. Parses off a port only when one is really present, so a bare
/// IPv6 address is returned intact — unlike `split(':').next()`, which would
/// truncate `2606:4700::1111` to `2606`.
pub fn ip_only(s: &str) -> String {
    match s.parse::<SocketAddr>() {
        Ok(sa) => sa.ip().to_string(),
        Err(_) => s.to_owned(),
    }
}

/// Extractor for the optional logged-in user. The `UserSessionTransformer`
/// middleware inserts a `UserSession` into the request extensions when the
/// request carries a valid session; this yields `None` otherwise.
pub struct MaybeUser(pub Option<UserSession>);

impl<S: Sync> FromRequestParts<S> for MaybeUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(MaybeUser(parts.extensions.get::<UserSession>().cloned()))
    }
}

impl MaybeUser {
    /// Convenience: the user id, if logged in.
    pub fn id(&self) -> Option<i64> {
        self.0.as_ref().map(|u| u.id)
    }
}

/// Append a `Set-Cookie` header to a response.
pub fn append_cookie(resp: &mut Response, cookie: Cookie<'static>) {
    resp.headers_mut().append(
        http::header::SET_COOKIE,
        http::HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );
}

/// A permanent auth cookie (path=/, SameSite=Lax), matching the old actix cookies.
pub fn auth_cookie(
    name: &'static str,
    value: String,
    secure: bool,
    http_only: bool,
) -> Cookie<'static> {
    Cookie::build((name, value))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(secure)
        .http_only(http_only)
        .permanent()
        .build()
}

/// A cookie that clears an existing one (empty value, expired at the epoch).
pub fn removal_cookie(name: &'static str) -> Cookie<'static> {
    let mut cookie = Cookie::build((name, ""))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(true)
        .build();
    cookie.make_removal();
    cookie
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(
                http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                http::HeaderValue::from_str(v).unwrap(),
            );
        }
        h
    }

    fn peer(s: &str) -> Option<SocketAddr> {
        Some(s.parse().unwrap())
    }

    #[test]
    fn realip_prefers_cf_connecting_ip() {
        // Behind Cloudflare, CF-Connecting-IP is the true client and wins over an
        // X-Forwarded-For that Cloudflare (or anyone upstream) may also have set.
        let h = headers(&[
            ("cf-connecting-ip", "198.51.100.9"),
            ("x-forwarded-for", "203.0.113.7, 70.41.3.18"),
        ]);
        assert_eq!(
            realip(&h, peer("10.0.0.1:5000")).as_deref(),
            Some("198.51.100.9")
        );
    }

    #[test]
    fn realip_blank_cf_connecting_ip_falls_through() {
        // A WireGuard/CI request has no CF-Connecting-IP; a blank one must not win.
        let h = headers(&[
            ("cf-connecting-ip", "   "),
            ("x-forwarded-for", "203.0.113.7"),
        ]);
        assert_eq!(realip(&h, None).as_deref(), Some("203.0.113.7"));
    }

    #[test]
    fn realip_prefers_leftmost_forwarded_for() {
        let h = headers(&[(
            "x-forwarded-for",
            "203.0.113.7, 70.41.3.18, 150.172.238.178",
        )]);
        assert_eq!(
            realip(&h, peer("10.0.0.1:5000")).as_deref(),
            Some("203.0.113.7")
        );
    }

    #[test]
    fn realip_single_forwarded_for() {
        let h = headers(&[("x-forwarded-for", "198.51.100.5")]);
        assert_eq!(realip(&h, None).as_deref(), Some("198.51.100.5"));
    }

    #[test]
    fn realip_empty_forwarded_for_falls_through_to_forwarded_header() {
        // A blank XFF entry must not win; the Forwarded `for=` is used next.
        let h = headers(&[
            ("x-forwarded-for", "   "),
            ("forwarded", "for=192.0.2.60;proto=http;by=203.0.113.43"),
        ]);
        assert_eq!(realip(&h, None).as_deref(), Some("192.0.2.60"));
    }

    #[test]
    fn realip_forwarded_header_strips_quotes() {
        let h = headers(&[("forwarded", "for=\"192.0.2.43\"")]);
        assert_eq!(realip(&h, None).as_deref(), Some("192.0.2.43"));
    }

    #[test]
    fn realip_falls_back_to_peer() {
        let h = headers(&[]);
        assert_eq!(
            realip(&h, peer("8.8.8.8:1234")).as_deref(),
            Some("8.8.8.8:1234")
        );
    }

    #[test]
    fn realip_none_when_nothing_available() {
        assert_eq!(realip(&headers(&[]), None), None);
    }

    #[test]
    fn ip_only_strips_port_but_keeps_ipv6_intact() {
        // IPv4, with and without a port.
        assert_eq!(ip_only("1.2.3.4:5000"), "1.2.3.4");
        assert_eq!(ip_only("1.2.3.4"), "1.2.3.4");
        // Bracketed IPv6 with a port collapses to the bare address.
        assert_eq!(ip_only("[2606:4700::1111]:443"), "2606:4700::1111");
        // A bare IPv6 (what CF-Connecting-IP yields) must NOT be truncated — this is
        // the `split(':').next()` bug the helper exists to avoid.
        assert_eq!(ip_only("2606:4700::1111"), "2606:4700::1111");
    }

    #[tokio::test]
    async fn maybe_user_present_and_absent() {
        // Absent: no UserSession in extensions.
        let mut parts = http::Request::builder().body(()).unwrap().into_parts().0;
        let MaybeUser(none) = MaybeUser::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert!(none.is_none());

        // Present: middleware would have inserted a UserSession.
        parts.extensions.insert(UserSession {
            id: 99,
            username: "trinity".to_string(),
            token: "tok".to_string(),
        });
        let user = MaybeUser::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(user.id(), Some(99));
        assert_eq!(user.0.unwrap().username, "trinity");
    }

    #[test]
    fn auth_cookie_attributes() {
        let c = auth_cookie("token", "abc".to_string(), true, true);
        assert_eq!(c.name(), "token");
        assert_eq!(c.value(), "abc");
        assert_eq!(c.path(), Some("/"));
        assert_eq!(c.same_site(), Some(SameSite::Lax));
        assert_eq!(c.secure(), Some(true));
        assert_eq!(c.http_only(), Some(true));
        // permanent() sets a far-future max-age.
        assert!(c.max_age().is_some());

        // secure/http_only are configurable.
        let c2 = auth_cookie("username", "neo".to_string(), false, false);
        assert_eq!(c2.secure(), Some(false));
        assert_eq!(c2.http_only(), Some(false));
    }

    #[test]
    fn removal_cookie_is_expired() {
        let c = removal_cookie("token");
        assert_eq!(c.value(), "");
        // make_removal() sets max-age to zero so the browser drops the cookie.
        let rendered = c.to_string();
        assert!(
            rendered.contains("Max-Age=0"),
            "removal cookie should expire immediately, got {rendered:?}"
        );
    }

    #[test]
    fn append_cookie_adds_set_cookie_header() {
        let mut resp = Response::new(Body::empty());
        append_cookie(
            &mut resp,
            auth_cookie("token", "xyz".to_string(), true, true),
        );
        append_cookie(&mut resp, removal_cookie("username"));

        let set: Vec<_> = resp
            .headers()
            .get_all(http::header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert_eq!(set.len(), 2, "append (not insert) keeps both cookies");
        assert!(set.iter().any(|s| s.starts_with("token=xyz")));
        assert!(set.iter().any(|s| s.starts_with("username=")));
    }
}
