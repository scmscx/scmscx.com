use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use bwcommon::{LoggedError, TrackingAnalytics};

use super::{usersession::UserSession, TraceID};
use crate::webutil::{realip, Pool};

/// Captures per-request metadata, then (after the handler runs) writes a row to
/// `userlogs` on a background task. Mirrors the old actix
/// `PostgresLoggingTransformer`.
pub async fn postgres_logging(pool: Pool, req: Request, next: Next) -> Response {
    let headers = req.headers();

    let header_opt = |name: &str| -> Option<String> {
        headers
            .get(name)
            .and_then(|x| x.to_str().ok())
            .map(std::string::ToString::to_string)
    };

    let peer = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0);

    let host = header_opt("host").unwrap_or_default();
    let real_addr = realip(headers, peer);
    let remote_addr = peer.map(|x| x.to_string());

    let ta = req.extensions().get::<TrackingAnalytics>();
    let tracking_analytics_id = ta.map(|x| x.tracking_analytics_id.clone());
    let tracking_analytics_was_provided_by_request = ta.map(|x| x.was_provided_by_request);

    let trace = req.extensions().get::<TraceID>();
    let trace_id = trace.map(|x| x.id.clone());
    let start_time = trace.map(|x| x.start_time);

    let user = req.extensions().get::<UserSession>();
    let user_id = user.map(|x| x.id);
    let user_username = user.map(|x| x.username.clone());
    let user_token = user.map(|x| x.token.clone());

    let path = req.uri().path().to_owned();
    let query_string = req.uri().query().unwrap_or("").to_owned();
    let method = req.method().to_string();
    let version = format!("{:?}", req.version());

    let user_agent = header_opt("user-agent");
    let if_modified_since = header_opt("if-modified-since");
    let if_none_match = header_opt("if-none-match");
    let sec_ch_ua = header_opt("sec-ch-ua");
    let sec_ch_ua_mobile = header_opt("sec-ch-ua-mobile");
    let sec_ch_ua_platform = header_opt("sec-ch-ua-platform");
    let accept_language = header_opt("accept-language");
    let accept_encoding = header_opt("accept-encoding");
    let accept = header_opt("accept");
    let cookies = header_opt("cookie");
    let referer = header_opt("referer");

    let res = next.run(req).await;

    let request_time = start_time.map(|x| Instant::now().duration_since(x).as_micros() as i64);

    let status = res.status().as_u16() as i16;
    let error = res.extensions().get::<LoggedError>().map(|e| e.0.clone());

    tokio::spawn(async move {
        let result = async move {
            let con = pool.get().await?;
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

            if let Some(err_string) = error {
                anyhow::ensure!(con.execute("INSERT INTO userlogs (log_time, host, ip_addr, remote_addr, tac, tracking_analytics_was_provided_by_request, trace_id, path, query_string, method, version, user_agent, error, if_modified_since, if_none_match, sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform, accept_language, accept_encoding, accept, cookies, request_time_us, user_id, user_username, user_token, referer) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)", &[&now, &host, &real_addr, &remote_addr, &tracking_analytics_id, &tracking_analytics_was_provided_by_request, &trace_id, &path, &query_string, &method, &version, &user_agent, &err_string, &if_modified_since, &if_none_match, &sec_ch_ua, &sec_ch_ua_mobile, &sec_ch_ua_platform, &accept_language, &accept_encoding, &accept, &cookies, &request_time, &user_id, &user_username, &user_token, &referer]).await? == 1);
            } else {
                anyhow::ensure!(con.execute("INSERT INTO userlogs (log_time, host, ip_addr, remote_addr, tac, tracking_analytics_was_provided_by_request, trace_id, path, query_string, method, version, user_agent, request_time_us, status, if_modified_since, if_none_match, sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform, accept_language, accept_encoding, accept, cookies, user_id, user_username, user_token, referer) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)", &[&now, &host, &real_addr, &remote_addr, &tracking_analytics_id, &tracking_analytics_was_provided_by_request, &trace_id, &path, &query_string, &method, &version, &user_agent, &request_time, &status, &if_modified_since, &if_none_match, &sec_ch_ua, &sec_ch_ua_mobile, &sec_ch_ua_platform, &accept_language, &accept_encoding, &accept, &cookies, &user_id, &user_username, &user_token, &referer]).await? == 1);
            }

            anyhow::Ok(())
        }
        .await;

        if let Err(err) = result {
            tracing::error!("failed to write userlog: {err:?}");
        }
    });

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    /// A pool that never connects (port 1). The background userlog INSERT fails
    /// harmlessly (it's a detached, best-effort task); the response passthrough —
    /// the part we assert — doesn't depend on it.
    fn dead_pool() -> Pool {
        let manager = bb8_postgres::PostgresConnectionManager::new(
            "host=127.0.0.1 port=1 user=x dbname=x".parse().unwrap(),
            bb8_postgres::tokio_postgres::NoTls,
        );
        bb8_postgres::bb8::Pool::builder().build_unchecked(manager)
    }

    fn app_with(router: Router) -> Router {
        let pool = dead_pool();
        router.layer(axum::middleware::from_fn(move |req, next| {
            postgres_logging(pool.clone(), req, next)
        }))
    }

    async fn drive(app: Router, uri: &str) -> Response {
        app.oneshot(
            axum::http::Request::builder()
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn successful_response_passes_through() {
        let app = app_with(Router::new().route("/api/ok", get(|| async { "ok" })));

        let res = drive(app, "/api/ok").await;
        assert_eq!(res.status(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn error_response_passes_through() {
        async fn boom() -> Result<&'static str, bwcommon::MyError> {
            Err("boom".into())
        }
        let app = app_with(Router::new().route("/api/boom", get(boom)));

        let res = drive(app, "/api/boom").await;
        // The MyError → 500 path stamps a LoggedError extension, which the
        // middleware records; the client still gets the 500.
        assert_eq!(res.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}
