//! Handles the `prompts/list` MCP method.
//!
//! Returns the list of available prompts. Currently empty —
//! KAMI does not expose prompts yet, but the endpoint is required
//! by the MCP specification for capability advertisement.

use kami_protocol::mcp::prompts::PromptsListResult;
use kami_protocol::{error_codes, JsonRpcErrorResponse, JsonRpcResponse, RequestId};

use crate::handler::JsonRpcOutput;

/// Handles the `prompts/list` request.
///
/// Returns an empty list since KAMI does not yet expose prompts.
pub(crate) fn handle_prompts_list(id: RequestId) -> JsonRpcOutput {
    let result = PromptsListResult { prompts: vec![] };

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
    fn prompts_list_returns_empty() {
        let output = handle_prompts_list(RequestId::Number(1));
        let json = match output {
            JsonRpcOutput::Success(r) => serde_json::to_string(&r).expect("ser"),
            JsonRpcOutput::Error(_) => panic!("expected success"),
        };
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("de");
        let prompts = parsed["result"]["prompts"].as_array().expect("arr");
        assert!(prompts.is_empty());
    }
}
