//! MCP method dispatch handler.
//!
//! `McpHandler` routes incoming JSON-RPC requests to the method-specific
//! handlers in the `dispatch` sub-modules and returns a typed response.

use std::sync::Arc;

use tracing::debug;

use kami_protocol::mcp::methods;
use kami_protocol::{
    error_codes, JsonRpcErrorResponse, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};
use kami_registry::ToolRepository;
use kami_runtime::KamiRuntime;

use crate::dispatch;

/// Handles MCP method dispatch over JSON-RPC.
///
/// Combines a `KamiRuntime` (for tool execution) and a
/// `ToolRepository` (for tool listing) to serve MCP requests.
pub struct McpHandler {
    runtime: Arc<KamiRuntime>,
    repository: Arc<dyn ToolRepository>,
}

/// Enum representing either a success or error JSON-RPC response.
#[derive(Debug)]
pub enum JsonRpcOutput {
    /// Success response.
    Success(JsonRpcResponse),
    /// Error response.
    Error(JsonRpcErrorResponse),
}

impl JsonRpcOutput {
    /// Serializes the output to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails (extremely rare).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        match self {
            Self::Success(r) => serde_json::to_string(r),
            Self::Error(r) => serde_json::to_string(r),
        }
    }
}

impl McpHandler {
    /// Creates a new handler with the given runtime and repository.
    pub fn new(runtime: Arc<KamiRuntime>, repository: Arc<dyn ToolRepository>) -> Self {
        Self {
            runtime,
            repository,
        }
    }

    /// Dispatches a JSON-RPC request to the appropriate method handler.
    #[tracing::instrument(skip(self, request), fields(method = %request.method))]
    pub async fn dispatch(&self, request: &JsonRpcRequest) -> JsonRpcOutput {
        debug!(method = %request.method, "dispatching MCP request");

        match request.method.as_str() {
            methods::INITIALIZE => {
                dispatch::initialize::handle_initialize(request.id.clone(), &request.params)
            }
            methods::TOOLS_LIST => {
                dispatch::tools_list::handle_tools_list(
                    request.id.clone(),
                    self.repository.as_ref(),
                )
                .await
            }
            methods::TOOLS_CALL => {
                dispatch::tools_call::handle_tools_call(
                    request.id.clone(),
                    &request.params,
                    &self.runtime,
                )
                .await
            }
            methods::PROMPTS_LIST => {
                dispatch::prompts_list::handle_prompts_list(request.id.clone())
            }
            methods::RESOURCES_LIST => {
                dispatch::resources_list::handle_resources_list(request.id.clone())
            }
            methods::RESOURCES_READ => {
                dispatch::resources_list::handle_resources_read(request.id.clone(), &request.params)
            }
            other => {
                tracing::warn!(method = other, "unknown MCP method");
                JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                    request.id.clone(),
                    error_codes::METHOD_NOT_FOUND,
                    format!("unknown method: {other}"),
                ))
            }
        }
    }

    /// Handles a JSON-RPC notification silently (no response is sent).
    ///
    /// Per the MCP spec, `notifications/initialized` must be accepted
    /// without generating a response.
    pub fn handle_notification(&self, notification: &JsonRpcNotification) {
        match notification.method.as_str() {
            methods::NOTIFICATIONS_INITIALIZED => {
                debug!("MCP session initialized by client");
            }
            other => {
                debug!(method = other, "notification ignored");
            }
        }
    }
}
