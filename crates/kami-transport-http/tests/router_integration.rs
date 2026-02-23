//! Integration tests for the HTTP router (handle_mcp, handle_health).

use std::sync::Arc;

use axum::body::Body;
use http::Request;
use tower::ServiceExt;

use kami_mcp::McpHandler;
use kami_runtime::{KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_transport_http::{build_router, AppState};

fn make_state(token: Option<&str>) -> AppState {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("db"));
    let config = RuntimeConfig {
        cache_size: 4,
        max_concurrent: 2,
        epoch_interruption: false,
        ..RuntimeConfig::default()
    };
    let runtime = Arc::new(KamiRuntime::new(config, repo.clone()).expect("rt"));
    AppState {
        handler: Arc::new(McpHandler::new(runtime, repo)),
        token: token.map(String::from),
    }
}

#[tokio::test]
async fn health_returns_ok() {
    let app = build_router(make_state(None));
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .expect("req");
    let resp = app.oneshot(req).await.expect("resp");
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn ready_endpoint_returns_ok() {
    let app = build_router(make_state(None));
    let req = Request::builder()
        .uri("/health/ready")
        .body(Body::empty())
        .expect("req");
    let resp = app.oneshot(req).await.expect("resp");
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 1024)
        .await
        .expect("body");
    let text = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(text.contains("ready"));
}

#[tokio::test]
async fn mcp_parse_error() {
    let app = build_router(make_state(None));
    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .body(Body::from("not json"))
        .expect("req");
    let resp = app.oneshot(req).await.expect("resp");
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 8192)
        .await
        .expect("body");
    let text = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(text.contains("parse error") || text.contains("Parse error"));
}

#[tokio::test]
async fn mcp_notification_returns_no_content() {
    let app = build_router(make_state(None));
    let body = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .body(Body::from(body))
        .expect("req");
    let resp = app.oneshot(req).await.expect("resp");
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn mcp_valid_request_returns_200() {
    let app = build_router(make_state(None));
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .body(Body::from(body))
        .expect("req");
    let resp = app.oneshot(req).await.expect("resp");
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), 8192)
        .await
        .expect("body");
    let text = String::from_utf8(bytes.to_vec()).expect("utf8");
    assert!(text.contains("\"jsonrpc\":\"2.0\""));
}

#[tokio::test]
async fn mcp_auth_required_but_missing() {
    let app = build_router(make_state(Some("secret")));
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .body(Body::from(body))
        .expect("req");
    let resp = app.oneshot(req).await.expect("resp");
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn mcp_auth_valid_bearer_passes() {
    let app = build_router(make_state(Some("s3cret")));
    let body = r#"{"jsonrpc":"2.0","id":2,"method":"initialize"}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("Authorization", "Bearer s3cret")
        .body(Body::from(body))
        .expect("req");
    let resp = app.oneshot(req).await.expect("resp");
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn mcp_invalid_request_missing_method() {
    let app = build_router(make_state(None));
    let body = r#"{"jsonrpc":"2.0","id":5}"#;
    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .body(Body::from(body))
        .expect("req");
    let resp = app.oneshot(req).await.expect("resp");
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), 8192)
        .await
        .expect("body");
    let text = String::from_utf8(bytes.to_vec()).expect("utf8");
    assert!(text.contains("Invalid request"));
}
