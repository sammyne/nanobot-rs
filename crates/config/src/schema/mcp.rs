//! MCP（Model Context Protocol）服务器配置
//!
//! 定义 MCP 服务器的连接配置，支持 Stdio（本地进程）和 HTTP/SSE（远程服务）两种方式。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// MCP 服务器传输类型配置
///
/// 定义 MCP 服务器的连接方式，支持 Stdio（本地进程）和 HTTP/SSE（远程服务）两种方式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged, rename_all = "lowercase")]
pub enum McpServerConfig {
    /// Stdio 方式：启动本地进程并通过标准输入/输出通信
    Stdio {
        /// 要执行的命令
        command: String,
        /// 命令行参数
        #[serde(default)]
        args: Vec<String>,
        /// 环境变量
        #[serde(default)]
        env: HashMap<String, String>,
    },
    /// HTTP/SSE 方式：连接远程 MCP 服务器
    Http {
        /// 服务器 URL
        url: String,
        /// HTTP 请求头
        #[serde(default)]
        headers: HashMap<String, String>,
        /// 工具调用超时时间（秒）
        #[serde(default = "default_tool_timeout")]
        tool_timeout: u64,
    },
}

/// 默认工具超时时间：30 秒
fn default_tool_timeout() -> u64 {
    30
}

impl McpServerConfig {
    /// 创建 Stdio 类型的配置
    pub fn stdio(command: impl Into<String>) -> Self {
        Self::Stdio { command: command.into(), args: Vec::new(), env: HashMap::new() }
    }

    /// 创建 HTTP 类型的配置
    pub fn http(url: impl Into<String>) -> Self {
        Self::Http { url: url.into(), headers: HashMap::new(), tool_timeout: default_tool_timeout() }
    }

    /// 设置命令行参数（仅对 Stdio 类型有效）
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        if let Self::Stdio { args: ref mut a, .. } = self {
            *a = args;
        }
        self
    }

    /// 设置环境变量（仅对 Stdio 类型有效）
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        if let Self::Stdio { env: ref mut e, .. } = self {
            *e = env;
        }
        self
    }

    /// 设置 HTTP 请求头（仅对 HTTP 类型有效）
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        if let Self::Http { headers: ref mut h, .. } = self {
            *h = headers;
        }
        self
    }

    /// 设置工具调用超时时间（秒），仅对 HTTP 类型有效
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        if let Self::Http { tool_timeout: ref mut t, .. } = self {
            *t = timeout;
        }
        self
    }

    /// 获取超时时间作为 Duration
    ///
    /// 对于 HTTP 类型返回配置的超时时间，对于 Stdio 类型返回默认值 30 秒
    pub fn timeout_duration(&self) -> std::time::Duration {
        match self {
            Self::Http { tool_timeout, .. } => std::time::Duration::from_secs(*tool_timeout),
            Self::Stdio { .. } => std::time::Duration::from_secs(default_tool_timeout()),
        }
    }
}
