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

pub fn per_ip_register_governor_config(
) -> GovernorConfig<RealIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    GovernorConfigBuilder::default()
        .key_extractor(RealIpKeyExtractor)
        .seconds_per_request(60 * 20)
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
