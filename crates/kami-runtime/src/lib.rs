//! # kami-runtime
//!
//! Runtime orchestrator for KAMI.
//! Manages tool execution, scheduling, component caching, and
//! tool resolution from the registry.
//!
//! Use `KamiRuntime` for high-level tool execution by ID, or
//! `WasmToolExecutor` directly for low-level component execution.

pub mod cache;
pub mod error;
pub mod executor;
pub mod integrity;
pub mod metrics;
pub mod orchestrator;
pub mod pipeline;
pub mod rate_limiter;
pub mod resolver;
pub mod runtime_config;
pub mod scheduler;
pub mod signature;
pub mod types;

pub use cache::{CachedComponent, ComponentCache};
pub use error::RuntimeError;
pub use executor::WasmToolExecutor;
pub use integrity::{compute_file_hash, verify_hash};
pub use metrics::{ExecutionMetrics, MetricsSnapshot};
pub use orchestrator::KamiRuntime;
pub use pipeline::{
    execute_pipeline, PipelineDefinition, PipelineError, PipelineResult, PipelineStep, StepResult,
};
pub use rate_limiter::{RateLimitConfig, RateLimiter};
pub use resolver::ToolResolver;
pub use runtime_config::RuntimeConfig;
pub use scheduler::{Scheduler, SchedulerConfig};
pub use signature::{
    generate_keypair, public_key_from_secret, sign_file, verify_file_signature, KeyPair,
};
pub use types::{ExecutionResult, ToolExecutor};
