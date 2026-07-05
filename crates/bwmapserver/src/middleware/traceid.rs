use std::time::Instant;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::{error, info, warn, Instrument};

use crate::webutil::realip;

#[derive(Clone, Debug)]
pub struct TraceID {
    pub id: String,
    pub start_time: Instant,
}

/// Assigns a short trace id to each request (stashed in the request extensions
/// for downstream middleware) and logs the final status/path/ip/user-agent.
pub async fn trace_id(mut req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();

    let trace_id: String = uuid::Uuid::new_v4()
        .as_simple()
        .to_string()
        .chars()
        .take(6)
        .collect();

    let peer = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0);
    let ip = realip(req.headers(), peer).unwrap_or_else(|| "x.x.x.x".to_owned());

    let user_agent = req.headers().get("user-agent").map_or_else(
        || "couldn't unwrap2".to_string(),
        |x| x.to_str().unwrap_or("couldn't unwrap").to_owned(),
    );

    req.extensions_mut().insert(TraceID {
        id: trace_id.clone(),
        start_time: Instant::now(),
    });

    let span = tracing::info_span!("traceid-middleware", trace_id = %trace_id);

    async move {
        let res = next.run(req).await;
        let status = res.status();
        if status.is_success() || status.is_redirection() {
            info!(status = %status, %path, %ip, %user_agent);
        } else if status.is_client_error() {
            warn!(status = %status, %path, %ip, %user_agent);
        } else if status.is_server_error() {
            error!(status = %status, %path, %ip, %user_agent);
        } else {
            warn!(status = %status, %path, %ip, %user_agent);
        }
        res
    }
    .instrument(span)
    .await
}
