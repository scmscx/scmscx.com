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
