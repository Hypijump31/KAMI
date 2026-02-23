//! Token-bucket rate limiter for tool execution.
//!
//! Provides per-tool and global rate limits to prevent abuse.
//! Uses an atomic token bucket that refills over a configurable window.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use kami_types::ToolId;

/// Configuration for the rate limiter.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window per tool (0 = unlimited).
    pub per_tool: u64,
    /// Maximum global requests per window (0 = unlimited).
    pub global: u64,
    /// Sliding window duration.
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            per_tool: 100,
            global: 1000,
            window: Duration::from_secs(60),
        }
    }
}

/// Token bucket for a single rate-limit counter.
#[derive(Debug)]
struct TokenBucket {
    tokens: u64,
    capacity: u64,
    last_refill: Instant,
    window: Duration,
}

impl TokenBucket {
    fn new(capacity: u64, window: Duration) -> Self {
        Self {
            tokens: capacity,
            capacity,
            last_refill: Instant::now(),
            window,
        }
    }

    /// Tries to consume one token. Returns `true` if allowed.
    fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed();
        if elapsed >= self.window {
            self.tokens = self.capacity;
            self.last_refill = Instant::now();
        }
    }
}

/// Rate limiter with per-tool and global limits.
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    global: Mutex<TokenBucket>,
    per_tool: Mutex<HashMap<ToolId, TokenBucket>>,
}

impl RateLimiter {
    /// Creates a new rate limiter with the given configuration.
    pub fn new(config: &RateLimitConfig) -> Self {
        Self {
            config: config.clone(),
            global: Mutex::new(TokenBucket::new(config.global, config.window)),
            per_tool: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the current rate-limit configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Checks if a request for the given tool is allowed.
    ///
    /// Returns `true` if the request is within both the per-tool and
    /// global rate limits.
    pub fn check(&self, tool_id: &ToolId) -> bool {
        if self.config.global == 0 && self.config.per_tool == 0 {
            return true;
        }
        if self.config.global > 0 {
            let mut global = self.global.lock().unwrap_or_else(|e| e.into_inner());
            if !global.try_acquire() {
                return false;
            }
        }
        if self.config.per_tool > 0 {
            let mut map = self.per_tool.lock().unwrap_or_else(|e| e.into_inner());
            let bucket = map
                .entry(tool_id.clone())
                .or_insert_with(|| TokenBucket::new(self.config.per_tool, self.config.window));
            if !bucket.try_acquire() {
                return false;
            }
        }
        true
    }
}
