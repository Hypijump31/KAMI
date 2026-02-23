//! Tests for the tools/list MCP handler.

use std::sync::Arc;

use serde_json::Value;

use kami_protocol::mcp::methods;
use kami_protocol::{JsonRpcRequest, RequestId};
use kami_registry::ToolRepository;
use kami_runtime::{KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_transport_stdio::McpHandler;
use kami_types::{SecurityConfig, Tool, ToolArgument, ToolId, ToolManifest, ToolVersion};

fn rpc(method: &str, id: i64, params: Option<Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: RequestId::Number(id),
        method: method.into(),
        params,
    }
}

fn make_handler_with_repo(repo: Arc<SqliteToolRepository>) -> McpHandler {
    let config = RuntimeConfig {
        cache_size: 4,
        max_concurrent: 2,
        epoch_interruption: false,
        ..RuntimeConfig::default()
    };
    let rt = Arc::new(KamiRuntime::new(config, repo.clone()).expect("rt"));
    McpHandler::new(rt, repo)
}

#[tokio::test]
async fn tools_list_empty_registry() {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("db"));
    let handler = make_handler_with_repo(repo);
    let req = rpc(methods::TOOLS_LIST, 1, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    let tools = parsed["result"]["tools"].as_array().expect("arr");
    assert!(tools.is_empty());
}

#[tokio::test]
async fn tools_list_with_registered_tool() {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("db"));
    let tool = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.t.echo").expect("id"),
            name: "echo".into(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "echo.wasm".into(),
            description: "An echo tool".into(),
            arguments: vec![ToolArgument {
                name: "msg".into(),
                arg_type: "string".into(),
                description: "message".into(),
                required: true,
                default: None,
            }],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/echo".into(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };
    repo.insert(&tool).await.expect("insert");
    let handler = make_handler_with_repo(repo);
    let req = rpc(methods::TOOLS_LIST, 2, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    let tools = parsed["result"]["tools"].as_array().expect("arr");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "dev.t.echo");
    assert!(tools[0]["inputSchema"]["properties"]["msg"].is_object());
}

#[tokio::test]
async fn tools_list_skips_disabled_tools() {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("db"));
    let tool = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.t.off").expect("id"),
            name: "off".into(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "off.wasm".into(),
            description: "disabled".into(),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/off".into(),
        enabled: false,
        pinned_version: None,
        updated_at: None,
    };
    repo.insert(&tool).await.expect("insert");
    let handler = make_handler_with_repo(repo);
    let req = rpc(methods::TOOLS_LIST, 3, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("ser");
    let parsed: Value = serde_json::from_str(&json_str).expect("de");
    let tools = parsed["result"]["tools"].as_array().expect("arr");
    assert!(tools.is_empty());
}
