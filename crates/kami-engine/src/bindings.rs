//! Host-side bindings for the `kami-tool` WIT world.
//!
//! Uses `wasmtime::component::bindgen!` to generate typed accessors
//! for tool exports and host import implementations.

use core::pin::Pin;

use wasmtime::component::{Component, Linker};
use wasmtime::Store;

use crate::error::EngineError;
use crate::state::HostState;

wasmtime::component::bindgen!({
    world: "kami-tool",
    path: "../../wit",
    async: true,
});

/// Implements the `kami:tool/host` interface for guest→host calls.
impl kami::tool::host::Host for HostState {
    fn log<'a, 'b>(
        &'a mut self,
        level: kami::tool::host::LogLevel,
        message: String,
    ) -> Pin<Box<dyn core::future::Future<Output = ()> + Send + 'b>>
    where
        Self: 'b,
        'a: 'b,
    {
        Box::pin(async move {
            use kami::tool::host::LogLevel;
            match level {
                LogLevel::Trace => tracing::trace!(target: "kami_guest", "{message}"),
                LogLevel::Debug => tracing::debug!(target: "kami_guest", "{message}"),
                LogLevel::Info => tracing::info!(target: "kami_guest", "{message}"),
                LogLevel::Warn => tracing::warn!(target: "kami_guest", "{message}"),
                LogLevel::Error => tracing::error!(target: "kami_guest", "{message}"),
            }
        })
    }
}

/// Instantiates a kami-tool component with typed bindings.
///
/// # Errors
///
/// Returns `EngineError::Instantiation` if the component doesn't match
/// the `kami-tool` WIT world.
pub async fn instantiate_tool(
    linker: &Linker<HostState>,
    store: &mut Store<HostState>,
    component: &Component,
) -> Result<KamiTool, EngineError> {
    KamiTool::instantiate_async(&mut *store, component, linker)
        .await
        .map_err(|e| EngineError::Instantiation {
            reason: "kami-tool component".to_string(),
            source: e,
        })
}

/// Calls the `run` export on a kami-tool component via typed bindings.
///
/// # Errors
///
/// Returns `EngineError::Trap` if execution fails.
pub async fn call_run(
    store: &mut Store<HostState>,
    tool: &KamiTool,
    input: &str,
) -> Result<Result<String, String>, EngineError> {
    tool.kami_tool_tool()
        .call_run(&mut *store, input)
        .await
        .map_err(|e: wasmtime::Error| EngineError::Trap {
            message: e.to_string(),
        })
}

/// Calls the `describe` export on a kami-tool component.
///
/// Returns the tool's self-reported metadata as a JSON string.
///
/// # Errors
///
/// Returns `EngineError::Trap` if the call fails or the component
/// does not implement `describe`.
pub async fn call_describe(
    store: &mut Store<HostState>,
    tool: &KamiTool,
) -> Result<String, EngineError> {
    tool.kami_tool_tool()
        .call_describe(&mut *store)
        .await
        .map_err(|e: wasmtime::Error| EngineError::Trap {
            message: e.to_string(),
        })
}
