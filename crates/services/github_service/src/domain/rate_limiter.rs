use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_made: u32,
    pub window_start: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_duration: Duration,
}