//! Handles the `resources/list` and `resources/read` MCP methods.
//!
//! KAMI does not currently expose resources; both endpoints return
//! appropriate empty/error responses while advertising the capability.

use serde_json::Value;

use kami_protocol::mcp::resources::{ResourcesListResult, ResourcesReadParams};
use kami_protocol::{error_codes, JsonRpcErrorResponse, JsonRpcResponse, RequestId};

use crate::handler::JsonRpcOutput;

/// Handles the `resources/list` request.
///
/// Returns an empty resource list.
pub(crate) fn handle_resources_list(id: RequestId) -> JsonRpcOutput {
    let result = ResourcesListResult { resources: vec![] };

    match serde_json::to_value(result) {
        Ok(v) => JsonRpcOutput::Success(JsonRpcResponse::success(id, v)),
        Err(e) => JsonRpcOutput::Error(JsonRpcErrorResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            e.to_string(),
        )),
    }
}

/// Handles the `resources/read` request.
///
/// Always returns a "not found" error since no resources are registered.
///
/// # Errors
///
/// Returns `INVALID_PARAMS` if the params are malformed, or a custom
/// error code if the URI is not found.
pub(crate) fn handle_resources_read(id: RequestId, params: &Option<Value>) -> JsonRpcOutput {
    let read_params = match params {
        Some(p) => match serde_json::from_value::<ResourcesReadParams>(p.clone()) {
            Ok(rp) => rp,
            Err(e) => {
                return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                    id,
                    error_codes::INVALID_PARAMS,
                    format!("invalid resources/read params: {e}"),
                ));
            }
        },
        None => {
            return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                id,
                error_codes::INVALID_PARAMS,
                "resources/read requires params with 'uri'",
            ));
        }
    };

    JsonRpcOutput::Error(JsonRpcErrorResponse::error(
        id,
        error_codes::INVALID_PARAMS,
        format!("resource not found: {}", read_params.uri),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources_list_returns_empty() {
        let output = handle_resources_list(RequestId::Number(1));
        let json = match output {
            JsonRpcOutput::Success(r) => serde_json::to_string(&r).expect("ser"),
            JsonRpcOutput::Error(_) => panic!("expected success"),
        };
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("de");
        let resources = parsed["result"]["resources"].as_array().expect("arr");
        assert!(resources.is_empty());
    }

    #[test]
    fn resources_read_no_params_returns_error() {
        let output = handle_resources_read(RequestId::Number(2), &None);
        assert!(matches!(output, JsonRpcOutput::Error(_)));
    }

    #[test]
    fn resources_read_valid_uri_returns_not_found() {
        let params = serde_json::json!({"uri": "file:///test.txt"});
        let output = handle_resources_read(RequestId::Number(3), &Some(params));
        match output {
            JsonRpcOutput::Error(e) => {
                let json = serde_json::to_string(&e).expect("ser");
                assert!(json.contains("resource not found"));
            }
            _ => panic!("expected error"),
        }
    }

    #[test]
    fn resources_read_invalid_params_returns_error() {
        let params = serde_json::json!(42);
        let output = handle_resources_read(RequestId::Number(4), &Some(params));
        assert!(matches!(output, JsonRpcOutput::Error(_)));
    }
}
