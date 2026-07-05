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

/// Best-effort real client IP, mirroring actix's `realip_remote_addr()`:
/// prefer the left-most `X-Forwarded-For` entry (set by our reverse proxy),
/// then the `Forwarded` header's `for=`, then the raw peer address.
///
/// The returned string may or may not include a port; callers that need a bare
/// address should strip it (`split(':').next()`), matching the old code.
pub fn realip(headers: &HeaderMap, peer: Option<SocketAddr>) -> Option<String> {
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
