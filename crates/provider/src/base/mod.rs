//! Provider 基础 trait 和类型定义

use std::borrow::Cow;

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

    /// 生成工具调用的预览字符串
    ///
    /// 将工具调用格式化为易读的字符串，例如 `web_search(input="query")`。
    /// 如果第一个参数值超过 40 个字符，会被截断并添加省略号。
    pub fn preview(&self) -> String {
        use serde_json::Value;

        let args = match serde_json::from_str::<Value>(&self.arguments) {
            Ok(Value::Object(v)) => v,
            _ => return self.name.clone(),
        };

        match args.iter().find(|(_, v)| !(v.is_array() || v.is_object())) {
            Some((k, Value::String(v))) => match nanobot_utils::strings::truncate(v, 40) {
                Some(vv) => format!("{}({k}=\"{vv}…\")", self.name),
                None => format!("{}({k}=\"{v}\")", self.name),
            },
            Some((k, v)) => format!("{}({k}={v})", self.name),
            None => self.name.clone(),
        }
    }
}

/// 用户消息内容
///
/// 支持纯文本和多模态（文本 + 图片混合）两种形式。
/// 使用 `#[serde(untagged)]` 保持向后兼容：旧 JSONL 中的纯字符串自动反序列化为 `Text` 变体。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserContent {
    /// 纯文本内容
    Text(String),
    /// 多模态内容（文本 + 图片混合）
    Parts(Vec<ContentPart>),
}

impl UserContent {
    /// 提取纯文本内容
    ///
    /// - `Text(s)` 直接返回引用
    /// - `Parts(parts)` 拼接所有 `Text` 片段
    pub fn text(&self) -> Cow<'_, str> {
        match self {
            Self::Text(s) => Cow::Borrowed(s),
            Self::Parts(parts) => {
                let texts: Vec<&str> = parts
                    .iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect();
                Cow::Owned(texts.join("\n"))
            }
        }
    }
}

impl From<String> for UserContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for UserContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

/// 内容片段
///
/// 用于 `UserContent::Parts` 中的单个内容元素。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// 文本片段
    Text {
        /// 文本内容
        text: String,
    },
    /// 图片片段
    Image {
        /// MIME 类型，如 `"image/png"`
        media_type: String,
        /// 裸 base64 编码数据（不含 `data:...;base64,` 前缀）
        data: String,
    },
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
        /// 消息内容（纯文本或多模态）
        content: UserContent,
    },
    /// 助手消息
    Assistant {
        /// 消息内容
        content: String,
        /// 工具调用列表
        tool_calls: Vec<ToolCall>,
        /// 思考过程（不透明数据，由 provider 写入和读取）
        #[serde(default, skip_serializing_if = "Option::is_none")]
        thinking: Option<serde_json::Value>,
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
        Self::User { content: UserContent::Text(content.into()) }
    }

    /// 创建带多模态内容的用户消息
    pub fn user_with_parts(parts: Vec<ContentPart>) -> Self {
        Self::User { content: UserContent::Parts(parts) }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::Assistant { content: content.into(), tool_calls: Vec::new(), thinking: None }
    }

    /// 创建带工具调用的助手消息
    pub fn assistant_with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self::Assistant { content: content.into(), tool_calls, thinking: None }
    }

    /// 创建带思考过程的助手消息
    pub fn assistant_with_thinking(
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
        thinking: serde_json::Value,
    ) -> Self {
        Self::Assistant { content: content.into(), tool_calls, thinking: Some(thinking) }
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

    /// 获取文本内容
    ///
    /// 对于 `User` 消息，如果是多模态内容则拼接所有文本片段。
    pub fn content(&self) -> Cow<'_, str> {
        match self {
            Self::System { content } => Cow::Borrowed(content),
            Self::User { content } => content.text(),
            Self::Assistant { content, .. } => Cow::Borrowed(content),
            Self::Tool { content, .. } => Cow::Borrowed(content),
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

    /// 获取思考过程（仅 Assistant 消息）
    pub fn thinking(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Assistant { thinking: Some(t), .. } => Some(t),
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

#[cfg(test)]
mod tests;
