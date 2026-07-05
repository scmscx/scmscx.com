use crate::common::MyError;

use reqwest::StatusCode;
use serde::Serialize;
use tokio::time::sleep;
use tracing::error;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LangData {
    English,
    Korean,
}

fn merge_json(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (a @ &mut serde_json::Value::Object(_), serde_json::Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge_json(a.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

pub async fn create_mixpanel_channel() -> std::sync::mpsc::Sender<serde_json::Value> {
    let (tx, rx) = std::sync::mpsc::channel::<serde_json::Value>();

    tokio::spawn(async move {
        let client = reqwest::Client::new();

        loop {
            sleep(std::time::Duration::from_secs(10)).await;

            let mut events = Vec::new();

            loop {
                let result = rx.try_recv();

                match result {
                    Ok(v) => {
                        events.push(v);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        if events.is_empty() {
                            return;
                        }
                    }
                }

                if events.len() > 1900 {
                    break;
                }
            }

            if !events.is_empty() && std::env::var("MIXPANEL_DISABLED").is_err() {
                for _ in 0..5 {
                    let ret = client
                        .post("https://api.mixpanel.com/import")
                        .basic_auth(
                            std::env::var("MIXPANEL_ACCOUNT_NAME").unwrap(),
                            Some(std::env::var("MIXPANEL_API_KEY").unwrap()),
                        )
                        .query(&[
                            ("strict", 1),
                            (
                                "project_id",
                                std::env::var("MIXPANEL_PROJECT_ID")
                                    .unwrap()
                                    .parse()
                                    .unwrap(),
                            ),
                        ])
                        .json(&events)
                        .send()
                        .await;

                    match ret {
                        Ok(ret) => {
                            if ret.status() == StatusCode::OK {
                                break;
                            }
                            error!(
                                "error from mixpanel. status: {}, body: {}",
                                ret.status(),
                                ret.text()
                                    .await
                                    .unwrap_or_else(|_| "failed to unwrap body".to_string())
                            );
                            sleep(std::time::Duration::from_secs(5)).await;
                        }
                        Err(err) => {
                            error!("error sending stuff to mixpanel: {err:?}");
                            sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    });

    tx
}

pub fn do_mixpanel_stuff(
    log_info: ApiLoggingInfo,
    tx: std::sync::mpsc::Sender<serde_json::Value>,
) -> Result<(), MyError> {
    let properties = serde_json::to_value(log_info.clone()).unwrap();

    let mut mixpanel = serde_json::json!({
        "distinct_id": log_info.req_info.tac,
        "$insert_id": uuid::Uuid::new_v4().as_simple().to_string(),
    });

    merge_json(&mut mixpanel, properties);

    let data = serde_json::json!({
        "event": log_info.req_info.event,
        "properties": mixpanel
    });

    Ok(tx.send(data)?)
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ApiSpecificInfoForLogging {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub map_id: Option<i64>,
    pub replay_id: Option<i64>,
    pub chk_hash: Option<String>,
    pub mapblob_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiRequestLoggingInfo {
    pub ip: Option<String>,
    pub tac: Option<String>,
    pub event: Option<String>,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub referer: String,
    pub accept_language: String,
    pub accept_encoding: String,
    pub user_agent: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiLoggingInfo {
    pub time: i64,
    #[serde(flatten)]
    pub req_info: ApiRequestLoggingInfo,

    #[serde(flatten)]
    pub properties: ApiSpecificInfoForLogging,
}

#[derive(Clone, Debug)]
pub struct TrackingAnalytics {
    pub tracking_analytics_id: String,
    pub was_provided_by_request: bool,
}

pub fn get_api_logging_info(
    req_info: ApiRequestLoggingInfo,
    properties: ApiSpecificInfoForLogging,
) -> ApiLoggingInfo {
    let start = std::time::SystemTime::now();
    let now_ms = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");

    ApiLoggingInfo {
        time: now_ms.as_millis() as i64,
        req_info,
        properties,
    }
}

// --- response logging helpers ------------------------------------------------

/// axum equivalent of `insert_extension`: render `body` into a response and
/// stash the per-request logging info in the response's extensions, where the
/// postgres-logging middleware reads it back out.
pub fn with_logging_info<T: axum::response::IntoResponse>(
    info: ApiSpecificInfoForLogging,
    body: T,
) -> axum::response::Response {
    let mut resp = axum::response::IntoResponse::into_response(body);
    resp.extensions_mut().insert(info);
    resp
}

fn header_str<'a>(headers: &'a http::HeaderMap, name: &str) -> &'a str {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
}

/// axum equivalent of `get_request_logging_info`. `ip` and `event` (the matched
/// route pattern) are resolved by the caller since they aren't available on the
/// bare request `Parts` (they come from a real-IP helper and `MatchedPath`).
pub fn get_request_logging_info_from_parts(
    parts: &http::request::Parts,
    ip: Option<String>,
    event: Option<String>,
) -> ApiRequestLoggingInfo {
    let headers = &parts.headers;

    let tac = parts
        .extensions
        .get::<TrackingAnalytics>()
        .map(|tac| tac.tracking_analytics_id.clone());

    ApiRequestLoggingInfo {
        ip,
        tac,
        event: event.or_else(|| Some(parts.uri.path().to_owned())),
        method: parts.method.to_string(),
        path: parts.uri.path().to_string(),
        query: parts.uri.query().map(std::string::ToString::to_string),
        referer: header_str(headers, "referer").to_string(),
        accept_language: header_str(headers, "accept-language").to_string(),
        accept_encoding: header_str(headers, "accept-encoding").to_string(),
        user_agent: header_str(headers, "user-agent").to_string(),
    }
}
