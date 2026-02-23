//! End-to-end MCP round-trip test.
//!
//! Exercises a full lifecycle:
//! initialize → tools/list (empty) → install → tools/list (1 tool) → uninstall

use std::sync::Arc;

use serde_json::{json, Value};

use kami_protocol::mcp::{methods, PROTOCOL_VERSION};
use kami_protocol::{JsonRpcRequest, RequestId};
use kami_registry::ToolRepository;
use kami_runtime::{KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_transport_stdio::McpHandler;
use kami_types::{SecurityConfig, Tool, ToolId, ToolManifest, ToolVersion};

fn make_handler() -> (McpHandler, Arc<dyn ToolRepository>) {
    let repo =
        Arc::new(SqliteToolRepository::open_in_memory().expect("open")) as Arc<dyn ToolRepository>;
    let config = RuntimeConfig {
        cache_size: 4,
        max_concurrent: 2,
        epoch_interruption: true,
        ..RuntimeConfig::default()
    };
    let runtime = KamiRuntime::new(config, repo.clone()).expect("runtime");
    let handler = McpHandler::new(Arc::new(runtime), repo.clone());
    (handler, repo)
}

fn rpc(method: &str, id: i64, params: Option<Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::Number(id),
        method: method.to_string(),
        params,
    }
}

fn sample_tool(id: &str, name: &str) -> Tool {
    Tool {
        manifest: ToolManifest {
            id: ToolId::new(id).expect("id"),
            name: name.to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "sample.wasm".to_string(),
            description: format!("{name} tool"),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/tmp/tools".to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    }
}

#[tokio::test]
async fn mcp_initialize_returns_capabilities() {
    let (handler, _) = make_handler();
    let req = rpc(
        methods::INITIALIZE,
        1,
        Some(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0.1" }
        })),
    );
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("serialize");
    let parsed: Value = serde_json::from_str(&json_str).expect("parse");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["result"]["protocolVersion"], PROTOCOL_VERSION);
    assert_eq!(parsed["result"]["serverInfo"]["name"], "kami");
}

#[tokio::test]
async fn mcp_tools_list_empty_then_populated() {
    let (handler, repo) = make_handler();

    // 1. tools/list should return empty
    let req = rpc(methods::TOOLS_LIST, 1, None);
    let output = handler.dispatch(&req).await;
    let parsed: Value = serde_json::from_str(&output.to_json().expect("s")).expect("p");
    let tools = parsed["result"]["tools"].as_array().expect("array");
    assert!(tools.is_empty(), "expected no tools initially");

    // 2. Insert a tool into the registry
    let tool = sample_tool("dev.test.alpha", "alpha");
    repo.insert(&tool).await.expect("insert");

    // 3. tools/list should now return one tool
    let req2 = rpc(methods::TOOLS_LIST, 2, None);
    let output2 = handler.dispatch(&req2).await;
    let parsed2: Value = serde_json::from_str(&output2.to_json().expect("s")).expect("p");
    let tools2 = parsed2["result"]["tools"].as_array().expect("array");
    assert_eq!(tools2.len(), 1);
    assert_eq!(tools2[0]["name"], "dev.test.alpha");
}

#[tokio::test]
async fn mcp_unknown_method_returns_error() {
    let (handler, _) = make_handler();
    let req = rpc("nonexistent/method", 99, None);
    let output = handler.dispatch(&req).await;
    let json_str = output.to_json().expect("serialize");
    let parsed: Value = serde_json::from_str(&json_str).expect("parse");

    assert!(parsed["error"]["code"].is_i64());
    assert_eq!(parsed["error"]["code"], -32601);
}

#[tokio::test]
async fn registry_lifecycle_install_update_pin_uninstall() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");
    let id = ToolId::new("dev.test.lifecycle").expect("id");

    // 1. Install
    let tool = sample_tool("dev.test.lifecycle", "lifecycle");
    repo.insert(&tool).await.expect("insert");

    // 2. Verify exists
    let found = repo.find_by_id(&id).await.expect("find");
    assert!(found.is_some());
    assert_eq!(
        found.as_ref().unwrap().manifest.version,
        ToolVersion::new(1, 0, 0)
    );

    // 3. Update (new version)
    let mut updated = found.unwrap();
    updated.manifest.version = ToolVersion::new(2, 0, 0);
    updated.updated_at = Some("1700000000".to_string());
    repo.update(&updated).await.expect("update");

    let found2 = repo.find_by_id(&id).await.expect("find").expect("exists");
    assert_eq!(found2.manifest.version, ToolVersion::new(2, 0, 0));
    assert_eq!(found2.updated_at.as_deref(), Some("1700000000"));

    // 4. Pin
    let mut pinned = found2;
    pinned.pinned_version = Some("2.0.0".to_string());
    repo.update(&pinned).await.expect("pin");

    let found3 = repo.find_by_id(&id).await.expect("find").expect("exists");
    assert_eq!(found3.pinned_version.as_deref(), Some("2.0.0"));

    // 5. Unpin
    let mut unpinned = found3;
    unpinned.pinned_version = None;
    repo.update(&unpinned).await.expect("unpin");

    let found4 = repo.find_by_id(&id).await.expect("find").expect("exists");
    assert!(found4.pinned_version.is_none());

    // 6. Uninstall
    let deleted = repo.delete(&id).await.expect("delete");
    assert!(deleted);
    assert!(repo.find_by_id(&id).await.expect("find").is_none());
}
