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
