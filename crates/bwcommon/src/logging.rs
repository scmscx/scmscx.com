use crate::common::MyError;

use actix_web::HttpMessage;
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

pub fn get_header<'a>(req: &'a actix_web::HttpRequest, name: &str) -> &'a str {
    if let Some(v) = req.headers().get(name) {
        v.to_str().unwrap_or("")
    } else {
        ""
    }
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

pub fn get_request_logging_info(req: &actix_web::HttpRequest) -> ApiRequestLoggingInfo {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .map(std::string::ToString::to_string);

    let query = req.uri().query().map(std::string::ToString::to_string);

    let tac = req
        .extensions()
        .get::<TrackingAnalytics>()
        .map(|tac| tac.tracking_analytics_id.clone());

    ApiRequestLoggingInfo {
        ip,
        tac,
        event: Some(req.match_pattern().map_or_else(
            || req.path().to_owned(),
            |x| {
                if x.is_empty() {
                    req.path().to_owned()
                } else {
                    x
                }
            },
        )),
        method: req.method().to_string(),
        path: req.uri().path().to_string(),
        query,
        referer: get_header(req, "referer").to_string(),
        accept_language: get_header(req, "accept-language").to_string(),
        accept_encoding: get_header(req, "accept-encoding").to_string(),
        user_agent: get_header(req, "user-agent").to_string(),
    }
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

pub fn insert_extension(
    mut resp: actix_web::HttpResponseBuilder,
    info: ApiSpecificInfoForLogging,
) -> actix_web::HttpResponseBuilder {
    resp.extensions_mut().insert(info);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::Method;
    use actix_web::test::TestRequest;
    use actix_web::HttpResponse;

    #[tokio::test]
    async fn insert_extension_stashes_logging_info_and_keeps_body() {
        let info = ApiSpecificInfoForLogging {
            map_id: Some(42),
            username: Some("neo".to_string()),
            ..Default::default()
        };

        let resp = insert_extension(HttpResponse::Ok(), info).body("hello body");

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        let stashed = resp
            .extensions()
            .get::<ApiSpecificInfoForLogging>()
            .cloned()
            .expect("logging info must be present in the response extensions");
        assert_eq!(stashed.map_id, Some(42));
        assert_eq!(stashed.username.as_deref(), Some("neo"));

        let bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(&bytes[..], b"hello body");
    }

    #[test]
    fn request_logging_info_maps_fields() {
        // x-forwarded-for feeds actix's realip_remote_addr(), the same client-IP
        // resolution the rate limiter and logging rely on.
        let req = TestRequest::default()
            .method(Method::POST)
            .uri("/api/thing?q=1&z=2")
            .insert_header(("referer", "https://ref"))
            .insert_header(("accept-language", "ko"))
            .insert_header(("accept-encoding", "gzip"))
            .insert_header(("user-agent", "UA/1.0"))
            .insert_header(("x-forwarded-for", "1.2.3.4"))
            .to_http_request();

        let info = get_request_logging_info(&req);

        assert_eq!(info.ip.as_deref(), Some("1.2.3.4"));
        assert_eq!(info.method, "POST");
        assert_eq!(info.path, "/api/thing");
        assert_eq!(info.query.as_deref(), Some("q=1&z=2"));
        // No matched route in a bare TestRequest → event falls back to the path.
        assert_eq!(info.event.as_deref(), Some("/api/thing"));
        assert_eq!(info.referer, "https://ref");
        assert_eq!(info.accept_language, "ko");
        assert_eq!(info.accept_encoding, "gzip");
        assert_eq!(info.user_agent, "UA/1.0");
        // No tac extension → None.
        assert!(info.tac.is_none());
    }

    #[test]
    fn request_logging_info_event_falls_back_to_path_and_picks_up_tac() {
        let req = TestRequest::default().uri("/raw/path").to_http_request();
        req.extensions_mut().insert(TrackingAnalytics {
            tracking_analytics_id: "tac-123".to_string(),
            was_provided_by_request: true,
        });

        let info = get_request_logging_info(&req);

        assert_eq!(info.event.as_deref(), Some("/raw/path"));
        assert_eq!(info.tac.as_deref(), Some("tac-123"));
        // Missing headers become empty strings, and no query → None.
        assert_eq!(info.referer, "");
        assert!(info.query.is_none());
    }

    #[test]
    fn api_logging_info_serializes_flattened() {
        let req_info = ApiRequestLoggingInfo {
            ip: Some("9.9.9.9".to_string()),
            tac: Some("t".to_string()),
            event: Some("/e".to_string()),
            method: "GET".to_string(),
            path: "/e".to_string(),
            query: None,
            referer: String::new(),
            accept_language: String::new(),
            accept_encoding: String::new(),
            user_agent: String::new(),
        };
        let props = ApiSpecificInfoForLogging {
            map_id: Some(7),
            ..Default::default()
        };

        let info = get_api_logging_info(req_info, props);
        let v = serde_json::to_value(&info).unwrap();

        // #[serde(flatten)] lifts req_info + properties fields to the top level.
        assert_eq!(v["method"], "GET");
        assert_eq!(v["ip"], "9.9.9.9");
        assert_eq!(v["map_id"], 7);
        assert!(v.get("time").is_some());
        assert!(v.get("req_info").is_none());
        assert!(v.get("properties").is_none());
    }
}
