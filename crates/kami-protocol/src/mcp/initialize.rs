//! MCP initialize method types.
//!
//! The initialize handshake is the first message exchanged between
//! client and server to negotiate capabilities and protocol version.

use serde::{Deserialize, Serialize};

/// Client capabilities declared during initialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Whether the client supports tool execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapability>,
}

/// Tool-related capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolCapability {}

/// Prompts-related capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsCapability {}

/// Resources-related capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesCapability {}

/// Server capabilities returned during initialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tools capability (present if server exposes tools).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapability>,
    /// Prompts capability (present if server exposes prompts).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    /// Resources capability (present if server exposes resources).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
}

/// Client info sent during initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name.
    pub name: String,
    /// Client version.
    pub version: String,
}

/// Server info returned during initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    pub version: String,
}

/// Request params for `initialize`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version requested by client.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client info.
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Response for `initialize`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version agreed by server.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server info.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// MCP protocol version supported by this implementation.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_params_roundtrip() {
        let params = InitializeParams {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities {
                tools: Some(ToolCapability {}),
            },
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };
        let json = serde_json::to_string(&params).expect("serialize");
        let back: InitializeParams = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.protocol_version, PROTOCOL_VERSION);
        assert_eq!(back.client_info.name, "test-client");
    }

    #[test]
    fn initialize_result_roundtrip() {
        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolCapability {}),
                prompts: Some(PromptsCapability {}),
                resources: Some(ResourcesCapability {}),
            },
            server_info: ServerInfo {
                name: "kami".to_string(),
                version: "0.1.0".to_string(),
            },
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let back: InitializeResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.server_info.name, "kami");
        assert!(back.capabilities.prompts.is_some());
        assert!(back.capabilities.resources.is_some());
    }

    #[test]
    fn client_capabilities_default_has_no_tools() {
        let caps = ClientCapabilities::default();
        assert!(caps.tools.is_none());
    }

    #[test]
    fn server_capabilities_empty_roundtrip() {
        let caps = ServerCapabilities::default();
        let json = serde_json::to_string(&caps).expect("ser");
        let back: ServerCapabilities = serde_json::from_str(&json).expect("de");
        assert!(back.tools.is_none());
    }

    #[test]
    fn protocol_version_constant() {
        assert_eq!(PROTOCOL_VERSION, "2024-11-05");
    }
}
