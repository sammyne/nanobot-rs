//! Provider 基础 trait 和类型定义

use anyhow::Result;
use nanobot_tools::ToolDefinition;
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
#[derive(Debug, Clone)]
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
        Self {
            id: id.into(),
            name: name.into(),
            arguments: params.to_string(),
        }
    }

    /// 解析参数为 JSON Value
    pub fn parse_arguments(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.arguments)
    }
}

/// 聊天消息枚举
#[derive(Debug, Clone)]
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
        Self::User {
            content: content.into(),
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::Assistant {
            content: content.into(),
            tool_calls: Vec::new(),
        }
    }

    /// 创建带工具调用的助手消息
    pub fn assistant_with_tools(
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self::Assistant {
            content: content.into(),
            tool_calls,
        }
    }

    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self::System {
            content: content.into(),
        }
    }

    /// 创建工具消息
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Tool {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
        }
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

/// LLM 响应结构
#[derive(Debug, Clone, Default)]
pub struct ProviderResponse {
    /// 响应内容
    pub content: String,
    /// 是否有工具调用
    pub has_tool_calls: bool,
    /// 工具调用列表
    pub tool_calls: Vec<ToolCall>,
}

impl ProviderResponse {
    /// 创建纯文本响应
    pub fn content(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            has_tool_calls: false,
            tool_calls: Vec::new(),
        }
    }

    /// 创建带工具调用的响应
    pub fn with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: content.into(),
            has_tool_calls: !tool_calls.is_empty(),
            tool_calls,
        }
    }
}

/// LLM 提供者 trait
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// 发送聊天请求
    async fn chat(&self, messages: &[Message]) -> Result<Message>;

    /// 绑定可用工具列表（在调用 `chat` 之前设置）
    ///
    /// # Arguments
    /// * `tools` - 工具定义列表，将被用于后续的聊天请求
    fn bind_tools(&mut self, tools: Vec<ToolDefinition>);
}