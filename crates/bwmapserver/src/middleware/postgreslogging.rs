use std::{
    future::{ready, Ready},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use super::{usersession::UserSession, TraceID};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, HttpMessage,
};
use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use bwcommon::{do_mixpanel_stuff, ApiSpecificInfoForLogging, TrackingAnalytics};
use futures_util::{future::LocalBoxFuture, FutureExt};

// use tracing::info;

pub struct PostgresLoggingTransformer;

impl<S, B> Transform<S, ServiceRequest> for PostgresLoggingTransformer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PostgresLoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PostgresLoggingMiddleware { service }))
    }
}

pub struct PostgresLoggingMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for PostgresLoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let host = req.connection_info().host().to_owned();
        let real_addr = req
            .connection_info()
            .realip_remote_addr()
            .map(|x| Some(x.to_owned()));
        let remote_addr = req
            .connection_info()
            .peer_addr()
            .map(|x| Some(x.to_owned()));

        let tracking_analytics_id = req
            .extensions()
            .get::<TrackingAnalytics>()
            .map(|x| Some(x.tracking_analytics_id.clone()));
        let tracking_analytics_was_provided_by_request = req
            .extensions()
            .get::<TrackingAnalytics>()
            .map(|x| Some(x.was_provided_by_request));

        let trace_id = req
            .extensions()
            .get::<TraceID>()
            .map(|x| Some(x.id.clone()));

        let user_id = req.extensions().get::<UserSession>().map(|x| x.id);
        let user_username = req
            .extensions()
            .get::<UserSession>()
            .map(|x| x.username.clone());
        let user_token = req
            .extensions()
            .get::<UserSession>()
            .map(|x| x.token.clone());

        let path = req.path().to_owned();
        let query_string = req.query_string().to_owned();
        let method = req.method().to_string();
        let version = format!("{:?}", req.version());

        let header_map = req.headers();

        let user_agent = header_map
            .get("user-agent")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let if_modified_since = header_map
            .get("if-modified-since")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let if_none_match = header_map
            .get("if-none-match")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let sec_ch_ua = header_map
            .get("sec-ch-ua")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let sec_ch_ua_mobile = header_map
            .get("sec-ch-ua-mobile")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let sec_ch_ua_platform = header_map
            .get("sec-ch-ua-platform")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let accept_language = header_map
            .get("accept-language")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let accept_encoding = header_map
            .get("accept-encoding")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let accept = header_map
            .get("accept")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let cookies = header_map
            .get("cookie")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));
        let referer = header_map
            .get("referer")
            .and_then(|x| x.to_str().map_or(None, |x| Some(x.to_owned())));

        let pool = req
            .app_data::<Data<Pool<PostgresConnectionManager<NoTls>>>>()
            .unwrap()
            .clone();

        let start_time = req.extensions().get::<TraceID>().map(|x| x.start_time);

        let fut = self.service.call(req);

        async move {
            let res = fut.await;

            let request_time = start_time
                .map(|x| Instant::now().duration_since(x).as_micros() as i64);

            match res {
                Ok(res) => {
                    let status = res.response().status().as_u16() as i16;

                    let req_info = bwcommon::get_request_logging_info(res.request());
                    let properties = match res
                        .response()
                        .extensions()
                        .get::<ApiSpecificInfoForLogging>()
                        .cloned()
                    {
                        Some(properties) => properties,
                        None => ApiSpecificInfoForLogging {
                            ..Default::default()
                        },
                    };

                    let info = bwcommon::get_api_logging_info(req_info, properties);

                    let tx = (***(res.request()
                        .app_data::<actix_web::web::Data<std::sync::mpsc::Sender<serde_json::Value>>>()
                        .unwrap()))
                    .clone();

                    do_mixpanel_stuff(info, tx)?;

                    actix_web::rt::spawn(async move {
                        async move {
                            let con = pool.get().await?;

                            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

                            anyhow::ensure!(con.execute("INSERT INTO userlogs (log_time, host, ip_addr, remote_addr, tac, tracking_analytics_was_provided_by_request, trace_id, path, query_string, method, version, user_agent, request_time_us, status, if_modified_since, if_none_match, sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform, accept_language, accept_encoding, accept, cookies, user_id, user_username, user_token, referer) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)", &[&now, &host, &real_addr, &remote_addr, &tracking_analytics_id, &tracking_analytics_was_provided_by_request, &trace_id, &path, &query_string, &method, &version, &user_agent, &request_time, &status, &if_modified_since, &if_none_match, &sec_ch_ua, &sec_ch_ua_mobile, &sec_ch_ua_platform, &accept_language, &accept_encoding, &accept, &cookies, &user_id, &user_username, &user_token, &referer])
                                .await? == 1);

                            anyhow::Ok(())
                        }.await.unwrap()
                    });

                    Ok(res)
                }
                Err(err) => {
                    let err_string = err.to_string();

                    actix_web::rt::spawn(async move {
                        async move {
                            let con = pool.get().await?;

                            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

                            anyhow::ensure!(con.execute("INSERT INTO userlogs (log_time, host, ip_addr, remote_addr, tac, tracking_analytics_was_provided_by_request, trace_id, path, query_string, method, version, user_agent, error, if_modified_since, if_none_match, sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform, accept_language, accept_encoding, accept, cookies, request_time_us, user_id, user_username, user_token, referer) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)", &[&now, &host, &real_addr, &remote_addr, &tracking_analytics_id, &tracking_analytics_was_provided_by_request, &trace_id, &path, &query_string, &method, &version, &user_agent, &err_string, &if_modified_since, &if_none_match, &sec_ch_ua, &sec_ch_ua_mobile, &sec_ch_ua_platform, &accept_language, &accept_encoding, &accept, &cookies, &request_time, &user_id, &user_username, &user_token, &referer]).await? == 1);

                            anyhow::Ok(())
                        }.await.unwrap()
                    });

                    Err(err)
                }
            }
        }
        .boxed_local()
    }
}
