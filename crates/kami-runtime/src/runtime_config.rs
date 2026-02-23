//! Configuration for the KAMI runtime.

use crate::rate_limiter::RateLimitConfig;

/// Configuration for the KAMI runtime.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Component cache size.
    pub cache_size: usize,
    /// Scheduler concurrency limit.
    pub max_concurrent: usize,
    /// Enable epoch interruption for timeout.
    pub epoch_interruption: bool,
    /// Rate limiter configuration.
    pub rate_limit: RateLimitConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            cache_size: 32,
            max_concurrent: 4,
            epoch_interruption: true,
            rate_limit: RateLimitConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = RuntimeConfig::default();
        assert_eq!(cfg.cache_size, 32);
        assert_eq!(cfg.max_concurrent, 4);
        assert!(cfg.epoch_interruption);
        assert_eq!(cfg.rate_limit.per_tool, 100);
    }

    #[test]
    fn config_is_cloneable() {
        let cfg = RuntimeConfig {
            cache_size: 64,
            max_concurrent: 8,
            epoch_interruption: false,
            rate_limit: RateLimitConfig::default(),
        };
        let copy = cfg.clone();
        assert_eq!(copy.cache_size, 64);
        assert!(!copy.epoch_interruption);
    }
}
