//! Integration tests for `KamiRuntime` orchestrator.

use std::sync::Arc;

use async_trait::async_trait;
use kami_engine::{create_engine, create_linker, InstanceConfig};
use kami_registry::{RepositoryError, ToolQuery, ToolRepository};
use kami_runtime::{KamiRuntime, RateLimitConfig, RuntimeConfig, RuntimeError};
use kami_types::{Tool, ToolId};

// ---------------------------------------------------------------------------
// Minimal mock repository
// ---------------------------------------------------------------------------

struct EmptyRepository;

#[async_trait]
impl ToolRepository for EmptyRepository {
    async fn find_by_id(&self, _id: &ToolId) -> Result<Option<Tool>, RepositoryError> {
        Ok(None)
    }

    async fn find_all(&self, _query: ToolQuery) -> Result<Vec<Tool>, RepositoryError> {
        Ok(vec![])
    }

    async fn insert(&self, _tool: &Tool) -> Result<(), RepositoryError> {
        Err(RepositoryError::Storage {
            message: "read-only".to_string(),
        })
    }

    async fn update(&self, _tool: &Tool) -> Result<(), RepositoryError> {
        Err(RepositoryError::Storage {
            message: "read-only".to_string(),
        })
    }

    async fn delete(&self, _id: &ToolId) -> Result<bool, RepositoryError> {
        Ok(false)
    }
}

fn make_runtime() -> KamiRuntime {
    let config = RuntimeConfig {
        cache_size: 4,
        max_concurrent: 2,
        epoch_interruption: true,
        ..RuntimeConfig::default()
    };
    KamiRuntime::new(config, Arc::new(EmptyRepository)).expect("runtime")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn orchestrator_tool_not_found() {
    let runtime = make_runtime();
    let id = ToolId::new("dev.test.unknown").expect("id");

    let result = runtime.execute(&id, "{}").await;

    assert!(
        matches!(result, Err(RuntimeError::ToolNotFound { .. })),
        "expected ToolNotFound",
    );
}

#[tokio::test]
async fn orchestrator_invalidate_cache_does_not_panic() {
    let runtime = make_runtime();
    let id = ToolId::new("dev.test.cache").expect("id");

    // Invalidating a tool that was never cached should be a no-op.
    runtime.invalidate_cache(&id).await;
}

#[tokio::test]
async fn orchestrator_scheduler_reports_permits() {
    let runtime = make_runtime();
    let scheduler = runtime.scheduler();

    assert_eq!(scheduler.max_concurrent(), 2);
    assert_eq!(scheduler.available_permits(), 2);
}

#[tokio::test]
async fn orchestrator_default_config_is_valid() {
    let config = RuntimeConfig::default();
    let result = KamiRuntime::new(config, Arc::new(EmptyRepository));
    assert!(
        result.is_ok(),
        "default RuntimeConfig should create a valid runtime"
    );
}

#[tokio::test]
async fn orchestrator_metrics_track_failed_execution() {
    let runtime = make_runtime();
    let id = ToolId::new("dev.test.metrics").expect("id");

    let _ = runtime.execute(&id, "{}").await;

    let snap = runtime.metrics().snapshot();
    assert_eq!(snap.total_executions, 1);
    assert_eq!(snap.failed_executions, 1);
    assert_eq!(snap.successful_executions, 0);
    assert_eq!(snap.cache_misses, 1);
    assert_eq!(snap.cache_hits, 0);
}

#[tokio::test]
async fn orchestrator_shutdown_completes() {
    let runtime = make_runtime();
    // Should drain with no in-flight tasks and return immediately.
    runtime.shutdown().await;
}

#[tokio::test]
async fn orchestrator_with_engine_creates_valid_runtime() {
    let instance_config = InstanceConfig::default();
    let engine = create_engine(&instance_config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let config = RuntimeConfig {
        cache_size: 2,
        max_concurrent: 1,
        epoch_interruption: false,
        ..RuntimeConfig::default()
    };
    let runtime = KamiRuntime::with_engine(engine, linker, config, Arc::new(EmptyRepository));
    let id = ToolId::new("dev.test.wengine").expect("id");
    let result = runtime.execute(&id, "{}").await;
    assert!(matches!(result, Err(RuntimeError::ToolNotFound { .. })));
}

#[tokio::test]
async fn orchestrator_resolver_accessible() {
    let runtime = make_runtime();
    // resolver() should be accessible without panic
    let _resolver = runtime.resolver();
}

#[tokio::test]
async fn orchestrator_rate_limited_rejects_excess_requests() {
    use std::time::Duration;

    let config = RuntimeConfig {
        cache_size: 4,
        max_concurrent: 2,
        epoch_interruption: true,
        rate_limit: RateLimitConfig {
            per_tool: 2,
            global: 0,
            window: Duration::from_secs(60),
        },
    };
    let runtime = KamiRuntime::new(config, Arc::new(EmptyRepository)).expect("runtime");
    let id = ToolId::new("dev.test.ratelimit").expect("id");

    // First two calls consume tokens (they fail with ToolNotFound, but rate limit passes)
    let _ = runtime.execute(&id, "{}").await;
    let _ = runtime.execute(&id, "{}").await;

    // Third call should be rate limited
    let result = runtime.execute(&id, "{}").await;
    assert!(
        matches!(result, Err(RuntimeError::RateLimited { .. })),
        "expected RateLimited, got {result:?}",
    );
}
