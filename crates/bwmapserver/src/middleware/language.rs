// let (lang, is_lang_set, langdata) = if let Some(cookie) = req.cookie("lang") {
//     let lang = cookie.value().to_string();

//     if lang == "eng" {
//         ("eng".to_string(), true, bwcommon::LangData::English)
//     } else if lang == "kor" {
//         ("kor".to_string(), true, bwcommon::LangData::Korean)
//     } else {
//         ("eng".to_string(), false, bwcommon::LangData::English)
//     }
// } else {
//     let langstring = req.headers().get("Accept-Language").unwrap_or(&HeaderValue::from_static("en-US,en;q=0.9")).to_str().unwrap_or("nullnull").to_owned();

//     let mut ret = ("eng".to_string(), false, bwcommon::LangData::English);

//     for langcode in langstring.split(',') {
//         if langcode.contains("en") {
//             ret = ("eng".to_string(), false, bwcommon::LangData::English);
//             break;
//         } else if langcode.contains("ko") {
//             ret = ("kor".to_string(), false, bwcommon::LangData::Korean);
//             break;
//         }
//     }

//     ret
// };

// req.extensions_mut().insert(langdata);

// let fut = srv.call(req);
// Box::pin(async move {
//     let mut res = fut.await?;

//     if !is_lang_set {
//         res.response_mut()
//             .add_cookie(
//                 &Cookie::build("lang", &lang)
//                     .path("/")
//                     .same_site(SameSite::Lax)
//                     .secure(true)
//                     .finish(),
//             )
//             .unwrap();
//     }

//     Ok(res)
// })
//     let path = req.path().to_owned();
//     let trace_id: String = uuid::Uuid::new_v4()
//         .as_simple()
//         .to_string()
//         .chars()
//         .take(6)
//         .collect();
//     let ip = req
//         .connection_info()
//         .realip_remote_addr()
//         .unwrap_or("x.x.x.x")
//         .to_owned();
//     let user_agent = req
//         .headers()
//         .get("user-agent")
//         .map(|x| x.to_str().unwrap_or("couldn't unwrap").to_owned())
//         .unwrap_or("couldn't unwrap2".to_string());
//     req.extensions_mut().insert(TraceID {
//         id: trace_id.clone(),
//         start_time: Instant::now(),
//     });
//     let fut = self.service.call(req);
//     async move {
//         match fut.await {
//             Ok(x) => {
//                 if x.status().is_success() {
//                     info!(status=%x.status(), %path, %ip, %user_agent);
//                 } else if x.status().is_redirection() {
//                     info!(status=%x.status(), %path, %ip, %user_agent);
//                 } else if x.status().is_client_error() {
//                     warn!(status=%x.status(), %path, %ip, %user_agent);
//                 } else if x.status().is_server_error() {
//                     error!(status=%x.status(), %path, %ip, %user_agent);
//                 } else {
//                     warn!(status=%x.status(), %path, %ip, %user_agent);
//                 }
//                 Ok(x)
//             }
//             Err(x) => {
//                 error!(%path, %ip, %user_agent, err=?x);
//                 Err(x)
//             }
//         }
//     }
//     .instrument(info_span!("traceid", trace_id = %trace_id))
//     .boxed_local()
// }

use std::future::{ready, Ready};

use actix_web::http::header::HeaderValue;
use actix_web::{
    cookie::{Cookie, SameSite},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::{future::LocalBoxFuture, FutureExt};
use log::info;
use tracing::{instrument, Instrument};

pub struct LanguageTransformer;

impl<S, B> Transform<S, ServiceRequest> for LanguageTransformer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LanguageMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LanguageMiddleware { service }))
    }
}

pub struct LanguageMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LanguageMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    #[instrument(skip_all, name = "")]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let (lang, is_lang_set, langdata) = if let Some(cookie) = req.cookie("lang") {
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
                .unwrap_or(&HeaderValue::from_static("en-US,en;q=0.9"))
                .to_str()
                .unwrap_or("nullnull")
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

        let fut = self.service.call(req);
        async move {
            let mut res = fut.await?;

            if !is_lang_set {
                res.response_mut()
                    .add_cookie(
                        &Cookie::build("lang", &lang)
                            .path("/")
                            .same_site(SameSite::Lax)
                            .secure(true)
                            .finish(),
                    )
                    .unwrap();
            }

            Ok(res)
        }
        .instrument(tracing::span::Span::current())
        .boxed_local()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::SET_COOKIE;
    use actix_web::{test, web, App, HttpMessage, HttpRequest, HttpResponse};

    /// Echoes the LangData the middleware stashed in the request extensions.
    async fn echo_lang(req: HttpRequest) -> HttpResponse {
        let lang = req.extensions().get::<bwcommon::LangData>().cloned();
        let body = match lang {
            Some(bwcommon::LangData::English) => "English",
            Some(bwcommon::LangData::Korean) => "Korean",
            None => "none",
        };
        HttpResponse::Ok().body(body)
    }

    async fn run(req: test::TestRequest) -> (Vec<String>, String) {
        let app = test::init_service(
            App::new()
                .wrap(LanguageTransformer)
                .default_service(web::to(echo_lang)),
        )
        .await;
        let resp = test::call_service(&app, req.to_request()).await;
        let cookies = resp
            .headers()
            .get_all(SET_COOKIE)
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
        (cookies, body)
    }

    #[actix_web::test]
    async fn accept_language_korean_sets_cookie_and_extension() {
        let (cookies, body) =
            run(test::TestRequest::get().insert_header(("accept-language", "ko-KR,ko;q=0.9")))
                .await;
        assert!(
            cookies.iter().any(|c| c.starts_with("lang=kor")),
            "expected lang=kor Set-Cookie, got {cookies:?}"
        );
        assert_eq!(body, "Korean");
    }

    #[actix_web::test]
    async fn accept_language_english_sets_cookie_and_extension() {
        let (cookies, body) =
            run(test::TestRequest::get().insert_header(("accept-language", "en-US,en;q=0.9")))
                .await;
        assert!(cookies.iter().any(|c| c.starts_with("lang=eng")));
        assert_eq!(body, "English");
    }

    #[actix_web::test]
    async fn no_headers_defaults_to_english_and_sets_cookie() {
        let (cookies, body) = run(test::TestRequest::get()).await;
        assert!(cookies.iter().any(|c| c.starts_with("lang=eng")));
        assert_eq!(body, "English");
    }

    #[actix_web::test]
    async fn valid_lang_cookie_is_respected_and_not_reset() {
        let (cookies, body) =
            run(test::TestRequest::get().cookie(actix_web::cookie::Cookie::new("lang", "kor")))
                .await;
        assert!(
            cookies.is_empty(),
            "should not re-set an already-provided lang cookie"
        );
        assert_eq!(body, "Korean");
    }

    #[actix_web::test]
    async fn cookie_overrides_accept_language_header() {
        let (cookies, body) = run(test::TestRequest::get()
            .cookie(actix_web::cookie::Cookie::new("lang", "eng"))
            .insert_header(("accept-language", "ko")))
        .await;
        assert!(cookies.is_empty());
        assert_eq!(body, "English");
    }

    #[actix_web::test]
    async fn garbage_lang_cookie_falls_back_to_english_and_resets_cookie() {
        let (cookies, body) =
            run(test::TestRequest::get().cookie(actix_web::cookie::Cookie::new("lang", "zzz")))
                .await;
        assert!(cookies.iter().any(|c| c.starts_with("lang=eng")));
        assert_eq!(body, "English");
    }
}
