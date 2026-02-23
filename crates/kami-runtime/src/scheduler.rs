//! Task scheduling with concurrency control and priorities.
//!
//! Uses a `tokio::sync::Semaphore` to limit concurrent WASM executions,
//! preventing resource exhaustion on the host.

use std::sync::Arc;

use tokio::sync::Semaphore;
use tracing::debug;

/// Configuration for the task scheduler.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum concurrent WASM executions.
    pub max_concurrent: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self { max_concurrent: 4 }
    }
}

/// Concurrency-limited task scheduler.
///
/// Wraps a semaphore to ensure at most `max_concurrent` WASM
/// executions run simultaneously.
#[derive(Clone)]
pub struct Scheduler {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl Scheduler {
    /// Creates a new scheduler with the given config.
    pub fn new(config: &SchedulerConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            max_concurrent: config.max_concurrent,
        }
    }

    /// Acquires a permit to execute a task.
    ///
    /// Blocks until a slot is available. Returns a guard that
    /// releases the permit when dropped.
    pub async fn acquire(&self) -> Result<SchedulerPermit, crate::error::RuntimeError> {
        debug!(
            available = self.semaphore.available_permits(),
            max = self.max_concurrent,
            "acquiring scheduler permit"
        );

        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| crate::error::RuntimeError::PoolExhausted)?;

        Ok(SchedulerPermit { _permit: permit })
    }

    /// Returns the number of available execution slots.
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Returns the maximum concurrency level.
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Drains the scheduler by waiting until all in-flight tasks finish.
    ///
    /// Acquires every permit — once all are held simultaneously, every
    /// previously-issued permit has been released, meaning all executing
    /// tasks have completed. Permits are dropped immediately after.
    pub async fn drain(&self) {
        let mut permits = Vec::with_capacity(self.max_concurrent);
        for _ in 0..self.max_concurrent {
            if let Ok(p) = self.semaphore.clone().acquire_owned().await {
                permits.push(p);
            }
        }
        // All in-flight executions are done; permits dropped here.
    }
}

/// RAII guard that releases a scheduler permit when dropped.
pub struct SchedulerPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scheduler_acquire_and_release() {
        let scheduler = Scheduler::new(&SchedulerConfig { max_concurrent: 2 });

        assert_eq!(scheduler.available_permits(), 2);

        let _p1 = scheduler.acquire().await.expect("permit 1");
        assert_eq!(scheduler.available_permits(), 1);

        let _p2 = scheduler.acquire().await.expect("permit 2");
        assert_eq!(scheduler.available_permits(), 0);

        drop(_p1);
        assert_eq!(scheduler.available_permits(), 1);
    }

    #[tokio::test]
    async fn scheduler_blocks_at_capacity() {
        let scheduler = Scheduler::new(&SchedulerConfig { max_concurrent: 1 });

        let _p1 = scheduler.acquire().await.expect("permit");
        assert_eq!(scheduler.available_permits(), 0);

        // Second acquire should block; use try_acquire to test
        let try_result = scheduler.semaphore.clone().try_acquire_owned();
        assert!(try_result.is_err(), "should be at capacity");
    }
}
