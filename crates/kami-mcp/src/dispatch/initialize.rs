//! Handles the `initialize` MCP method.

use serde_json::Value;

use kami_protocol::mcp::initialize::{
    InitializeParams, InitializeResult, PromptsCapability, ResourcesCapability, ServerCapabilities,
    ServerInfo, ToolCapability, PROTOCOL_VERSION,
};
use kami_protocol::{error_codes, JsonRpcErrorResponse, JsonRpcResponse, RequestId};

use crate::handler::JsonRpcOutput;

/// Handles the `initialize` request and returns the server capabilities.
pub(crate) fn handle_initialize(id: RequestId, params: &Option<Value>) -> JsonRpcOutput {
    if let Some(p) = params {
        if let Err(e) = serde_json::from_value::<InitializeParams>(p.clone()) {
            return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                id,
                error_codes::INVALID_PARAMS,
                format!("invalid initialize params: {e}"),
            ));
        }
    }

    let result = InitializeResult {
        protocol_version: PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolCapability {}),
            prompts: Some(PromptsCapability {}),
            resources: Some(ResourcesCapability {}),
        },
        server_info: ServerInfo {
            name: "kami".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    match serde_json::to_value(result) {
        Ok(v) => JsonRpcOutput::Success(JsonRpcResponse::success(id, v)),
        Err(e) => JsonRpcOutput::Error(JsonRpcErrorResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            e.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_no_params_returns_success() {
        let id = RequestId::Number(1);
        let result = handle_initialize(id, &None);
        assert!(matches!(result, JsonRpcOutput::Success(_)));
    }

    #[test]
    fn initialize_with_valid_params_returns_success() {
        let id = RequestId::Number(2);
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        });
        let result = handle_initialize(id, &Some(params));
        assert!(matches!(result, JsonRpcOutput::Success(_)));
    }

    #[test]
    fn initialize_with_non_object_params_returns_error() {
        let id = RequestId::Number(3);
        let params = serde_json::json!(42);
        let result = handle_initialize(id, &Some(params));
        assert!(matches!(result, JsonRpcOutput::Error(_)));
    }
}
