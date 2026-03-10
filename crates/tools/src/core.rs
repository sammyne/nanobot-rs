//! 基础工具抽象层
//!
//! 定义 Tool trait 和通用的工具结果、错误类型。

use async_trait::async_trait;
use schemars::schema::SchemaObject;
use thiserror::Error;

/// 工具执行结果
pub type ToolResult = Result<String, ToolError>;

/// 工具错误类型
#[derive(Error, Debug, Clone)]
pub enum ToolError {
    /// 参数验证错误
    #[error("参数验证失败: {field} - {message}")]
    Validation { field: String, message: String },

    /// 执行错误
    #[error("工具执行失败: {0}")]
    Execution(String),

    /// 工具不存在
    #[error("工具不存在: {0}")]
    NotFound(String),

    /// 权限拒绝
    #[error("权限被拒绝: 路径 {path} 超出允许范围 {allowed:?}")]
    PermissionDenied { path: String, allowed: Option<String> },

    /// 超时
    #[error("工具执行超时: 限制 {limit}s, 实际执行 {elapsed:?}")]
    Timeout { limit: u64, elapsed: std::time::Duration },

    /// 路径错误
    #[error("路径错误: {0}")]
    Path(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(String),
}

impl ToolError {
    /// 创建参数验证错误
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// 创建执行错误
    pub fn execution(msg: impl Into<String>) -> Self {
        Self::Execution(msg.into())
    }

    /// 创建路径错误
    pub fn path(msg: impl Into<String>) -> Self {
        Self::Path(msg.into())
    }

    /// 创建 IO 错误
    pub fn io(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

/// 工具执行上下文
///
/// 携带工具执行时的请求来源信息
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// 通道名称（如 dingtalk、wechat 等）
    channel: String,
    /// 聊天标识
    chat_id: String,
}

impl ToolContext {
    /// 创建新的工具上下文
    pub fn new(channel: impl Into<String>, chat_id: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            chat_id: chat_id.into(),
        }
    }

    /// 获取通道名称
    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// 获取聊天标识
    pub fn chat_id(&self) -> &str {
        &self.chat_id
    }
}

/// 工具定义结构体（用于 OpenAI Function Calling）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 参数 Schema
    pub parameters: serde_json::Value,
}

/// Tool trait - 所有工具的抽象接口
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 生成 JSON Schema 描述参数
    fn parameters(&self) -> SchemaObject;

    /// 异步执行工具
    ///
    /// # Arguments
    /// * `ctx` - 工具执行上下文，包含 channel 和 chat_id
    /// * `params` - JSON 格式的参数
    ///
    /// # Returns
    /// 成功返回输出字符串，失败返回 ToolError
    async fn execute(&self, ctx: &ToolContext, params: serde_json::Value) -> ToolResult;

    /// 转换为 OpenAI Function Calling 格式
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::to_value(self.parameters()).unwrap_or_default(),
        }
    }
}

/// Helper: 验证必需参数
pub fn require_param(params: &serde_json::Value, name: &str) -> Result<String, ToolError> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ToolError::validation(name, format!("缺少必需参数: {name}")))
}

/// Helper: 获取可选参数
pub fn optional_param(params: &serde_json::Value, name: &str) -> Option<String> {
    params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Helper: 获取布尔参数（默认 false）
pub fn bool_param(params: &serde_json::Value, name: &str) -> bool {
    params.get(name).and_then(|v| v.as_bool()).unwrap_or(false)
}

/// Helper: 获取数值参数
pub fn u64_param(params: &serde_json::Value, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
