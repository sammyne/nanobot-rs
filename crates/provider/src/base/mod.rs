//! Provider 基础 trait 和类型定义

use anyhow::Result;
use nanobot_tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 提供者相关错误
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("LLM API 调用失败: {0}")]
    Api(String),

    #[error("请求超时")]
    Timeout,

    #[error("配置错误: {0}")]
    Config(String),
}

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 调用 ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 参数（JSON 字符串）
    pub arguments: String,
}

impl ToolCall {
    /// 创建新的工具调用
    pub fn new(id: impl Into<String>, name: impl Into<String>, params: serde_json::Value) -> Self {
        Self { id: id.into(), name: name.into(), arguments: params.to_string() }
    }

    /// 解析参数为指定类型
    ///
    /// # Type Parameters
    ///
    /// * `T` - 目标类型，必须实现 `Deserialize`
    ///
    /// # Errors
    ///
    /// 返回 `serde_json::Error` 如果反序列化失败
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use nanobot_provider::ToolCall;
    /// # let tool_call = ToolCall::new("id", "name", serde_json::json!({ "action": "run" }));
    /// #[derive(serde::Deserialize)]
    /// struct Action {
    ///     action: String,
    /// }
    ///
    /// let action: Action = tool_call.parse_arguments()?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn parse_arguments<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_str(&self.arguments)
    }
}

/// 聊天消息枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    /// 系统消息
    System {
        /// 消息内容
        content: String,
    },
    /// 用户消息
    User {
        /// 消息内容
        content: String,
    },
    /// 助手消息
    Assistant {
        /// 消息内容
        content: String,
        /// 工具调用列表
        tool_calls: Vec<ToolCall>,
    },
    /// 工具消息（工具调用结果）
    Tool {
        /// 消息内容
        content: String,
        /// 关联的工具调用 ID
        tool_call_id: String,
    },
}

impl Message {
    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self::User { content: content.into() }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::Assistant { content: content.into(), tool_calls: Vec::new() }
    }

    /// 创建带工具调用的助手消息
    pub fn assistant_with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self::Assistant { content: content.into(), tool_calls }
    }

    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self::System { content: content.into() }
    }

    /// 创建工具消息
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Tool { tool_call_id: tool_call_id.into(), content: content.into() }
    }

    /// 获取角色
    pub fn role(&self) -> &str {
        match self {
            Self::System { .. } => "system",
            Self::User { .. } => "user",
            Self::Assistant { .. } => "assistant",
            Self::Tool { .. } => "tool",
        }
    }

    /// 获取内容
    pub fn content(&self) -> &str {
        match self {
            Self::System { content } => content,
            Self::User { content } => content,
            Self::Assistant { content, .. } => content,
            Self::Tool { content, .. } => content,
        }
    }

    /// 获取工具调用列表
    ///
    /// 对于非 Assistant 消息返回空切片
    pub fn tool_calls(&self) -> &[ToolCall] {
        match self {
            Self::Assistant { tool_calls, .. } => tool_calls,
            _ => &[],
        }
    }

    /// 获取工具调用 ID（仅 Tool 消息）
    pub fn tool_call_id(&self) -> Option<&str> {
        match self {
            Self::Tool { tool_call_id, .. } => Some(tool_call_id),
            _ => None,
        }
    }
}

/// LLM 调用选项
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// 最大生成 token 数
    pub max_tokens: u16,

    /// 温度参数（0.0-2.0）
    pub temperature: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self { max_tokens: 4096, temperature: 0.7 }
    }
}

/// LLM 响应结构
#[derive(Debug, Clone, Default)]
pub struct ProviderResponse {
    /// 响应内容
    pub content: String,
    /// 工具调用列表
    pub tool_calls: Vec<ToolCall>,
}

impl ProviderResponse {
    /// 创建纯文本响应
    pub fn content(content: impl Into<String>) -> Self {
        Self { content: content.into(), tool_calls: Vec::new() }
    }

    /// 创建带工具调用的响应
    pub fn with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self { content: content.into(), tool_calls }
    }
}

/// LLM 提供者 trait
#[async_trait::async_trait]
pub trait Provider: Send + Sync + Clone + 'static {
    /// 发送聊天请求
    async fn chat(&self, messages: &[Message], options: &Options) -> Result<Message>;

    /// 绑定可用工具列表（在调用 `chat` 之前设置）
    ///
    /// # Arguments
    /// * `tools` - 工具定义列表，将被用于后续的聊天请求
    fn bind_tools(&mut self, tools: Vec<ToolDefinition>);
}
