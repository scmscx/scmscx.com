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

    let (Some(cookie_username), Some(cookie_token)) = (jar.get("username"), jar.get("token"))
    else {
        return next.run(req).await;
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
