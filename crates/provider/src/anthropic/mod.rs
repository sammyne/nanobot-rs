//! Anthropic Messages API 提供者实现

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use nanobot_config::ProviderConfig;
use nanobot_tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{Message, Options, Provider, ProviderError, ToolCall};

/// Anthropic API 版本
const ANTHROPIC_VERSION: &str = "2023-06-01";

// ============ Anthropic API 请求/响应结构体 ============

/// Anthropic Messages API 请求体
#[derive(Debug, Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u16,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<AnthropicTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
}

/// Anthropic 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<ContentBlock>,
}

/// Anthropic content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    /// 文本内容
    Text { text: String },
    /// 工具调用（请求/响应中）
    ToolUse { id: String, name: String, input: serde_json::Value },
    /// 工具结果（请求中）
    ToolResult { tool_use_id: String, content: String },
    /// 思考过程（响应中，需原样回传）
    Thinking {
        thinking: String,
        #[serde(default)]
        signature: Option<String>,
    },
    /// 图片内容（请求中）
    Image { source: ImageSource },
}

/// Anthropic 图片来源
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageSource {
    r#type: String,
    media_type: String,
    data: String,
}

/// Anthropic 工具定义
#[derive(Debug, Clone, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

/// Anthropic Messages API 响应体
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

/// Anthropic API 错误响应
#[derive(Debug, Deserialize)]
struct AnthropicErrorResponse {
    error: AnthropicErrorDetail,
}

/// Anthropic API 错误详情
#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    message: String,
}

// ============ AnthropicLike 提供者 ============

/// Anthropic Messages API 提供者
#[derive(Clone)]
pub struct AnthropicLike {
    /// HTTP 客户端
    client: reqwest::Client,

    /// API Key
    api_key: String,

    /// API Base URL
    api_base: String,

    /// 模型名称
    model: String,

    /// 请求超时（秒）
    timeout: u64,

    /// 绑定的工具列表（Anthropic 格式）
    tools: Arc<Vec<AnthropicTool>>,

    /// 自定义请求头
    extra_headers: Option<HashMap<String, String>>,
}

impl AnthropicLike {
    /// 创建新的 Anthropic 提供者
    pub fn new(config: &ProviderConfig, model: &str) -> Result<Self> {
        Self::new_with_timeout(config, model, 120)
    }

    /// 创建新的 Anthropic 提供者，指定超时时间
    pub fn new_with_timeout(config: &ProviderConfig, model: &str, timeout: u64) -> Result<Self> {
        let api_base = config.api_base.as_deref().unwrap_or("https://api.anthropic.com/v1");

        info!("初始化 Anthropic 提供者: model={model}, base_url={api_base}");

        let client = reqwest::Client::new();

        Ok(Self {
            client,
            api_key: config.api_key.clone(),
            api_base: api_base.to_string(),
            model: model.to_string(),
            timeout,
            tools: Arc::new(Vec::new()),
            extra_headers: config.extra_headers.clone(),
        })
    }
}

/// 将内部 `Message` 列表转换为 Anthropic 格式
///
/// 返回 `(system, messages)`：
/// - `system`：从 `Message::System` 中提取的系统提示（多个则拼接）
/// - `messages`：转换后的 Anthropic 消息列表
fn convert_messages(messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
    let mut system_parts: Vec<&str> = Vec::new();
    let mut result: Vec<AnthropicMessage> = Vec::new();
    // 用于收集连续的 Tool 消息
    let mut pending_tool_results: Vec<ContentBlock> = Vec::new();

    for msg in messages {
        // 遇到非 Tool 消息时，先 flush 积累的 tool_result blocks
        if !matches!(msg, Message::Tool { .. }) && !pending_tool_results.is_empty() {
            result.push(AnthropicMessage {
                role: "user".to_string(),
                content: std::mem::take(&mut pending_tool_results),
            });
        }

        match msg {
            Message::System { content } => {
                system_parts.push(content);
            }
            Message::User { content } => {
                let blocks = match content {
                    crate::UserContent::Text(text) => {
                        vec![ContentBlock::Text { text: text.clone() }]
                    }
                    crate::UserContent::Parts(parts) => parts
                        .iter()
                        .map(|part| match part {
                            crate::ContentPart::Text { text } => ContentBlock::Text { text: text.clone() },
                            crate::ContentPart::Image { media_type, data } => ContentBlock::Image {
                                source: ImageSource {
                                    r#type: "base64".to_string(),
                                    media_type: media_type.clone(),
                                    data: data.clone(),
                                },
                            },
                        })
                        .collect(),
                };
                result.push(AnthropicMessage { role: "user".to_string(), content: blocks });
            }
            Message::Assistant { content, tool_calls, thinking } => {
                let mut blocks = Vec::new();

                // thinking 放最前面（Anthropic 要求 thinking 在其他 content blocks 之前）
                if let Some(Ok(block)) = thinking.as_ref().map(|v| serde_json::from_value::<ContentBlock>(v.clone())) {
                    blocks.push(block);
                }

                if !content.is_empty() {
                    blocks.push(ContentBlock::Text { text: content.clone() });
                }
                for tc in tool_calls {
                    let input = serde_json::from_str(&tc.arguments).unwrap_or_default();
                    blocks.push(ContentBlock::ToolUse { id: tc.id.clone(), name: tc.name.clone(), input });
                }
                // Anthropic 要求 content 非空
                if blocks.is_empty() {
                    blocks.push(ContentBlock::Text { text: String::new() });
                }
                result.push(AnthropicMessage { role: "assistant".to_string(), content: blocks });
            }
            Message::Tool { tool_call_id, content } => {
                pending_tool_results
                    .push(ContentBlock::ToolResult { tool_use_id: tool_call_id.clone(), content: content.clone() });
            }
        }
    }

    // flush 末尾残留的 tool_result blocks
    if !pending_tool_results.is_empty() {
        result.push(AnthropicMessage { role: "user".to_string(), content: pending_tool_results });
    }

    let system = if system_parts.is_empty() { None } else { Some(system_parts.join("\n")) };

    (system, result)
}

