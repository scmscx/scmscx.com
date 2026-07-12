use std::num::NonZeroU32;
use std::sync::OnceLock;
use std::time::Duration;

use actix_governor::{
    GovernorConfig, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError,
};
use actix_web::dev::ServiceRequest;
use actix_web::http::header::RETRY_AFTER;
use actix_web::HttpResponse;
use governor::clock::{Clock, DefaultClock, QuantaInstant};
use governor::middleware::NoOpMiddleware;
use governor::state::keyed::DashMapStateStore;
use governor::{Quota, RateLimiter};

type KeyedRateLimiter<K> = RateLimiter<K, DashMapStateStore<K>, DefaultClock>;

/// Honors `X-Forwarded-For` / `Forwarded` from our reverse proxy via
/// `realip_remote_addr()`, matching the pattern in middleware/postgreslogging.rs.
#[derive(Clone)]
pub struct RealIpKeyExtractor;

impl KeyExtractor for RealIpKeyExtractor {
    type Key = String;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        let conn = req.connection_info();
        let ip = conn
            .realip_remote_addr()
            .or_else(|| conn.peer_addr())
            .ok_or_else(|| SimpleKeyExtractionError::new("could not determine client IP"))?;
        Ok(ip.to_owned())
    }
}

pub fn per_ip_login_governor_config(
) -> GovernorConfig<RealIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    GovernorConfigBuilder::default()
        .key_extractor(RealIpKeyExtractor)
        .seconds_per_request(3)
        .burst_size(20)
        .finish()
        .expect("valid governor config")
}

/// One registration per 20 minutes (per IP) once the burst is spent. Extracted as
/// a named constant so its value is unit-testable — the finished `GovernorConfig`
/// exposes no getter, and the replenish period isn't observable in a fast test.
pub(crate) const REGISTER_SECONDS_PER_REQUEST: u64 = 60 * 20;

pub fn per_ip_register_governor_config(
) -> GovernorConfig<RealIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    GovernorConfigBuilder::default()
        .key_extractor(RealIpKeyExtractor)
        .seconds_per_request(REGISTER_SECONDS_PER_REQUEST)
        .burst_size(3)
        .finish()
        .expect("valid governor config")
}

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

    pub fn check(&self, username: &str) -> Result<(), HttpResponse> {
        match self.inner.check_key(&username.to_owned()) {
            Ok(()) => Ok(()),
            Err(not_until) => Err(too_many_requests(not_until.wait_time_from(clock().now()))),
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
fn too_many_requests(retry_after: Duration) -> HttpResponse {
    HttpResponse::TooManyRequests()
        .insert_header((RETRY_AFTER, retry_after.as_secs().max(1).to_string()))
        .body("Too many attempts. Please slow down.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    fn retry_after_secs(resp: &HttpResponse) -> u64 {
        resp.headers()
            .get(RETRY_AFTER)
            .expect("Retry-After header present")
            .to_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
    }

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
        // The 11th immediate attempt is rejected with a 429.
        let err = limiter
            .check("neo")
            .expect_err("11th attempt must be denied");
        assert_eq!(err.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(retry_after_secs(&err) >= 1, "Retry-After floored to >= 1s");
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
    fn register_replenish_period_is_twenty_minutes() {
        // The per-IP register limiter replenishes one slot every 20 minutes. This
        // pins the arithmetic (60 * 20) that builds it — the governor config itself
        // exposes no getter, so the constant is the only testable surface.
        assert_eq!(REGISTER_SECONDS_PER_REQUEST, 20 * 60);
        assert_eq!(REGISTER_SECONDS_PER_REQUEST, 1200);
    }

    #[test]
    fn too_many_requests_floors_retry_after() {
        let resp = too_many_requests(Duration::from_millis(10));
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(retry_after_secs(&resp), 1, "sub-second waits floor to 1s");

        let resp = too_many_requests(Duration::from_secs(42));
        assert_eq!(retry_after_secs(&resp), 42);
    }
}
