//! MCP method dispatch functions.
//!
//! Each sub-module handles one family of MCP methods as free functions,
//! keeping `McpHandler` itself thin (struct + routing only).

pub(crate) mod initialize;
pub(crate) mod prompts_list;
pub(crate) mod resources_list;
pub(crate) mod tools_call;
pub(crate) mod tools_list;