#[async_trait::async_trait]
impl Provider for AnthropicLike {
    async fn chat(&self, messages: &[Message], options: &Options) -> Result<Message> {
        let tools: Vec<AnthropicTool> = if self.tools.is_empty() { Vec::new() } else { (*self.tools).clone() };
        debug!("发送 Anthropic 聊天请求, 消息数量: {}, 工具数量: {}", messages.len(), tools.len());

        let (system, anthropic_messages) = convert_messages(messages);

        let tool_choice = options.tool_choice.as_ref().map(|tc| match tc {
            crate::ToolChoice::Auto => serde_json::json!({"type": "auto"}),
            crate::ToolChoice::Required => serde_json::json!({"type": "any"}),
            crate::ToolChoice::Named(name) => serde_json::json!({"type": "tool", "name": name}),
        });

        let request_body = AnthropicRequest {
            model: &self.model,
            max_tokens: options.max_tokens,
            messages: anthropic_messages,
            system,
            temperature: Some(options.temperature),
            tools,
            tool_choice,
        };

        // 构建 HTTP 请求
        let url = format!("{}/messages", self.api_base.trim_end_matches('/'));
        let mut req = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json");

        // 添加自定义请求头
        if let Some(headers) = &self.extra_headers {
            for (key, value) in headers {
                req = req.header(key, value);
            }
        }

        let req = req.json(&request_body);

        // 发送请求（带超时）
        let response = tokio::time::timeout(Duration::from_secs(self.timeout), req.send())
            .await
            .map_err(|_| ProviderError::Timeout)?
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        // 检查 HTTP 状态码
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let error_msg =
                serde_json::from_str::<AnthropicErrorResponse>(&body).map(|e| e.error.message).unwrap_or(body);

            let err = if status.as_u16() == 429 {
                ProviderError::RateLimit(error_msg)
            } else if status.is_server_error() {
                ProviderError::ServerError(format!("HTTP {status}: {error_msg}"))
            } else {
                ProviderError::Api(format!("HTTP {status}: {error_msg}"))
            };
            return Err(err.into());
        }

        // 读取响应体
        let body = response.text().await.map_err(|e| ProviderError::Api(format!("读取响应体失败: {e}")))?;
        debug!("Anthropic 原始响应: {body}");

        let resp: AnthropicResponse =
            serde_json::from_str(&body).map_err(|e| ProviderError::Api(format!("响应解析失败: {e}")))?;

        // 从响应中提取文本、工具调用和 thinking
        let mut text_parts: Vec<String> = Vec::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut thinking: Option<serde_json::Value> = None;

        for block in resp.content {
            match block {
                ContentBlock::Text { text } => {
                    text_parts.push(text);
                }
                ContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall::new(id, name, input));
                }
                ContentBlock::Thinking { .. } => {
                    // 序列化为不透明 Value，存入 Message::Assistant.thinking
                    thinking = serde_json::to_value(&block).ok();
                }
                ContentBlock::ToolResult { .. } => {}
                ContentBlock::Image { .. } => {} // 图片仅在请求中使用，响应中忽略
            }
        }

        let content = text_parts.join("");

        if !tool_calls.is_empty() {
            info!("收到 Anthropic 响应(带工具调用): {} 个工具调用, 内容长度: {}", tool_calls.len(), content.len());
        } else {
            info!("收到 Anthropic 响应, 长度: {}", content.len());
        }

        match thinking {
            Some(t) => Ok(Message::assistant_with_thinking(content, tool_calls, t)),
            None => Ok(Message::assistant_with_tools(content, tool_calls)),
        }
    }

    fn bind_tools(&mut self, tools: Vec<ToolDefinition>) {
        info!("绑定 {} 个工具到 Anthropic 提供者", tools.len());
        self.tools = Arc::new(
            tools
                .into_iter()
                .map(|td| {
                    debug!(
                        "工具 '{}' input_schema: {}",
                        td.name,
                        serde_json::to_string(&td.parameters).unwrap_or_default()
                    );
                    AnthropicTool { name: td.name, description: td.description, input_schema: td.parameters }
                })
                .collect(),
        );
    }
}

#[cfg(test)]
mod tests;
