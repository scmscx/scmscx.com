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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::routing::get;
    use axum::{Extension, Router};
    use tower::ServiceExt;

    /// Reports the trace id assigned by the middleware, or "none".
    async fn echo_trace(trace: Option<Extension<TraceID>>) -> String {
        trace.map_or_else(|| "none".to_string(), |Extension(t)| t.id)
    }

    async fn run() -> Response {
        let app = Router::new()
            .route("/", get(echo_trace))
            .layer(axum::middleware::from_fn(trace_id));
        app.oneshot(
            axum::http::Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn body(res: Response) -> String {
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn assigns_six_char_trace_id_visible_to_handler() {
        let res = run().await;
        assert_eq!(res.status(), StatusCode::OK);
        let id = body(res).await;
        assert_eq!(id.len(), 6, "trace id is truncated to 6 chars, got {id:?}");
        assert!(
            id.chars().all(|c| c.is_ascii_hexdigit()),
            "trace id should be hex, got {id:?}"
        );
    }

    #[tokio::test]
    async fn trace_ids_are_unique_per_request() {
        let a = body(run().await).await;
        let b = body(run().await).await;
        assert_ne!(a, b, "each request must get a fresh trace id");
    }

    #[tokio::test]
    async fn preserves_downstream_status() {
        async fn teapot() -> StatusCode {
            StatusCode::IM_A_TEAPOT
        }
        let app = Router::new()
            .route("/", get(teapot))
            .layer(axum::middleware::from_fn(trace_id));
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::IM_A_TEAPOT);
    }
}
