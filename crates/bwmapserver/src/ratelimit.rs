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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_burst_then_denies() {
        let limiter = UsernameLoginLimiter::new();
        // Quota allows a burst of 10 for a given username.
        for i in 0..10 {
            assert!(
                limiter.check("neo").is_ok(),
                "attempt {i} should be allowed within the burst"
            );
        }
        // The 11th immediate attempt is rejected.
        let err = limiter
            .check("neo")
            .expect_err("11th attempt must be denied");
        assert_eq!(err.status(), StatusCode::TOO_MANY_REQUESTS);
        // Retry-After is present and floored to at least 1 second.
        let retry = err
            .headers()
            .get(RETRY_AFTER)
            .expect("Retry-After header present")
            .to_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert!(retry >= 1, "Retry-After floored to >= 1s, got {retry}");
    }

    #[test]
    fn usernames_are_independent() {
        let limiter = UsernameLoginLimiter::new();
        for _ in 0..10 {
            limiter.check("alice").unwrap();
        }
        // alice is now throttled, but bob is unaffected.
        assert!(limiter.check("alice").is_err());
        assert!(limiter.check("bob").is_ok());
    }

    #[test]
    fn too_many_requests_floors_retry_after() {
        let resp = too_many_requests(Duration::from_millis(10));
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            resp.headers().get(RETRY_AFTER).unwrap().to_str().unwrap(),
            "1",
            "sub-second waits are floored to 1s"
        );

        let resp = too_many_requests(Duration::from_secs(42));
        assert_eq!(
            resp.headers().get(RETRY_AFTER).unwrap().to_str().unwrap(),
            "42"
        );
    }
}
