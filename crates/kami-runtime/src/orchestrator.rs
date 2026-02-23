//! Top-level runtime orchestrator — combines resolution, scheduling, and WASM execution.

use std::sync::Arc;

use kami_engine::{create_engine, create_linker, HostState, InstanceConfig};
use kami_registry::ToolRepository;
use kami_types::ToolId;
use tracing::{info, warn};
use wasmtime::{component::Linker, Engine};

use crate::rate_limiter::RateLimiter;
use crate::scheduler::{Scheduler, SchedulerConfig};
use crate::types::{ExecutionResult, ToolExecutor};
use crate::{cache::ComponentCache, error::RuntimeError, executor::WasmToolExecutor};
use crate::{metrics::ExecutionMetrics, resolver::ToolResolver, runtime_config::RuntimeConfig};

/// Top-level runtime orchestrator.
///
/// Combines tool resolution, scheduling, rate limiting, and sandboxed WASM execution.
/// Use `metrics()` to read live atomic counters.
pub struct KamiRuntime {
    executor: WasmToolExecutor,
    resolver: ToolResolver,
    scheduler: Scheduler,
    rate_limiter: RateLimiter,
    metrics: Arc<ExecutionMetrics>,
}

impl KamiRuntime {
    /// Creates a new runtime with the given configuration and repository.
    ///
    /// # Errors
    /// Returns `RuntimeError` if the engine or linker cannot be created.
    pub fn new(
        config: RuntimeConfig,
        repository: Arc<dyn ToolRepository>,
    ) -> Result<Self, RuntimeError> {
        let instance_config = InstanceConfig {
            epoch_interruption: config.epoch_interruption,
            ..InstanceConfig::default()
        };
        let engine = create_engine(&instance_config)?;
        let linker = create_linker(&engine)?;
        let cache = ComponentCache::new(config.cache_size);
        let scheduler = Scheduler::new(&SchedulerConfig {
            max_concurrent: config.max_concurrent,
        });
        let rate_limiter = RateLimiter::new(&config.rate_limit);
        let metrics = ExecutionMetrics::new_shared();
        Ok(Self {
            executor: WasmToolExecutor::new(engine.clone(), linker),
            resolver: ToolResolver::new(engine, cache, repository),
            scheduler,
            rate_limiter,
            metrics,
        })
    }

    /// Creates a runtime from an existing engine and linker.
    pub fn with_engine(
        engine: Engine,
        linker: Linker<HostState>,
        config: RuntimeConfig,
        repository: Arc<dyn ToolRepository>,
    ) -> Self {
        let cache = ComponentCache::new(config.cache_size);
        let scheduler_config = SchedulerConfig {
            max_concurrent: config.max_concurrent,
        };
        let rate_limiter = RateLimiter::new(&config.rate_limit);
        let metrics = ExecutionMetrics::new_shared();
        Self {
            executor: WasmToolExecutor::new(engine.clone(), linker),
            resolver: ToolResolver::new(engine, cache, repository),
            scheduler: Scheduler::new(&scheduler_config),
            rate_limiter,
            metrics,
        }
    }

    /// Executes a tool by its ID with the given JSON input.
    ///
    /// # Errors
    /// Returns `RuntimeError::RateLimited` if the request exceeds rate limits.
    /// Returns `RuntimeError::ToolNotFound` or `RuntimeError::PoolExhausted`.
    #[tracing::instrument(skip(self, input), fields(tool_id = %tool_id))]
    pub async fn execute(
        &self,
        tool_id: &ToolId,
        input: &str,
    ) -> Result<ExecutionResult, RuntimeError> {
        info!(%tool_id, "executing tool");
        self.metrics.record_attempt();

        // Rate limit check — before any expensive work
        if !self.rate_limiter.check(tool_id) {
            self.metrics.record_failure();
            return Err(RuntimeError::RateLimited {
                tool_id: tool_id.to_string(),
                limit: self.rate_limiter.config().per_tool,
                window_secs: self.rate_limiter.config().window.as_secs(),
            });
        }

        if self.resolver.cache().get(tool_id).await.is_some() {
            self.metrics.record_cache_hit();
        } else {
            self.metrics.record_cache_miss();
        }

        let _permit = self
            .scheduler
            .acquire()
            .await
            .inspect_err(|_| self.metrics.record_failure())?;
        let cached = self
            .resolver
            .resolve(tool_id)
            .await
            .inspect_err(|_| self.metrics.record_failure())?;

        let result = self
            .executor
            .execute(&cached.component, input, &cached.security)
            .await;

        match &result {
            Ok(r) => {
                self.metrics.record_success(r.fuel_consumed);
                info!(%tool_id, success = r.success, duration_ms = r.duration_ms,
                    fuel = r.fuel_consumed, "execution complete");
            }
            Err(e) => {
                self.metrics.record_failure();
                warn!(%tool_id, error = %e, "execution failed");
            }
        }
        result
    }

    /// Gracefully shuts down the runtime by draining all in-flight executions.
    pub async fn shutdown(&self) {
        self.scheduler.drain().await;
        info!("runtime shutdown complete");
    }

    /// Invalidates the component cache for a specific tool.
    pub async fn invalidate_cache(&self, tool_id: &ToolId) {
        self.resolver.invalidate(tool_id).await;
    }

    /// Returns a shared handle to the runtime execution metrics.
    pub fn metrics(&self) -> Arc<ExecutionMetrics> {
        self.metrics.clone()
    }

    /// Returns a reference to the tool resolver.
    pub fn resolver(&self) -> &ToolResolver {
        &self.resolver
    }

    /// Returns a reference to the task scheduler.
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }
}
