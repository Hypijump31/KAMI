//! MCP dispatch edge-case tests for tools/call and handler coverage.

use std::sync::Arc;

use serde_json::{json, Value};

use kami_protocol::mcp::methods;
use kami_protocol::{JsonRpcNotification, JsonRpcRequest, RequestId};
use kami_runtime::{KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_transport_stdio::McpHandler;

fn make_handler() -> McpHandler {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("db"));
    let config = RuntimeConfig {
        cache_size: 4,
        max_concurrent: 2,
        epoch_interruption: false,
        ..RuntimeConfig::default()
    };
    let runtime = KamiRuntime::new(config, repo.clone()).expect("runtime");
    McpHandler::new(Arc::new(runtime), repo)
}

fn rpc(method: &str, id: i64, params: Option<Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: RequestId::Number(id),
        method: method.into(),
        params,
    }
}

#[tokio::test]
async fn tools_call_missing_params_returns_error() {
    let handler = make_handler();
    let req = rpc(methods::TOOLS_CALL, 1, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    assert!(parsed["error"]["code"].is_i64());
}

#[tokio::test]
async fn tools_call_invalid_params_returns_error() {
    let handler = make_handler();
    let req = rpc(methods::TOOLS_CALL, 2, Some(json!("not an object")));
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    assert!(parsed["error"]["message"].as_str().is_some());
}

#[tokio::test]
async fn tools_call_invalid_tool_name_returns_error() {
    let handler = make_handler();
    let req = rpc(
        methods::TOOLS_CALL,
        3,
        Some(json!({"name": "no-dot", "arguments": {}})),
    );
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    assert!(parsed["error"]["message"]
        .as_str()
        .expect("msg")
        .contains("invalid tool name"));
}

#[tokio::test]
async fn tools_call_nonexistent_tool_returns_error_content() {
    let handler = make_handler();
    let req = rpc(
        methods::TOOLS_CALL,
        4,
        Some(json!({"name": "dev.test.missing", "arguments": {}})),
    );
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    // Either an error response or a call result with isError=true
    let has_error = parsed.get("error").is_some()
        || parsed
            .pointer("/result/isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    assert!(has_error, "should indicate error for missing tool");
}

#[tokio::test]
async fn initialize_without_params_succeeds() {
    let handler = make_handler();
    let req = rpc(methods::INITIALIZE, 10, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    assert_eq!(parsed["result"]["serverInfo"]["name"], "kami");
}

#[tokio::test]
async fn initialize_with_invalid_params_returns_error() {
    let handler = make_handler();
    let req = rpc(
        methods::INITIALIZE,
        11,
        Some(json!({"protocolVersion": 123})),
    );
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    assert!(parsed["error"].is_object());
}

#[test]
fn handle_notification_initialized_does_not_panic() {
    let handler = make_handler();
    let notif = JsonRpcNotification {
        jsonrpc: "2.0".into(),
        method: methods::NOTIFICATIONS_INITIALIZED.into(),
        params: None,
    };
    handler.handle_notification(&notif);
}

#[tokio::test]
async fn prompts_list_returns_empty_array() {
    let handler = make_handler();
    let req = rpc(methods::PROMPTS_LIST, 20, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    let prompts = parsed["result"]["prompts"].as_array().expect("arr");
    assert!(prompts.is_empty());
}

#[tokio::test]
async fn resources_list_returns_empty_array() {
    let handler = make_handler();
    let req = rpc(methods::RESOURCES_LIST, 21, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    let resources = parsed["result"]["resources"].as_array().expect("arr");
    assert!(resources.is_empty());
}

#[tokio::test]
async fn resources_read_without_params_returns_error() {
    let handler = make_handler();
    let req = rpc(methods::RESOURCES_READ, 22, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    assert!(parsed["error"]["code"].is_i64());
}

#[tokio::test]
async fn resources_read_with_uri_returns_not_found() {
    let handler = make_handler();
    let req = rpc(
        methods::RESOURCES_READ,
        23,
        Some(json!({"uri": "file:///test"})),
    );
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    assert!(parsed["error"]["message"]
        .as_str()
        .expect("msg")
        .contains("resource not found"));
}

#[tokio::test]
async fn initialize_advertises_all_capabilities() {
    let handler = make_handler();
    let req = rpc(methods::INITIALIZE, 30, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    let caps = &parsed["result"]["capabilities"];
    assert!(caps["tools"].is_object(), "tools capability missing");
    assert!(caps["prompts"].is_object(), "prompts capability missing");
    assert!(
        caps["resources"].is_object(),
        "resources capability missing"
    );
}
