//! Tools 配置模块
//!
//! 定义工具相关的配置。
//!
//! # 配置示例
//!
//! ```json
//! {
//!   "restrictToWorkspace": true,
//!   "exec": {
//!     "timeout": 120,
//!     "pathAppend": "/usr/local/bin"
//!   },
//!   "mcpServers": {}
//! }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::mcp::McpServerConfig;

/// ExecTool 配置
///
/// 控制 Shell 命令执行的行为。
///
/// # 字段说明
///
/// - `timeout`: 命令执行超时时间（单位：秒），默认 60 秒
/// - `pathAppend`: 追加到 PATH 环境变量的路径，多个路径用冒号分隔（Linux/macOS）或分号分隔（Windows）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExecToolConfig {
    /// 命令执行超时时间（单位：秒）
    pub timeout: u64,

    /// 追加到 PATH 环境变量的路径
    pub path_append: String,
}

impl Default for ExecToolConfig {
    fn default() -> Self {
        Self { timeout: 60, path_append: String::new() }
    }
}

/// 工具配置集合
///
/// 管理所有工具的全局配置和行为。
///
/// # 字段说明
///
/// - `mcpServers`: MCP 服务器配置映射
/// - `restrictToWorkspace`: 是否限制工具在工作空间内执行（默认 false）
/// - `exec`: ExecTool 的专属配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsConfig {
    /// MCP 服务器配置
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,

    /// 是否限制工具在工作空间内执行
    ///
    /// 当设置为 true 时：
    /// - 文件系统工具（ReadFileTool、WriteFileTool 等）仅允许在工作空间目录内操作
    /// - ExecTool 会检查命令中的路径是否在工作空间范围内
    #[serde(default)]
    pub restrict_to_workspace: bool,

    /// ExecTool 配置
    ///
    /// 控制 Shell 命令执行的超时时间和 PATH 环境变量。
    #[serde(default)]
    pub exec: ExecToolConfig,
}
