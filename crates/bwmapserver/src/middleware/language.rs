use axum::extract::{Extension, Request};
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::CookieJar;

use crate::i18n::{self, Source, Strings};
use std::sync::Arc;

/// The language for this request, in the request extensions.
///
/// A newtype rather than a bare `String` so the extension lookup cannot collide
/// with some other string a middleware stashes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Language(pub String);

/// The cookie the frontend reads. Shared name, one authority.
pub const LANG_COOKIE: &str = "lang2";

/// Resolves the request language, stashes it in the extensions, and stores it in
/// the `lang2` cookie -- but only when the visitor actually asked for a language
/// we have.
///
/// Storing it is what keeps the two sides in agreement: the browser applies
/// `Set-Cookie` before it runs any script, so `app/modules/context.tsx` finds the
/// cookie already present and adopts this decision rather than making its own
/// from `navigator.languages`.
///
/// Falling back to English is deliberately NOT stored. Someone browsing in Thai
/// would otherwise be pinned to English by their own cookie, and adding Thai
/// later would never reach them; leaving them cookie-less means every request
/// re-resolves and they get it the day it exists.
pub async fn language(
    Extension(strings): Extension<Arc<Strings>>,
    mut req: Request,
    next: Next,
) -> Response {
    let jar = CookieJar::from_headers(req.headers());
    let cookie = jar.get(LANG_COOKIE).map(|c| c.value().to_string());

    let accept_language = req
        .headers()
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let (lang, source) = i18n::resolve(cookie.as_deref(), accept_language.as_deref(), &strings);

    req.extensions_mut().insert(Language(lang.clone()));

    let mut res = next.run(req).await;

    // Only HTML varies by the language we just picked, so only HTML says so.
    // Stamping it on everything would include /assets/*, which are content
    // hashed and served `immutable` for a year -- and browsers do honour Vary,
    // so `Vary: cookie` there means the entire bundle is re-downloaded the first
    // time any cookie changes (logging in, or `tac` simply appearing).
    if crate::middleware::response_is_html(&res) {
        res.headers_mut().append(
            header::VARY,
            HeaderValue::from_static("accept-language, cookie"),
        );
    }

    // Don't attach a cookie to a shared-cacheable response: a Set-Cookie makes
    // Cloudflare bypass its cache (see `response_is_publicly_cacheable`).
    if source == Source::AcceptLanguage && !crate::middleware::response_is_publicly_cacheable(&res)
    {
        // Permanent, matching what the frontend writes in context.tsx. A session
        // cookie here would expire on browser close and re-resolve to the same
        // answer, which is harmless but means the two sides disagree about the
        // lifetime of a cookie they share.
        crate::webutil::append_cookie(&mut res, crate::webutil::lang_cookie(lang));
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

    /// Echoes the language the middleware resolved, as HTML -- the Vary header
    /// is only stamped on HTML, so the test route has to look like a page.
    async fn echo(lang: Option<Extension<Language>>) -> Response {
        let body = lang.map_or_else(|| "none".to_string(), |Extension(l)| l.0);
        ([(header::CONTENT_TYPE, "text/html")], body).into_response()
    }

    async fn run(headers: &[(&str, &str)]) -> Response {
        let strings = Arc::new(Strings::load().unwrap());
        let app = Router::new()
            .route("/", get(echo))
            .layer(axum::middleware::from_fn(language))
            .layer(Extension(strings));

        let mut builder = axum::http::Request::builder().uri("/");
        for (k, v) in headers {
            builder = builder.header(*k, *v);
        }
        app.oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn body_of(res: Response) -> String {
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn set_cookies(res: &Response) -> Vec<String> {
        res.headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect()
    }

    #[tokio::test]
    async fn defaults_to_english_without_storing_it() {
        let res = run(&[]).await;
        assert!(
            !set_cookies(&res).iter().any(|c| c.starts_with("lang2=")),
            "a fallback must not be stored, got {:?}",
            set_cookies(&res)
        );
        assert_eq!(body_of(res).await, "en");
    }

    /// A visitor whose language we do not carry gets English, but is left
    /// cookie-less so that adding their language later just starts working.
    #[tokio::test]
    async fn an_unsupported_language_is_not_stored() {
        let res = run(&[("accept-language", "th,vi;q=0.9")]).await;
        assert!(
            !set_cookies(&res).iter().any(|c| c.starts_with("lang2=")),
            "got {:?}",
            set_cookies(&res)
        );
        assert_eq!(body_of(res).await, "en");
    }

    #[tokio::test]
    async fn negotiates_from_accept_language() {
        let res = run(&[("accept-language", "ko-KR,ko;q=0.9,en-US;q=0.8")]).await;
        assert!(set_cookies(&res).iter().any(|c| c.starts_with("lang2=ko")));
        assert_eq!(body_of(res).await, "ko");
    }

    #[tokio::test]
    async fn an_existing_cookie_wins_and_is_not_re_set() {
        let res = run(&[("cookie", "lang2=de"), ("accept-language", "ko")]).await;
        assert!(
            !set_cookies(&res).iter().any(|c| c.starts_with("lang2=")),
            "a cookie we honored must not be re-set, got {:?}",
            set_cookies(&res)
        );
        assert_eq!(body_of(res).await, "de");
    }

    #[tokio::test]
    async fn an_unsupported_cookie_is_replaced() {
        let res = run(&[("cookie", "lang2=xx"), ("accept-language", "fr")]).await;
        assert!(set_cookies(&res).iter().any(|c| c.starts_with("lang2=fr")));
        assert_eq!(body_of(res).await, "fr");
    }

    /// A non-HTML response must not carry the language Vary: `/assets/*` are
    /// content-hashed and `immutable`, and a `Vary: cookie` there costs the
    /// whole bundle on the first cookie change.
    #[tokio::test]
    async fn non_html_does_not_declare_a_language_vary() {
        async fn asset() -> Response {
            ([(header::CONTENT_TYPE, "application/javascript")], "x").into_response()
        }

        let strings = Arc::new(Strings::load().unwrap());
        let app = Router::new()
            .route("/", get(asset))
            .layer(axum::middleware::from_fn(language))
            .layer(Extension(strings));

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            res.headers().get_all(header::VARY).iter().count() == 0,
            "assets must not vary by language, got {:?}",
            res.headers()
                .get_all(header::VARY)
                .iter()
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn response_declares_what_it_varies_on() {
        let res = run(&[]).await;
        let vary = res
            .headers()
            .get_all(header::VARY)
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect::<Vec<_>>()
            .join(", ");
        assert!(vary.contains("accept-language"), "got {vary:?}");
        assert!(vary.contains("cookie"), "got {vary:?}");
    }
}
