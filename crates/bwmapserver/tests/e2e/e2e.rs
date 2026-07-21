//! End-to-end tests.
//!
//! Each test drives the **compiled `scmscx-com` binary** over HTTP against a real
//! Postgres. Being black-box, the suite is framework-agnostic: the exact same
//! tests validate the actix build on `lwm` and the axum build once the refactor
//! is rebased in — proving the migration didn't change observable behavior. They
//! cover the DB-backed paths unit tests can't reach: registration, login, session
//! validation, and rate limiting.
//!
//! The per-test [`Harness`](harness::Harness) — an isolated template-copied
//! database plus its own app process — and the HTTP helpers live in the `harness`
//! module; see there for how the suite is wired into `make e2e`.

mod harness;
mod map;

use harness::{cookie_value, is_session_valid, json_body, set_cookies, Harness};
use reqwest::StatusCode;

// ---------------------------------------------------------------------------
// Tests — each fully isolated (own DB, own app, own rate-limit budget), so they
// run in parallel and can be read and extended independently.
// ---------------------------------------------------------------------------

/// The language and `tac` middleware run on every request and set their cookies
/// exactly once, honoring `Accept-Language` and existing cookies.
#[tokio::test]
async fn middleware_sets_language_and_tac_cookies() {
    let h = Harness::start().await;
    let c = harness::client();

    // Fresh request → both cookies minted; language defaults to english.
    let resp = c.get(h.url("/sitemap.txt")).send().await.unwrap();
    assert!(resp.status().is_success());
    assert_eq!(cookie_value(&resp, "lang").as_deref(), Some("eng"));
    let tac = cookie_value(&resp, "tac").expect("tac cookie minted");
    assert_eq!(tac.len(), 64, "tac is a sha-256 hex digest");
    assert!(tac.chars().all(|ch| ch.is_ascii_hexdigit()));

    // Accept-Language negotiates korean.
    let resp = c
        .get(h.url("/sitemap.txt"))
        .header("accept-language", "ko-KR,ko;q=0.9")
        .send()
        .await
        .unwrap();
    assert_eq!(cookie_value(&resp, "lang").as_deref(), Some("kor"));

    // Caller-supplied cookies are respected and NOT re-set.
    let resp = c
        .get(h.url("/sitemap.txt"))
        .header("cookie", format!("lang=kor; tac={tac}"))
        .send()
        .await
        .unwrap();
    assert!(
        !set_cookies(&resp)
            .iter()
            .any(|c| c.starts_with("lang=") || c.starts_with("tac=")),
        "existing lang/tac cookies must not be reset, got {:?}",
        set_cookies(&resp)
    );
}

/// A representative spread of the data API returns 200 + well-formed JSON of the
/// expected shape (empty on the fresh schema — that the queries execute at all is
/// itself the invariant we want).
#[tokio::test]
async fn api_endpoints_return_json() {
    let h = Harness::start().await;
    let c = harness::client();

    // List endpoints serialize a Vec → JSON array.
    for path in [
        "/api/recent_activity",
        "/api/uiv2/last_uploaded_maps",
        "/api/uiv2/last_uploaded_replays",
        "/api/uiv2/featured_maps",
    ] {
        let resp = c.get(h.url(path)).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "GET {path}");
        assert!(
            json_body(resp).await.is_array(),
            "GET {path} should return a JSON array"
        );
    }

    // search returns { total_results, maps: [...] }.
    let resp = c
        .get(h.url("/api/uiv2/search?query="))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert!(
        body["maps"].is_array(),
        "search.maps should be an array, got {body}"
    );
    assert!(
        body["total_results"].is_number(),
        "search.total_results should be a number"
    );
}

