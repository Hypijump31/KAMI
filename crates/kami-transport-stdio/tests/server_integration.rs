//! Integration tests for the MCP stdio server loop.

use std::sync::Arc;

use kami_runtime::{KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_transport_stdio::{McpHandler, McpServer, StdioTransport};

fn make_handler() -> McpHandler {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("db"));
    let config = RuntimeConfig {
        cache_size: 2,
        max_concurrent: 1,
        epoch_interruption: false,
        ..RuntimeConfig::default()
    };
    let rt = Arc::new(KamiRuntime::new(config, repo.clone()).expect("rt"));
    McpHandler::new(rt, repo)
}

#[tokio::test]
async fn server_handles_valid_request() {
    let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}\n";
    let reader = tokio::io::BufReader::new(input.as_bytes());
    let mut output = Vec::new();
    let transport = StdioTransport::new(reader, &mut output);
    let mut server = McpServer::new(transport, make_handler());
    server.run().await.expect("run");
    let response = String::from_utf8(output).expect("utf8");
    assert!(response.contains("\"jsonrpc\":\"2.0\""));
    assert!(response.contains("\"id\":1"));
}

#[tokio::test]
async fn server_handles_notification_silently() {
    let input = "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n";
    let reader = tokio::io::BufReader::new(input.as_bytes());
    let mut output = Vec::new();
    let transport = StdioTransport::new(reader, &mut output);
    let mut server = McpServer::new(transport, make_handler());
    server.run().await.expect("run");
    let response = String::from_utf8(output).expect("utf8");
    assert!(response.is_empty(), "notifications must not produce output");
}

#[tokio::test]
async fn server_returns_parse_error_on_garbage() {
    let input = "not json at all\n";
    let reader = tokio::io::BufReader::new(input.as_bytes());
    let mut output = Vec::new();
    let transport = StdioTransport::new(reader, &mut output);
    let mut server = McpServer::new(transport, make_handler());
    server.run().await.expect("run");
    let response = String::from_utf8(output).expect("utf8");
    assert!(response.contains("parse error"));
}

#[tokio::test]
async fn server_handles_empty_lines() {
    let input = "\n\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"initialize\"}\n";
    let reader = tokio::io::BufReader::new(input.as_bytes());
    let mut output = Vec::new();
    let transport = StdioTransport::new(reader, &mut output);
    let mut server = McpServer::new(transport, make_handler());
    server.run().await.expect("run");
    let response = String::from_utf8(output).expect("utf8");
    assert!(response.contains("\"id\":2"));
}

#[tokio::test]
async fn server_eof_shuts_down_cleanly() {
    let input: &[u8] = b"";
    let reader = tokio::io::BufReader::new(input);
    let mut output = Vec::new();
    let transport = StdioTransport::new(reader, &mut output);
    let mut server = McpServer::new(transport, make_handler());
    server.run().await.expect("clean shutdown on eof");
    assert!(output.is_empty());
}

#[tokio::test]
async fn server_unknown_method_returns_error() {
    let input = "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"foo/bar\"}\n";
    let reader = tokio::io::BufReader::new(input.as_bytes());
    let mut output = Vec::new();
    let transport = StdioTransport::new(reader, &mut output);
    let mut server = McpServer::new(transport, make_handler());
    server.run().await.expect("run");
    let response = String::from_utf8(output).expect("utf8");
    assert!(response.contains("unknown method"));
}
