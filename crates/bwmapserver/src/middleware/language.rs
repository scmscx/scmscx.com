use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use tracing::info;

/// Resolves the request language (from the `lang` cookie or `Accept-Language`),
/// stashes the `LangData` in the request extensions, and sets the `lang` cookie
/// on the response when it wasn't already provided.
pub async fn language(mut req: Request, next: Next) -> Response {
    let jar = CookieJar::from_headers(req.headers());

    let (lang, is_lang_set, langdata) = if let Some(cookie) = jar.get("lang") {
        let lang = cookie.value().to_string();

        if lang == "eng" {
            ("eng".to_string(), true, bwcommon::LangData::English)
        } else if lang == "kor" {
            ("kor".to_string(), true, bwcommon::LangData::Korean)
        } else {
            ("eng".to_string(), false, bwcommon::LangData::English)
        }
    } else {
        let langstring = req
            .headers()
            .get("Accept-Language")
            .and_then(|x| x.to_str().ok())
            .unwrap_or("en-US,en;q=0.9")
            .to_owned();

        let mut ret = ("eng".to_string(), false, bwcommon::LangData::English);

        for langcode in langstring.split(',') {
            if langcode.contains("en") {
                ret = ("eng".to_string(), false, bwcommon::LangData::English);
                break;
            } else if langcode.contains("ko") {
                ret = ("kor".to_string(), false, bwcommon::LangData::Korean);
                break;
            }
        }

        ret
    };

    info!("setting language data: {:?}", langdata);
    req.extensions_mut().insert(langdata);

    let mut res = next.run(req).await;

    if !is_lang_set {
        let cookie = Cookie::build(("lang", lang))
            .path("/")
            .same_site(SameSite::Lax)
            .secure(true)
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

    /// Echoes the LangData the `language` middleware stashed in the extensions.
    async fn echo_lang(lang: Option<Extension<bwcommon::LangData>>) -> String {
        match lang {
            Some(Extension(bwcommon::LangData::English)) => "English".to_string(),
            Some(Extension(bwcommon::LangData::Korean)) => "Korean".to_string(),
            None => "none".to_string(),
        }
    }

    async fn run(headers: &[(&str, &str)]) -> Response {
        let app = Router::new()
            .route("/", get(echo_lang))
            .layer(axum::middleware::from_fn(language));

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
    async fn accept_language_korean_sets_cookie_and_extension() {
        let res = run(&[("accept-language", "ko-KR,ko;q=0.9")]).await;
        let cookies = set_cookies(&res);
        assert!(
            cookies.iter().any(|c| c.starts_with("lang=kor")),
            "expected lang=kor Set-Cookie, got {cookies:?}"
        );
        assert_eq!(body(res).await, "Korean");
    }

    #[tokio::test]
    async fn accept_language_english_sets_cookie_and_extension() {
        let res = run(&[("accept-language", "en-US,en;q=0.9")]).await;
        assert!(set_cookies(&res).iter().any(|c| c.starts_with("lang=eng")));
        assert_eq!(body(res).await, "English");
    }

    #[tokio::test]
    async fn no_headers_defaults_to_english_and_sets_cookie() {
        let res = run(&[]).await;
        assert!(set_cookies(&res).iter().any(|c| c.starts_with("lang=eng")));
        assert_eq!(body(res).await, "English");
    }

    #[tokio::test]
    async fn valid_lang_cookie_is_respected_and_not_reset() {
        let res = run(&[("cookie", "lang=kor")]).await;
        // When the request already carries a valid lang cookie, the middleware
        // does NOT emit a Set-Cookie.
        assert!(
            set_cookies(&res).is_empty(),
            "should not re-set an already-provided lang cookie"
        );
        assert_eq!(body(res).await, "Korean");
    }

    #[tokio::test]
    async fn cookie_overrides_accept_language_header() {
        // Cookie wins over the header.
        let res = run(&[("cookie", "lang=eng"), ("accept-language", "ko")]).await;
        assert!(set_cookies(&res).is_empty());
        assert_eq!(body(res).await, "English");
    }

    #[tokio::test]
    async fn garbage_lang_cookie_falls_back_to_english_and_resets_cookie() {
        let res = run(&[("cookie", "lang=zzz")]).await;
        // Unknown value → English, and because is_lang_set is false the cookie
        // is (re)written to a valid value.
        assert!(set_cookies(&res).iter().any(|c| c.starts_with("lang=eng")));
        assert_eq!(body(res).await, "English");
    }
}
