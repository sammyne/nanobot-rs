//! MCP (Model Context Protocol) client and tool wrapper
//!
//! This crate provides MCP client functionality, allowing AI Agents to call
//! tools provided by MCP servers through the unified Tool trait interface.

mod wrapper;

pub use wrapper::{McpError, McpToolWrapper, connect};
