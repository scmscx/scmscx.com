mod defaultnostore;
mod endtoendduration;
mod etag;
mod immutableassets;
mod language;
mod metrics;
mod postgreslogging;
mod traceid;
mod trackinganalytics;
mod usersession;

use axum::http::header;
use axum::response::Response;

/// Whether a response has opted into shared/CDN caching, signalled by a `public`
/// `Cache-Control`. Per-user cookies must not be attached to such responses: a
/// `Set-Cookie` makes Cloudflare bypass its cache entirely, and caching one would
/// bake a single visitor's `tac`/`lang2` into the copy served to everyone else.
/// The cookie-setting middlewares consult this before appending their `Set-Cookie`.
///
/// Keyed on `public` (not `immutable`) so it also covers the shorter-lived but
/// still shared-cacheable minimap responses; `private` responses are deliberately
/// excluded since a shared cache won't store them anyway.
pub(crate) fn response_is_publicly_cacheable(res: &Response) -> bool {
    res.headers()
        .get(header::CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("public"))
}

/// Whether a response is a server-rendered page rather than an asset or an API
/// reply. Lives here beside `response_is_publicly_cacheable` because two
/// middlewares ask the question -- `etag` to decide what is worth hashing, and
/// `language` to decide what varies by language -- and they have to agree: a
/// definition that drifts in one would leave the two disagreeing about which
/// responses are pages.
pub(crate) fn response_is_html(res: &Response) -> bool {
    res.headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/html"))
}

pub use traceid::trace_id;
pub use traceid::TraceID;

pub use trackinganalytics::tracking_analytics;

pub use endtoendduration::end_to_end_duration;

pub use language::language;
pub use language::Language;
pub use language::LANG_COOKIE;

pub use metrics::metrics;

pub use postgreslogging::postgres_logging;

pub use usersession::user_session;
pub use usersession::UserSession;

pub use defaultnostore::default_no_store;

pub use etag::etag;

pub use immutableassets::immutable_assets;