/// Every documented `sort` value is accepted (each maps to an `ORDER BY` clause);
/// an unrecognized one is a hard error. This runs against an empty DB — the point
/// is that each branch produces valid SQL, not the ordering itself — so a deleted
/// sort match arm (which would fall through to the error path) turns its 200 into
/// a 500.
#[tokio::test]
async fn search_accepts_every_sort_order() {
    let h = Harness::start().await;
    let c = harness::client();

    for sort in [
        "relevancy",
        "scenario",
        "lastmodifiedold",
        "lastmodifiednew",
        "timeuploadedold",
        "timeuploadednew",
    ] {
        // Both the empty-query and keyword-query code paths share the sort match.
        for query in ["", "keyword"] {
            let resp = c
                .get(h.url(&format!("/api/uiv2/search/{query}?sort={sort}")))
                .send()
                .await
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "sort={sort:?} query={query:?} is a valid ordering"
            );
            assert!(json_body(resp).await["maps"].is_array());
        }
    }

    // An unknown sort is rejected up front by the handler's own allowlist (400),
    // before it ever reaches the query builder.
    assert_eq!(
        c.get(h.url("/api/uiv2/search/keyword?sort=nonsense"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::BAD_REQUEST,
        "an unrecognized sort is a 400"
    );
}

#[tokio::test]
async fn unknown_route_is_404() {
    let h = Harness::start().await;
    let c = harness::client();
    let resp = c
        .get(h.url("/definitely/not/a/route"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// The full account lifecycle — the DB-backed path the unit tests can't reach:
/// register → login → session validation, including the invalid-session logout.
#[tokio::test]
async fn auth_lifecycle() {
    let h = Harness::start().await;
    let c = harness::client();

    let user = "neo";
    let pass = "correct horse battery";
    let reg_body = |u: &str, p: &str, pc: &str| {
        serde_json::json!({ "username": u, "password": p, "password_confirm": pc }).to_string()
    };

    // Register → 200 with auth cookies.
    let resp = c
        .post(h.url("/api/register"))
        .header("content-type", "application/json")
        .body(reg_body(user, pass, pass))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "register should succeed");
    let reg_token = cookie_value(&resp, "token").expect("register sets a token cookie");
    assert_eq!(cookie_value(&resp, "username").as_deref(), Some(user));

    // Registering the same username again is rejected.
    let resp = c
        .post(h.url("/api/register"))
        .header("content-type", "application/json")
        .body(reg_body(user, pass, pass))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "duplicate username rejected"
    );

    let valid_url = h.url("/api/uiv2/is_session_valid");

    // A request carrying the auth cookies is a valid session...
    let resp = is_session_valid(
        &c,
        &valid_url,
        Some(&format!("username={user}; token={reg_token}")),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        json_body(resp).await,
        serde_json::json!(true),
        "cookies → valid session"
    );

    // ...no cookies → not logged in...
    assert_eq!(
        json_body(is_session_valid(&c, &valid_url, None).await).await,
        serde_json::json!(false)
    );

    // ...and a wrong token is a stale session → the middleware logs it out (301).
    let resp = is_session_valid(
        &c,
        &valid_url,
        Some(&format!("username={user}; token=deadbeefdeadbeef")),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::MOVED_PERMANENTLY,
        "invalid token should trigger a logout redirect"
    );

    // Login with the right password returns a usable token; wrong password 401s.
    let resp = c
        .post(h.url("/api/login"))
        .header("content-type", "application/json")
        .body(serde_json::json!({ "username": user, "password": pass }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "login should succeed");
    let login_token = cookie_value(&resp, "token").expect("login sets a token cookie");
    let resp = is_session_valid(
        &c,
        &valid_url,
        Some(&format!("username={user}; token={login_token}")),
    )
    .await;
    assert_eq!(
        json_body(resp).await,
        serde_json::json!(true),
        "login token is a valid session"
    );

    let resp = c
        .post(h.url("/api/login"))
        .header("content-type", "application/json")
        .body(serde_json::json!({ "username": user, "password": "wrong" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "wrong password rejected"
    );
}

/// The register endpoint is throttled per client IP (burst of 3). With a fresh
/// app the budget is full: three registrations succeed, the fourth is 429'd.
#[tokio::test]
async fn register_is_ip_rate_limited() {
    let h = Harness::start().await;
    let c = harness::client();
    let reg = |u: &str| {
        c.post(h.url("/api/register"))
            .header("content-type", "application/json")
            .body(
                serde_json::json!({ "username": u, "password": "pw", "password_confirm": "pw" })
                    .to_string(),
            )
    };

    for i in 0..3 {
        let status = reg(&format!("user{i}")).send().await.unwrap().status();
        assert_eq!(status, StatusCode::OK, "registration {i} within burst");
    }
    let resp = reg("user_over_burst").send().await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "4th register from one IP is throttled"
    );
    assert!(
        resp.headers().contains_key(reqwest::header::RETRY_AFTER),
        "429 carries Retry-After"
    );
}

/// The per-username login limiter allows a burst of 10 failed attempts, then 429s
/// — enforced separately from the IP limiter because the username lives in the
/// JSON body. A fresh app starts with a full budget.
#[tokio::test]
async fn login_is_username_rate_limited() {
    let h = Harness::start().await;
    let c = harness::client();
    let attempt = || {
        c.post(h.url("/api/login"))
            .header("content-type", "application/json")
            .body(serde_json::json!({ "username": "ghost", "password": "nope" }).to_string())
    };

    // First 10 attempts are plain auth failures.
    for i in 0..10 {
        let status = attempt().send().await.unwrap().status();
        assert_eq!(status, StatusCode::UNAUTHORIZED, "attempt {i} within burst");
    }
    // The 11th is throttled.
    let status = attempt().send().await.unwrap().status();
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "11th attempt for one username is throttled"
    );
}

/// A username of exactly 100 chars is at the cap and must log in: the guard is
/// `username.len() > 100`, so 100 is allowed. Pins the boundary against a `>`→`==`
/// mutation (which would reject a 100-char username as if it were over-long).
#[tokio::test]
async fn login_allows_boundary_length_username() {
    let h = Harness::start().await;
    let c = harness::client();
    let user = "u".repeat(100);
    register(&c, &h, &user, "pw").await;

    let resp = c
        .post(h.url("/api/login"))
        .header("x-forwarded-for", next_ip())
        .header("content-type", "application/json")
        .body(serde_json::json!({ "username": user, "password": "pw" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "a 100-char username is exactly at the cap and logs in"
    );
}

/// An empty username is rejected up front by the `is_empty() ||` guard, *before*
/// the per-username limiter is ever consulted — so a burst of empty-username logins
/// keeps returning a plain 401 and never trips the limiter. Pins that ordering
/// against a `||`→`&&` mutation, which makes the guard dead (a name can't be both
/// empty and >100), letting the empty key accrue in the limiter until the 11th
/// attempt would 429 instead of 401.
#[tokio::test]
async fn login_rejects_empty_username_before_the_limiter() {
    let h = Harness::start().await;
    let c = harness::client();
    let attempt = || {
        c.post(h.url("/api/login"))
            .header("content-type", "application/json")
            .body(serde_json::json!({ "username": "", "password": "x" }).to_string())
    };

    // The username limiter's burst is 10; send more than that from a single IP
    // (well under the per-IP login burst of 20). Every one must be a 401 — under
    // the mutant the 11th would be a 429.
    for i in 0..11 {
        let status = attempt().send().await.unwrap().status();
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "empty-username attempt {i} is a plain 401, never rate-limited"
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers for the workflows below.
// ---------------------------------------------------------------------------

/// A distinct client IP for each call. The per-IP register/login limiter (burst
/// 3 registrations, 10 logins per IP) keys off `X-Forwarded-For`, so stamping a
/// fresh IP lets one test legitimately make many attempts without tripping it —
/// the dedicated `*_rate_limited` tests above deliberately omit this to trip it.
fn next_ip() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    format!("10.1.{}.{}", (n >> 8) & 0xff, n & 0xff)
}

/// `username=…; token=…` cookie header for an authenticated request.
fn auth(user: &str, token: &str) -> String {
    format!("username={user}; token={token}")
}

/// Register `user`/`pass` from a fresh client IP and return the `token` cookie.
async fn register(c: &reqwest::Client, h: &Harness, user: &str, pass: &str) -> String {
    let resp = c
        .post(h.url("/api/register"))
        .header("x-forwarded-for", next_ip())
        .header("content-type", "application/json")
        .body(
            serde_json::json!({ "username": user, "password": pass, "password_confirm": pass })
                .to_string(),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "register {user}");
    cookie_value(&resp, "token").expect("register sets a token cookie")
}

/// Both logout handlers 307-redirect and clear the two auth cookies; they differ
/// only in destination (`/api/logout` → `/`, `/api/uiv2/logout` → `/uiv2`).
#[tokio::test]
async fn logout_endpoints_redirect_and_clear_cookies() {
    let h = Harness::start().await;
    let c = harness::client();

    for (path, location) in [("/api/logout", "/"), ("/api/uiv2/logout", "/uiv2")] {
        let resp = c.get(h.url(path)).send().await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::TEMPORARY_REDIRECT,
            "GET {path} is a 307"
        );
        assert_eq!(
            resp.headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some(location),
            "{path} redirects to {location}"
        );
        // Both cookies are reissued with an empty value → cleared in the browser.
        for name in ["username", "token"] {
            assert_eq!(
                cookie_value(&resp, name).as_deref(),
                Some(""),
                "{path} clears the {name} cookie, got {:?}",
                set_cookies(&resp)
            );
        }
    }
}

/// The user-session middleware logs out (301) any request whose cookies don't
/// resolve to a live session — a `username` with no `token`, and a `username` for
/// an account that doesn't exist — complementing the wrong-token case in
/// `auth_lifecycle`.
#[tokio::test]
async fn stale_session_cookies_are_logged_out() {
    let h = Harness::start().await;
    let c = harness::client();
    let valid_url = h.url("/api/uiv2/is_session_valid");

    // A username cookie with no token is a stale session → 301 logout.
    let resp = is_session_valid(&c, &valid_url, Some("username=ghost")).await;
    assert_eq!(
        resp.status(),
        StatusCode::MOVED_PERMANENTLY,
        "username without a token is logged out"
    );

    // A username for an account that was never registered → 301 logout.
    let resp = is_session_valid(
        &c,
        &valid_url,
        Some("username=ghost; token=deadbeefdeadbeef"),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::MOVED_PERMANENTLY,
        "an unknown account is logged out"
    );
}

/// The server-rendered HTML shells return 200 `text/html`. Two behaviors are
/// deliberately permissive and worth pinning against a refactor: an unknown
/// username still renders (there is no server-side 404 for users), and
/// `/moderation` is not gated server-side (admin checks live in the client bundle).
#[tokio::test]
async fn ssr_pages_render_html_shells() {
    let h = Harness::start().await;
    let c = harness::client();

    for path in [
        "/",
        "/about",
        "/upload",
        "/login",
        "/search",
        "/moderation",
        "/user/nonexistent-user",
    ] {
        let resp = c.get(h.url(path)).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "GET {path} renders");
        let ct = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        assert!(
            ct.starts_with("text/html"),
            "GET {path} is html (a 500 would be text/plain), got {ct:?}"
        );
        // A rendered template is non-trivial; an error body would be tiny.
        assert!(
            resp.text().await.unwrap().len() > 100,
            "GET {path} rendered a non-empty page"
        );
    }
}

/// `/search/{query}` renders the live result count into the page server-side (not
/// in the client bundle), and the query-less `/search` renders the generic
/// heading — both assertable over plain HTTP.
#[tokio::test]
async fn ssr_search_page_reports_result_count() {
    let h = Harness::start().await;
    let c = harness::client();

    let body = c
        .get(h.url("/search"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        body.contains("Search StarCraft: Brood War Maps"),
        "the query-less search page shows the generic title"
    );

    // Fresh DB → zero matches; the count is injected server-side.
    let body = c
        .get(h.url("/search/zzznomatchmarker"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        body.contains("0 maps found for: zzznomatchmarker"),
        "the search page reports the live (zero) count"
    );
}

/// Sitemaps, the PWA manifest, and the JS-redirect landing pages — small, mostly
/// static endpoints with exact, framework-independent contracts.
#[tokio::test]
async fn sitemaps_manifest_and_redirect_pages() {
    let h = Harness::start().await;
    let c = harness::client();

    // /sitemap.txt is a fixed six-line list.
    let resp = c.get(h.url("/sitemap.txt")).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.text().await.unwrap(),
        "https://scmscx.com/\n\
         https://scmscx.com/search\n\
         https://scmscx.com/about\n\
         https://scmscx.com/recent\n\
         https://scmscx.com/login\n\
         https://scmscx.com/register\n"
    );

    // The DB-backed sitemaps are empty on a fresh schema but still 200.
    for path in ["/a.txt", "/b.txt", "/c.txt"] {
        let resp = c.get(h.url(path)).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "GET {path}");
        // The real handler sets `text/plain` even when the body is empty; a mutant
        // that replaces it with `HttpResponse::Ok().finish()` would still 200 with
        // an empty body but drop the Content-Type, so pin the header explicitly.
        assert_eq!(
            resp.headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("text/plain"),
            "{path} is served as text/plain"
        );
        assert_eq!(
            resp.text().await.unwrap(),
            "",
            "{path} is empty on a fresh DB"
        );
    }

    // The web manifest is JSON carrying the app identity.
    let resp = c.get(h.url("/site.webmanifest")).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/json")
    );
    assert_eq!(
        json_body(resp).await["name"],
        serde_json::json!("scmscx.com")
    );

    // /map and /replay are 200 HTML landing pages that redirect client-side in JS
    // (not HTTP 30x).
    for path in ["/map", "/replay"] {
        let resp = c.get(h.url(path)).send().await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "GET {path} is a 200 landing page, not a redirect"
        );
        assert!(
            resp.text()
                .await
                .unwrap()
                .contains("window.location.replace"),
            "{path} redirects via inline JS"
        );
    }
}

/// Discovery/list and per-map lookup endpoints against a fresh (empty) DB. The
/// empty-DB behavior is a real contract: list endpoints return `[]`; `similar_maps`
/// (a plain query) returns an empty set; but the `query_one`-based lookups surface
/// "missing" as a **500** (the app has no 404-for-missing mapping), and the
/// privileged random-selection endpoint is a **401** without an allowlisted session.
#[tokio::test]
async fn discovery_endpoints_on_empty_db() {
    let h = Harness::start().await;
    let c = harness::client();

    for path in [
        "/api/uiv2/last_viewed_maps",
        "/api/uiv2/last_downloaded_maps",
        "/api/uiv2/most_viewed_maps",
        "/api/uiv2/most_downloaded_maps",
    ] {
        let resp = c.get(h.url(path)).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "GET {path}");
        assert_eq!(
            json_body(resp).await,
            serde_json::json!([]),
            "{path} is empty"
        );
    }

    // similar_maps uses a plain query, so an unknown id is 200 with an empty list.
    let resp = c
        .get(h.url("/api/similar_maps/12345"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await, serde_json::json!({ "v2": [] }));

    // random has nothing to choose from → 500 ("no maps found").
    assert_eq!(
        c.get(h.url("/api/uiv2/random"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "random over an empty DB is a 500"
    );

    // search_result_popup does a query_one → a missing map is a 500, not a 404.
    assert_eq!(
        c.get(h.url("/api/search_result_popup/12345"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "popup for a missing map is a 500"
    );

    // The privileged random-selection endpoint needs an allowlisted session.
    assert_eq!(
        c.get(h.url("/api/get_selection_of_random_maps"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED,
        "random-selection requires auth"
    );
}

/// Changing the password requires a session, and takes effect immediately: the
/// old password stops working and the new one logs in.
#[tokio::test]
async fn change_password_takes_effect() {
    let h = Harness::start().await;
    let c = harness::client();
    let token = register(&c, &h, "pwuser", "oldpass").await;

    let change = |cookie: Option<&str>| {
        let mut req = c
            .post(h.url("/api/change-password"))
            .header("content-type", "application/json")
            .body(
                serde_json::json!({ "password": "newpass", "password_confirm": "newpass" })
                    .to_string(),
            );
        if let Some(ck) = cookie {
            req = req.header("cookie", ck.to_string());
        }
        req.send()
    };

    // No session → 401.
    assert_eq!(
        change(None).await.unwrap().status(),
        StatusCode::UNAUTHORIZED,
        "change-password needs a session"
    );
    // With the session → 200.
    assert_eq!(
        change(Some(&auth("pwuser", &token)))
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "the owner changes the password"
    );

    let login = |pass: &str| {
        c.post(h.url("/api/login"))
            .header("x-forwarded-for", next_ip())
            .header("content-type", "application/json")
            .body(serde_json::json!({ "username": "pwuser", "password": pass }).to_string())
    };
    assert_eq!(
        login("oldpass").send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED,
        "the old password no longer works"
    );
    assert_eq!(
        login("newpass").send().await.unwrap().status(),
        StatusCode::OK,
        "the new password works"
    );
}

/// Changing the username requires the current password, issues a fresh `username`
/// cookie, and lets the account be reached (and log in) under the new name. The
/// existing `token` stays valid because it isn't reissued.
#[tokio::test]
async fn change_username_takes_effect() {
    let h = Harness::start().await;
    let c = harness::client();
    let token = register(&c, &h, "oldname", "secret").await;

    let rename = |password: &str| {
        c.post(h.url("/api/change-username"))
            .header("cookie", auth("oldname", &token))
            .header("content-type", "application/json")
            .body(
                serde_json::json!({
                    "username": "newname",
                    "username_confirm": "newname",
                    "password": password,
                })
                .to_string(),
            )
            .send()
    };

    // Wrong current password → 400.
    assert_eq!(
        rename("wrong").await.unwrap().status(),
        StatusCode::BAD_REQUEST,
        "the current password is required"
    );

    // Correct password → 200 and a new username cookie.
    let resp = rename("secret").await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "the rename succeeds");
    assert_eq!(
        cookie_value(&resp, "username").as_deref(),
        Some("newname"),
        "a fresh username cookie is issued"
    );

    // The unchanged token + new name is still a live session...
    let resp = is_session_valid(
        &c,
        &h.url("/api/uiv2/is_session_valid"),
        Some(&auth("newname", &token)),
    )
    .await;
    assert_eq!(
        json_body(resp).await,
        serde_json::json!(true),
        "the token still resolves under the new name"
    );
    // ...and login works under the new name.
    assert_eq!(
        c.post(h.url("/api/login"))
            .header("x-forwarded-for", next_ip())
            .header("content-type", "application/json")
            .body(serde_json::json!({ "username": "newname", "password": "secret" }).to_string())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "login under the new name works"
    );
}

/// The credential length caps are `> 100` (100 is allowed, 101 is not). Pinning
/// both sides of the boundary catches an off-by-one mutation of the comparison
/// (`>` → `==`/`>=`), which a test that only used short inputs would miss.
#[tokio::test]
async fn register_enforces_length_bounds() {
    let h = Harness::start().await;
    let c = harness::client();
    let do_register = |u: String, p: String| {
        c.post(h.url("/api/register"))
            .header("x-forwarded-for", next_ip())
            .header("content-type", "application/json")
            .body(
                serde_json::json!({ "username": u, "password": p, "password_confirm": p })
                    .to_string(),
            )
    };

    // Username: exactly 100 chars is accepted; 101 is rejected.
    assert_eq!(
        do_register("u".repeat(100), "pw".into())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "a 100-char username is allowed"
    );
    let resp = do_register("u".repeat(101), "pw".into())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.text().await.unwrap(),
        "Why would you try to create a username that long",
        "a 101-char username is rejected"
    );

    // Password: exactly 100 chars is accepted; 101 is rejected.
    assert_eq!(
        do_register("pw_len_ok".into(), "p".repeat(100))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "a 100-char password is allowed"
    );
    let resp = do_register("pw_len_over".into(), "p".repeat(101))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.text().await.unwrap(),
        "Why would you try to create a password that long",
        "a 101-char password is rejected"
    );
}

/// change-password enforces the same `> 100` cap on the new password.
#[tokio::test]
async fn change_password_enforces_length_bound() {
    let h = Harness::start().await;
    let c = harness::client();
    let token = register(&c, &h, "cpwlen", "oldpass").await;
    let change = |p: String| {
        c.post(h.url("/api/change-password"))
            .header("cookie", auth("cpwlen", &token))
            .header("content-type", "application/json")
            .body(serde_json::json!({ "password": p, "password_confirm": p }).to_string())
    };

    // 100 chars: accepted. 101 chars: rejected with the length message.
    assert_eq!(
        change("p".repeat(100)).send().await.unwrap().status(),
        StatusCode::OK,
        "a 100-char password change is allowed"
    );
    let resp = change("p".repeat(101)).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.text().await.unwrap(),
        "Why would you try to create a password that long"
    );
}

/// change-username enforces the same `> 100` cap on the new username. The length
/// gate runs before the password check, so the over-long case trips it regardless
/// of the password; the boundary (100) still needs the correct password to reach a
/// success.
#[tokio::test]
async fn change_username_enforces_length_bound() {
    let h = Harness::start().await;
    let c = harness::client();
    let token = register(&c, &h, "culen", "secret").await;
    let rename = |u: String, pass: &str| {
        c.post(h.url("/api/change-username"))
            .header("cookie", auth("culen", &token))
            .header("content-type", "application/json")
            .body(
                serde_json::json!({ "username": u, "username_confirm": u, "password": pass })
                    .to_string(),
            )
    };

    // 101 chars trips the length gate (before the password is even checked).
    let resp = rename("u".repeat(101), "secret").send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.text().await.unwrap(),
        "Why would you try to create a username that long"
    );

    // Exactly 100 chars is under the cap and (with the right password) succeeds.
    assert_eq!(
        rename("u".repeat(100), "secret")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "a 100-char username is allowed"
    );
}

/// In production mode the auth cookies login issues are marked `Secure` (the flag is
/// `secure(!is_dev_mode())`, and the harness runs in prod mode). Dropping the `!`
/// would ship them without `Secure`.
#[tokio::test]
async fn login_sets_secure_auth_cookies() {
    let h = Harness::start().await;
    let c = harness::client();
    register(&c, &h, "secureuser", "pw").await;

    let resp = c
        .post(h.url("/api/login"))
        .header("x-forwarded-for", next_ip())
        .header("content-type", "application/json")
        .body(serde_json::json!({ "username": "secureuser", "password": "pw" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    for name in ["token", "username"] {
        let set_cookie = set_cookies(&resp)
            .into_iter()
            .find(|c| c.starts_with(&format!("{name}=")))
            .unwrap_or_else(|| panic!("login sets a {name} cookie"));
        assert!(
            set_cookie.contains("Secure"),
            "the {name} cookie is Secure in prod mode: {set_cookie}"
        );
    }
}

/// The mutating endpoints reject malformed input with specific 4xx codes and
/// messages. This pins the message text and — deliberately — the *order* the
/// checks run in, which a coarser status-only test would let a refactor reorder.
#[tokio::test]
async fn mutation_endpoints_validate_input() {
    let h = Harness::start().await;
    let c = harness::client();

    // Each register call uses a fresh IP so validation 400s don't burn the limiter.
    let do_register = |u: &str, p: &str, pc: &str| {
        c.post(h.url("/api/register"))
            .header("x-forwarded-for", next_ip())
            .header("content-type", "application/json")
            .body(
                serde_json::json!({ "username": u, "password": p, "password_confirm": pc })
                    .to_string(),
            )
    };
    let assert_400 = |body: String, expected: &'static str| {
        assert!(
            body == expected,
            "expected 400 body {expected:?}, got {body:?}"
        );
    };

    // register: empty username.
    let resp = do_register("", "p", "p").send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_400(
        resp.text().await.unwrap(),
        "The provided username must not be the empty string",
    );
    // register: password mismatch is checked BEFORE the password-empty check.
    let resp = do_register("u1", "a", "b").send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_400(
        resp.text().await.unwrap(),
        "The two provided passwords must match",
    );
    // register: matching-but-empty passwords then hit the empty check.
    let resp = do_register("u2", "", "").send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_400(
        resp.text().await.unwrap(),
        "The provided password must not be the empty string",
    );

    // change-password: auth is checked before any field validation.
    let resp = c
        .post(h.url("/api/change-password"))
        .header("content-type", "application/json")
        .body(serde_json::json!({ "password": "x", "password_confirm": "y" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "auth precedes field validation"
    );

    // change-username: empty username is checked BEFORE the confirm-match.
    let token = register(&c, &h, "cuvalidate", "pw").await;
    let rename = |u: &str, uc: &str| {
        c.post(h.url("/api/change-username"))
            .header("cookie", auth("cuvalidate", &token))
            .header("content-type", "application/json")
            .body(
                serde_json::json!({ "username": u, "username_confirm": uc, "password": "pw" })
                    .to_string(),
            )
    };
    let resp = rename("", "").send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_400(
        resp.text().await.unwrap(),
        "The provided username must not be the empty string",
    );
    let resp = rename("aaa", "bbb").send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_400(
        resp.text().await.unwrap(),
        "The provided usernames must match",
    );
}
