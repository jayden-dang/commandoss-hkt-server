use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::warn;

pub struct RateLimiterImpl {
    limits: Arc<Mutex<HashMap<String, RateLimit>>>,
    max_requests: u32,
    window_duration: Duration,
}

struct RateLimit {
    requests_made: u32,
    window_start: Instant,
}

impl RateLimiterImpl {
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration,
        }
    }

    pub async fn check_limit(&self, key: &str) -> Result<()> {
        let mut limits = self.limits.lock().await;
        let now = Instant::now();

        match limits.get_mut(key) {
            Some(limit) => {
                // Check if window has expired
                if now.duration_since(limit.window_start) >= self.window_duration {
                    limit.requests_made = 1;
                    limit.window_start = now;
                    Ok(())
                } else if limit.requests_made < self.max_requests {
                    limit.requests_made += 1;
                    Ok(())
                } else {
                    let retry_after = self.window_duration - now.duration_since(limit.window_start);
                    warn!(
                        "Rate limit exceeded for key: {}. Retry after: {:?}",
                        key, retry_after
                    );
                    Err(Error::RateLimitExceeded {
                        retry_after_seconds: retry_after.as_secs(),
                    })
                }
            }
            None => {
                limits.insert(
                    key.to_string(),
                    RateLimit {
                        requests_made: 1,
                        window_start: now,
                    },
                );
                Ok(())
            }
        }
    }

    pub async fn reset_limit(&self, key: &str) {
        let mut limits = self.limits.lock().await;
        limits.remove(key);
    }

    pub async fn get_remaining_requests(&self, key: &str) -> u32 {
        let limits = self.limits.lock().await;
        match limits.get(key) {
            Some(limit) => {
                let now = Instant::now();
                if now.duration_since(limit.window_start) >= self.window_duration {
                    self.max_requests
                } else {
                    self.max_requests.saturating_sub(limit.requests_made)
                }
            }
            None => self.max_requests,
        }
    }
}