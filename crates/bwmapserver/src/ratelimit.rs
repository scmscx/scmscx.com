use std::num::NonZeroU32;
use std::sync::OnceLock;
use std::time::Duration;

use axum::http::header::RETRY_AFTER;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use governor::clock::{Clock, DefaultClock};
use governor::state::keyed::DashMapStateStore;
use governor::{Quota, RateLimiter};

type KeyedRateLimiter<K> = RateLimiter<K, DashMapStateStore<K>, DefaultClock>;

/// Per-username failure limiter for /api/login: 10 attempts per 15 min.
/// Counts every attempt — governor's GCRA model has no reset-on-success
/// hook, so successful logins still consume a cell. Used inline in the
/// login handler because the username lives in the JSON body and
/// middleware can't peek at that without buffering.
pub struct UsernameLoginLimiter {
    inner: KeyedRateLimiter<String>,
}

impl UsernameLoginLimiter {
    pub fn new() -> Self {
        let quota = Quota::with_period(Duration::from_secs(15 * 60 / 10))
            .expect("nonzero period")
            .allow_burst(NonZeroU32::new(10).expect("10 is nonzero"));
        Self {
            inner: RateLimiter::keyed(quota),
        }
    }

    pub fn check(&self, username: &str) -> Result<(), Box<Response>> {
        match self.inner.check_key(&username.to_owned()) {
            Ok(()) => Ok(()),
            Err(not_until) => Err(Box::new(too_many_requests(
                not_until.wait_time_from(clock().now()),
            ))),
        }
    }

    pub fn retain_recent(&self) {
        self.inner.retain_recent();
        self.inner.shrink_to_fit();
    }
}

fn clock() -> &'static DefaultClock {
    static CLOCK: OnceLock<DefaultClock> = OnceLock::new();
    CLOCK.get_or_init(DefaultClock::default)
}

/// Floor to 1s — governor can return a 0s wait, which makes Retry-After useless.
fn too_many_requests(retry_after: Duration) -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [(RETRY_AFTER, retry_after.as_secs().max(1).to_string())],
        "Too many attempts. Please slow down.",
    )
        .into_response()
}
