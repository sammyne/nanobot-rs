//! Tools 配置模块
//!
//! 定义工具相关的配置。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::mcp::McpServerConfig;

/// 工具配置集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsConfig {
    /// MCP 服务器配置
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}
