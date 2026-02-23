//! HTTP server that binds an axum Router to a TCP socket.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;

use kami_mcp::McpHandler;

use crate::error::HttpTransportError;
use crate::router::{build_router, AppState};

/// Axum-based HTTP server for the MCP JSON-RPC transport.
pub struct HttpServer {
    pub(crate) addr: SocketAddr,
    pub(crate) state: AppState,
}

impl HttpServer {
    /// Creates a new HTTP server.
    ///
    /// # Arguments
    ///
    /// * `handler` — shared MCP dispatcher
    /// * `port` — TCP port to listen on
    /// * `token` — optional Bearer token for authentication
    pub fn new(handler: Arc<McpHandler>, port: u16, token: Option<String>) -> Self {
        Self {
            addr: SocketAddr::from(([0, 0, 0, 0], port)),
            state: AppState { handler, token },
        }
    }

    /// Starts the server and blocks until it exits.
    ///
    /// # Errors
    ///
    /// Returns an error if the TCP bind fails or the server crashes.
    pub async fn run(self) -> Result<(), HttpTransportError> {
        let listener =
            TcpListener::bind(self.addr)
                .await
                .map_err(|e| HttpTransportError::Bind {
                    addr: self.addr.to_string(),
                    source: e,
                })?;

        tracing::info!(addr = %self.addr, "KAMI MCP HTTP server ready");

        let router = build_router(self.state);
        axum::serve(listener, router)
            .await
            .map_err(|e| HttpTransportError::Serve(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kami_mcp::McpHandler;
    use kami_runtime::{KamiRuntime, RuntimeConfig};
    use kami_store_sqlite::SqliteToolRepository;

    fn make_handler() -> Arc<McpHandler> {
        let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("in-memory db"));
        let config = RuntimeConfig {
            cache_size: 4,
            max_concurrent: 2,
            epoch_interruption: false,
            ..RuntimeConfig::default()
        };
        let runtime = Arc::new(KamiRuntime::new(config, repo.clone()).expect("runtime"));
        Arc::new(McpHandler::new(runtime, repo))
    }

    #[test]
    fn new_sets_correct_port() {
        let server = HttpServer::new(make_handler(), 3000, None);
        assert_eq!(server.addr.port(), 3000);
    }

    #[test]
    fn new_stores_bearer_token() {
        let server = HttpServer::new(make_handler(), 8080, Some("s3cret".to_string()));
        assert_eq!(server.state.token.as_deref(), Some("s3cret"));
    }

    #[test]
    fn new_with_no_token() {
        let server = HttpServer::new(make_handler(), 9000, None);
        assert!(server.state.token.is_none());
    }
}
