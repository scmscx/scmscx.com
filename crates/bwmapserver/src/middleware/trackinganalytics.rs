use std::future::{ready, Ready};

use actix_web::{
    cookie::{Cookie, SameSite},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderValue,
    Error, HttpMessage,
};
use bwcommon::TrackingAnalytics;
use futures_util::{future::LocalBoxFuture, FutureExt};
use sha2::{Digest, Sha256};
use tracing::{info, instrument, Instrument};

pub struct TrackingAnalyticsTransformer;

impl<S, B> Transform<S, ServiceRequest> for TrackingAnalyticsTransformer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TrackingAnalyticsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TrackingAnalyticsMiddleware { service }))
    }
}

pub struct TrackingAnalyticsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TrackingAnalyticsMiddleware<S>
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
        let (tracking_analytics_id, was_provided_by_request) = if let Some(cookie) =
            req.cookie("tac")
        {
            let tac = cookie.value().to_string();

            (tac, true)
        } else {
            const SALT: &[u8] =
                b"this is a salt so that nobody can guess the content of these hashes";
            let user_agent = req
                .headers()
                .get("user-agent")
                .unwrap_or(&HeaderValue::from_static("default"))
                .clone();
            let accept_language = req
                .headers()
                .get("accept-language")
                .unwrap_or(&HeaderValue::from_static("default"))
                .clone();
            let sec_ch_ua = req
                .headers()
                .get("sec-ch-ua")
                .unwrap_or(&HeaderValue::from_static("default"))
                .clone();
            let sec_ch_mobile = req
                .headers()
                .get("sec-ch-mobile")
                .unwrap_or(&HeaderValue::from_static("default"))
                .clone();
            let sec_ch_platform = req
                .headers()
                .get("sec-ch-ua-platform")
                .unwrap_or(&HeaderValue::from_static("default"))
                .clone();
            let ip_address = req
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("default:5000")
                .split(':')
                .next()
                .unwrap_or("default")
                .to_owned();
            info!("tac created. user_agent: {user_agent:?}, accept_language: {accept_language:?}, sec_ch_ua: {sec_ch_ua:?}, sec_ch_mobile: {sec_ch_mobile:?}, sec_ch_platform: {sec_ch_platform:?}, ip_address: {ip_address:?}");
            let mut hasher = Sha256::new();
            hasher.update(SALT);
            hasher.update(ip_address);
            hasher.update(user_agent);
            hasher.update(accept_language);
            hasher.update(sec_ch_ua);
            hasher.update(sec_ch_mobile);
            hasher.update(sec_ch_platform);
            (format!("{:x}", hasher.finalize()), false)
        };

        req.extensions_mut().insert(TrackingAnalytics {
            tracking_analytics_id: tracking_analytics_id.clone(),
            was_provided_by_request,
        });

        let tracking_analytics_id_clone = tracking_analytics_id.clone();

        info!("tac: {}", tracking_analytics_id);

        let fut = self.service.call(req);
        async move {
            let mut res = fut.await?;

            if !was_provided_by_request {
                res.response_mut()
                    .add_cookie(
                        &Cookie::build("tac", &tracking_analytics_id_clone)
                            .path("/")
                            .same_site(SameSite::Lax)
                            .secure(true)
                            .http_only(false)
                            .permanent()
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

    /// Reports "<id>|<was_provided_by_request>".
    async fn echo_tac(req: HttpRequest) -> HttpResponse {
        let out = req
            .extensions()
            .get::<TrackingAnalytics>()
            .map(|t| format!("{}|{}", t.tracking_analytics_id, t.was_provided_by_request));
        HttpResponse::Ok().body(out.unwrap_or_else(|| "none".to_string()))
    }

    async fn run(req: test::TestRequest) -> (Vec<String>, String) {
        let app = test::init_service(
            App::new()
                .wrap(TrackingAnalyticsTransformer)
                .default_service(web::to(echo_tac)),
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
    async fn reuses_existing_tac_cookie_without_resetting() {
        let (cookies, body) = run(test::TestRequest::get()
            .cookie(actix_web::cookie::Cookie::new("tac", "existing-id-123")))
        .await;
        assert!(
            cookies.is_empty(),
            "an already-provided tac must not be re-set"
        );
        assert_eq!(body, "existing-id-123|true");
    }

    #[actix_web::test]
    async fn generates_tac_when_absent_and_sets_cookie() {
        let (cookies, body) =
            run(test::TestRequest::get().insert_header(("user-agent", "UA/1.0"))).await;
        assert!(
            cookies.iter().any(|c| c.starts_with("tac=")),
            "a fresh tac cookie must be set, got {cookies:?}"
        );
        let (id, provided) = body.split_once('|').unwrap();
        assert_eq!(provided, "false");
        // The generated id is a SHA-256 hex digest.
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[actix_web::test]
    async fn generated_tac_is_deterministic_for_same_fingerprint() {
        let build = || {
            test::TestRequest::get()
                .insert_header(("user-agent", "Mozilla/5.0"))
                .insert_header(("accept-language", "en-US"))
                .insert_header(("sec-ch-ua-platform", "\"Linux\""))
        };
        let (_, a) = run(build()).await;
        let (_, b) = run(build()).await;
        assert_eq!(a, b, "same fingerprint headers must hash to the same tac");
    }

    #[actix_web::test]
    async fn different_user_agent_yields_different_tac() {
        let (_, a) =
            run(test::TestRequest::get().insert_header(("user-agent", "Mozilla/5.0"))).await;
        let (_, b) = run(test::TestRequest::get().insert_header(("user-agent", "curl/8.0"))).await;
        assert_ne!(a, b, "distinct fingerprints must produce distinct tacs");
    }
}
