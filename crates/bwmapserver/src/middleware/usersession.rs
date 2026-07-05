use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use tracing::info;

use crate::webutil::Pool;

#[derive(Clone, Debug)]
pub struct UserSession {
    pub id: i64,
    pub username: String,
    pub token: String,
}

/// Build a 301 response that logs the user out by clearing the auth cookies.
fn log_out_user() -> Response {
    let mut resp = Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header(header::LOCATION, "/")
        .body(axum::body::Body::empty())
        .unwrap();

    for name in ["username", "token"] {
        let mut cookie = Cookie::build((name, ""))
            .path("/")
            .same_site(SameSite::Lax)
            .secure(true)
            .build();
        cookie.make_removal();
        resp.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie.to_string()).unwrap(),
        );
    }

    resp
}

/// Validates the `username`/`token` cookie pair against the DB and, when valid,
/// inserts a `UserSession` into the request extensions. An invalid/stale
/// session short-circuits with a logout redirect.
pub async fn user_session(pool: Pool, mut req: Request, next: Next) -> Response {
    let jar = CookieJar::from_headers(req.headers());

    // No username cookie → not logged in, proceed untouched.
    let Some(cookie_username) = jar.get("username") else {
        return next.run(req).await;
    };

    // A username cookie without a matching token is a stale/invalid session and
    // is logged out (matching the pre-axum actix behavior — this branch is a pure
    // refactor and must not change it).
    let Some(cookie_token) = jar.get("token") else {
        return log_out_user();
    };

    let con = pool.get().await.unwrap();
    let row = con
        .query_opt(
            "select id, token, username from account where username = $1",
            &[&cookie_username.value()],
        )
        .await
        .unwrap();

    let Some(row) = row else {
        return log_out_user();
    };

    let (id, token, username) = (
        row.get::<_, i64>(0),
        row.get::<_, String>(1),
        row.get::<_, String>(2),
    );

    if cookie_token.value() == token.as_str() {
        info!("id: {}, username: {}, token: {}", id, username, token);
        req.extensions_mut().insert(UserSession {
            id,
            username,
            token,
        });
        next.run(req).await
    } else {
        log_out_user()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::routing::get;
    use axum::{Extension, Router};
    use tower::ServiceExt;

    /// A pool that never connects (port 1). Only the branches that don't touch
    /// the DB are exercised here; anything that calls `pool.get()` would fail.
    fn dead_pool() -> Pool {
        let manager = bb8_postgres::PostgresConnectionManager::new(
            "host=127.0.0.1 port=1 user=x dbname=x".parse().unwrap(),
            bb8_postgres::tokio_postgres::NoTls,
        );
        bb8_postgres::bb8::Pool::builder().build_unchecked(manager)
    }

    #[test]
    fn log_out_user_clears_auth_cookies_and_redirects() {
        let resp = log_out_user();
        assert_eq!(resp.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(resp.headers().get(header::LOCATION).unwrap(), "/");

        let cookies: Vec<String> = resp
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert_eq!(cookies.len(), 2);
        // Both auth cookies are cleared (Max-Age=0).
        for name in ["username", "token"] {
            assert!(
                cookies
                    .iter()
                    .any(|c| c.starts_with(&format!("{name}=")) && c.contains("Max-Age=0")),
                "expected a cleared {name} cookie, got {cookies:?}"
            );
        }
    }

    #[tokio::test]
    async fn passes_through_when_no_auth_cookies() {
        // Without username+token cookies the middleware returns early, before
        // ever touching the pool, so the dead pool is never queried.
        async fn echo(user: Option<Extension<UserSession>>) -> &'static str {
            if user.is_some() {
                "user"
            } else {
                "anon"
            }
        }

        let pool = dead_pool();
        let app = Router::new()
            .route("/", get(echo))
            .layer(axum::middleware::from_fn(move |req, next| {
                user_session(pool.clone(), req, next)
            }));

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"anon");
    }

    #[tokio::test]
    async fn username_without_token_logs_out_and_clears_cookies() {
        // A `username` cookie without a `token` is a stale session → 301 logout
        // that clears both cookies. This is DB-free (it returns before the pool
        // is touched) and matches the actix behavior — parity is the whole point.
        let pool = dead_pool();
        let app =
            Router::new()
                .route("/", get(|| async { "ok" }))
                .layer(axum::middleware::from_fn(move |req, next| {
                    user_session(pool.clone(), req, next)
                }));
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .header("cookie", "username=neo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.headers().get(header::LOCATION).unwrap(), "/");

        let cookies: Vec<String> = res
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert_eq!(cookies.len(), 2);
        for name in ["username", "token"] {
            assert!(
                cookies
                    .iter()
                    .any(|c| c.starts_with(&format!("{name}=")) && c.contains("Max-Age=0")),
                "expected a cleared {name} cookie, got {cookies:?}"
            );
        }
    }
}
