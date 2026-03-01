//! OpenAI 提供者实现

use crate::{Message, Provider, ProviderError};
use anyhow::Result;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionTool, ChatCompletionToolType, CreateChatCompletionRequestArgs,
        FunctionCall, FunctionObject,
    },
};
use nanobot_config::{Config as NanobotConfig, ProviderConfig};
use nanobot_tools::ToolDefinition;
use std::time::Duration;
use tracing::{debug, info};

/// OpenAI 提供者
pub struct OpenAILike {
    /// 客户端
    client: Client<OpenAIConfig>,

    /// 模型名称
    model: String,

    /// 请求超时（秒）
    timeout: u64,

    /// 绑定的工具列表（OpenAI 格式）
    tools: Vec<ChatCompletionTool>,
}

impl OpenAILike {
    /// 创建新的 OpenAI 提供者
    pub fn new(config: &ProviderConfig, model: &str) -> Result<Self> {
        Self::new_with_timeout(config, model, 120)
    }

    /// 创建新的 OpenAI 提供者，指定超时时间
    pub fn new_with_timeout(config: &ProviderConfig, model: &str, timeout: u64) -> Result<Self> {
        let api_base = config
            .api_base
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");

        info!(
            "初始化 OpenAI 提供者: model={}, base_url={}",
            model, api_base
        );

        // 创建自定义配置
        let openai_config = OpenAIConfig::new()
            .with_api_base(api_base)
            .with_api_key(&config.api_key);

        // 创建客户端
        let client = Client::with_config(openai_config);

        Ok(Self {
            client,
            model: model.to_string(),
            timeout,
            tools: Vec::new(),
        })
    }

    /// 从应用配置创建
    pub fn from_config(config: &NanobotConfig) -> Result<Self> {
        let provider_config = config.provider();
        let model = &config.agents.defaults.model;
        Self::new(&provider_config, model)
    }

    /// 转换消息为 OpenAI 格式
    fn convert_messages(&self, messages: &[Message]) -> Result<Vec<ChatCompletionRequestMessage>> {
        let mut result = Vec::new();

        for msg in messages {
            let chat_msg = match msg {
                Message::System { content } => ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(content.as_str())
                        .build()?,
                ),
                Message::User { content } => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(content.as_str())
                        .build()?,
                ),
                Message::Assistant { content, tool_calls } => {
                    let mut assistant_msg = ChatCompletionRequestAssistantMessageArgs::default()
                        .content(content.as_str())
                        .build()?;

                    // 如果有工具调用，添加到消息中
                    if !tool_calls.is_empty() {
                        let openai_tool_calls: Vec<ChatCompletionMessageToolCall> = tool_calls
                            .iter()
                            .map(|tc| {
                                let arguments = tc.parse_arguments().unwrap_or_default();
                                ChatCompletionMessageToolCall {
                                    id: tc.id.clone(),
                                    r#type: ChatCompletionToolType::Function,
                                    function: FunctionCall {
                                        name: tc.name.clone(),
                                        arguments: serde_json::to_string(&arguments).unwrap_or_default(),
                                    },
                                }
                            })
                            .collect();
                        assistant_msg.tool_calls = Some(openai_tool_calls);
                    }

                    ChatCompletionRequestMessage::Assistant(assistant_msg)
                }
                Message::Tool { tool_call_id, content } => {
                    ChatCompletionRequestMessage::Tool(
                        ChatCompletionRequestToolMessageArgs::default()
                            .content(content.as_str())
                            .tool_call_id(tool_call_id)
                            .build()?,
                    )
                }
            };
            result.push(chat_msg);
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl Provider for OpenAILike {
    async fn chat(&self, messages: &[Message]) -> Result<Message> {
        use crate::ToolCall;

        // 工具已经由 bind_tools 转换为 OpenAI 格式，直接使用
        let chat_tools: Vec<ChatCompletionTool> = if self.tools.is_empty() {
            Vec::new()
        } else {
            self.tools.clone()
        };

        debug!(
            "发送聊天请求, 消息数量: {}, 工具数量: {}",
            messages.len(),
            chat_tools.len()
        );

        // 转换消息格式
        let chat_messages = self.convert_messages(messages)?;

        // 构建请求（带工具支持）
        let request = if !chat_tools.is_empty() {
            CreateChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(chat_messages)
                .tools(chat_tools)
                .build()?
        } else {
            CreateChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(chat_messages)
                .build()?
        };

        // 发送请求（带超时）
        let response = tokio::time::timeout(
            Duration::from_secs(self.timeout),
            self.client.chat().create(request),
        )
        .await
        .map_err(|_| ProviderError::Timeout)?
        .map_err(|e| ProviderError::Api(e.to_string()))?;

        // 获取第一个选择
        let choice = response
            .choices
            .first()
            .ok_or_else(|| ProviderError::Api("响应中没有选择".to_string()))?;

        // 提取回复内容
        let content = choice.message.content.clone().unwrap_or_default();

        // 检查是否有工具调用
        if let Some(tool_calls) = &choice.message.tool_calls {
            if !tool_calls.is_empty() {
                let converted_tool_calls: Vec<ToolCall> = tool_calls
                    .iter()
                    .map(|tc| {
                        ToolCall::new(
                            tc.id.clone(),
                            tc.function.name.clone(),
                            serde_json::from_str(&tc.function.arguments).unwrap_or_default(),
                        )
                    })
                    .collect();

                info!(
                    "收到 LLM 响应(带工具调用): {} 个工具调用, 内容长度: {}",
                    converted_tool_calls.len(),
                    content.len()
                );

                // 构造包含工具调用的响应
                return Ok(Message::assistant_with_tools(content, converted_tool_calls));
            }
        }

        info!("收到 LLM 响应, 长度: {}", content.len());

        // 返回普通文本响应
        Ok(Message::assistant(content))
    }

    fn bind_tools(&mut self, tools: Vec<ToolDefinition>) {
        info!("绑定 {} 个工具到 OpenAI 提供者", tools.len());
        self.tools = tools
            .into_iter()
            .map(|td| ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: td.name,
                    description: Some(td.description),
                    parameters: Some(td.parameters),
                    strict: None,
                },
            })
            .collect();
    }
}

#[cfg(test)]
mod tests;
