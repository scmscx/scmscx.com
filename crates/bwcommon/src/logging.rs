use crate::common::MyError;

use actix_web::HttpMessage;
use serde::Serialize;
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
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            let mut events = Vec::new();

            loop {
                let result = rx.try_recv();

                if events.len() > 1900 {
                    break;
                }

                match result {
                    Ok(v) => {
                        events.push(v);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        return;
                    }
                }
            }

            if events.len() > 0 {
                if std::env::var("MIXPANEL_DISABLED").is_err() {
                    let ret = reqwest::Client::new()
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

                    if let Err(err) = ret {
                        error!("error sending stuff to mixpanel: {err:?}");
                    } else if let Ok(ret) = ret {
                        if ret.status() != 200 {
                            error!("error from mixpanel: {}", ret.text().await.unwrap())
                        }
                    }

                    events.clear();
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
    let ip = match req.connection_info().realip_remote_addr() {
        Some(ip) => Some(ip.to_string()),
        None => None,
    };

    let query = match req.uri().query() {
        Some(query) => Some(query.to_string()),
        None => None,
    };

    let tac = match req.extensions().get::<TrackingAnalytics>() {
        Some(tac) => Some(tac.tracking_analytics_id.clone()),
        None => None,
    };

    ApiRequestLoggingInfo {
        ip,
        tac,
        event: Some(
            req.match_pattern()
                .map(|x| {
                    if x.is_empty() {
                        req.path().to_owned()
                    } else {
                        x
                    }
                })
                .unwrap_or(req.path().to_owned()),
        ),
        method: req.method().to_string(),
        path: req.uri().path().to_string(),
        query,
        referer: get_header(&req, "referer").to_string(),
        accept_language: get_header(&req, "accept-language").to_string(),
        accept_encoding: get_header(&req, "accept-encoding").to_string(),
        user_agent: get_header(&req, "user-agent").to_string(),
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
