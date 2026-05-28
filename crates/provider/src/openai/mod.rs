//! OpenAI 提供者实现

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
    ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart, ChatCompletionTool,
    ChatCompletionToolType, CreateChatCompletionRequestArgs, FunctionCall, FunctionObject, ImageUrl,
};
use nanobot_config::{Config as NanobotConfig, ProviderConfig};
use nanobot_tools::ToolDefinition;
use tracing::{debug, info};

use crate::{Message, MeteredMessage, Options, Provider, ProviderError, TokenUsage, ToolCall};

/// OpenAI 提供者
#[derive(Clone)]
pub struct OpenAILike {
    /// 客户端
    client: Client<OpenAIConfig>,

    /// 模型名称
    model: String,

    /// 请求超时（秒）
    timeout: u64,

    /// 绑定的工具列表（OpenAI 格式）
    tools: Arc<Vec<ChatCompletionTool>>,
}

impl OpenAILike {
    /// 创建新的 OpenAI 提供者
    pub fn new(config: &ProviderConfig, model: &str) -> Result<Self> {
        Self::new_with_timeout(config, model, 120)
    }

    /// 创建新的 OpenAI 提供者，指定超时时间
    pub fn new_with_timeout(config: &ProviderConfig, model: &str, timeout: u64) -> Result<Self> {
        let api_base = config.api_base.as_deref().unwrap_or("https://api.openai.com/v1");

        info!("初始化 OpenAI 提供者: model={}, base_url={}", model, api_base);

        // 创建自定义配置
        let openai_config = OpenAIConfig::new().with_api_base(api_base).with_api_key(&config.api_key);

        // 创建客户端
        let client = Client::with_config(openai_config);

        Ok(Self { client, model: model.to_string(), timeout, tools: Arc::new(Vec::new()) })
    }

    /// 从应用配置创建
    pub fn from_config(config: &NanobotConfig) -> Result<Self> {
        let provider_config = config.provider();
        let model = &config.agents.defaults.model;
        Self::new(&provider_config, model)
    }
}

impl From<&crate::ContentPart> for ChatCompletionRequestUserMessageContentPart {
    fn from(part: &crate::ContentPart) -> Self {
        match part {
            crate::ContentPart::Text { text } => {
                Self::Text(ChatCompletionRequestMessageContentPartText { text: text.clone() })
            }
            crate::ContentPart::Image { media_type, data } => {
                Self::ImageUrl(ChatCompletionRequestMessageContentPartImage {
                    image_url: ImageUrl { url: format!("data:{media_type};base64,{data}"), detail: None },
                })
            }
        }
    }
}

impl From<&crate::UserContent> for ChatCompletionRequestUserMessageContent {
    fn from(content: &crate::UserContent) -> Self {
        match content {
            crate::UserContent::Text(text) => Self::Text(text.clone()),
            crate::UserContent::Parts(parts) => Self::Array(parts.iter().map(Into::into).collect()),
        }
    }
}

/// 将内部 Message 类型转换为 OpenAI 的请求消息格式
impl TryFrom<&Message> for ChatCompletionRequestMessage {
    type Error = anyhow::Error;

    fn try_from(msg: &Message) -> Result<Self, Self::Error> {
        let chat_msg = match msg {
            Message::System { content } => ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default().content(content.as_str()).build()?,
            ),
            Message::User { content } => {
                let user_content: ChatCompletionRequestUserMessageContent = content.into();
                ChatCompletionRequestMessage::User(user_content.into())
            }
            Message::Assistant { content, tool_calls, .. } => {
                let mut assistant_msg =
                    ChatCompletionRequestAssistantMessageArgs::default().content(content.as_str()).build()?;
                if !tool_calls.is_empty() {
                    assistant_msg.tool_calls = Some(tool_calls.iter().map(Into::into).collect());
                }
                ChatCompletionRequestMessage::Assistant(assistant_msg)
            }
            Message::Tool { tool_call_id, content } => ChatCompletionRequestMessage::Tool(
                ChatCompletionRequestToolMessageArgs::default()
                    .content(content.as_str())
                    .tool_call_id(tool_call_id)
                    .build()?,
            ),
        };
        Ok(chat_msg)
    }
}

#[async_trait::async_trait]
impl Provider for OpenAILike {
    async fn chat(&self, messages: &[Message], options: &Options) -> Result<MeteredMessage> {
        // 工具已经由 bind_tools 转换为 OpenAI 格式，直接使用
        let chat_tools: Vec<ChatCompletionTool> =
            if self.tools.is_empty() { Vec::new() } else { (*self.tools).clone() };
        debug!("发送聊天请求, 消息数量: {}, 工具数量: {}", messages.len(), chat_tools.len());

        // 转换消息格式
        let chat_messages: Vec<ChatCompletionRequestMessage> =
            messages.iter().map(TryInto::try_into).collect::<Result<_>>()?;

        // 构建请求（带工具支持）
        let mut builder = CreateChatCompletionRequestArgs::default();
        builder
            .model(&self.model)
            .messages(chat_messages)
            .max_tokens(options.max_tokens)
            .temperature(options.temperature);

        if let Some(re) = options.reasoning_effort {
            let openai_re = match re {
                nanobot_config::ReasoningEffort::Low => async_openai::types::ReasoningEffort::Low,
                nanobot_config::ReasoningEffort::Medium => async_openai::types::ReasoningEffort::Medium,
                nanobot_config::ReasoningEffort::High => async_openai::types::ReasoningEffort::High,
            };
            builder.reasoning_effort(openai_re);
        }

        if !chat_tools.is_empty() {
            builder.tools(chat_tools);
        }

        if let Some(ref tc) = options.tool_choice {
            use async_openai::types::{ChatCompletionNamedToolChoice, ChatCompletionToolChoiceOption, FunctionName};
            let choice = match tc {
                crate::ToolChoice::Auto => ChatCompletionToolChoiceOption::Auto,
                crate::ToolChoice::Required => ChatCompletionToolChoiceOption::Required,
                crate::ToolChoice::Named(name) => {
                    ChatCompletionToolChoiceOption::Named(ChatCompletionNamedToolChoice {
                        r#type: async_openai::types::ChatCompletionToolType::Function,
                        function: FunctionName { name: name.clone() },
                    })
                }
            };
            builder.tool_choice(choice);
        }

        let request = builder.build()?;

        // 发送请求（带超时），使用 byot 直接反序列化为自定义响应结构以提取非标准字段（如 reasoning_content）
        let resp: response::ChatCompletionResponse =
            tokio::time::timeout(Duration::from_secs(self.timeout), self.client.chat().create_byot(request))
                .await
                .map_err(|_| ProviderError::Timeout)?
                .map_err(|e| ProviderError::Api(e.to_string()))?;

        resp.try_into()
    }

    fn bind_tools(&mut self, tools: Vec<ToolDefinition>) {
        info!("绑定 {} 个工具到 OpenAI 提供者", tools.len());
        self.tools = Arc::new(
            tools
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
                .collect(),
        );
    }
}

mod response;

impl TryFrom<response::ChatCompletionResponse> for MeteredMessage {
    type Error = anyhow::Error;

    fn try_from(resp: response::ChatCompletionResponse) -> Result<Self> {
        let usage = resp.usage.map(|u| {
            let cached = u.prompt_tokens_details.and_then(|d| d.cached_tokens).filter(|&v| v > 0);
            TokenUsage { input: u.prompt_tokens, output: u.completion_tokens, cached }
        });

        let choice =
            resp.choices.into_iter().next().ok_or_else(|| ProviderError::Api("响应中没有 choices".to_string()))?;
        let content = choice.message.content.unwrap_or_default();
        let reasoning = choice.message.reasoning_content;

        let tool_calls: Vec<ToolCall> = choice
            .message
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| {
                ToolCall::new(tc.id, tc.function.name, serde_json::from_str(&tc.function.arguments).unwrap_or_default())
            })
            .collect();

        let message = if !tool_calls.is_empty() {
            info!("收到 LLM 响应(带工具调用): {} 个工具调用, 内容长度: {}", tool_calls.len(), content.len());
            let mut msg = Message::assistant_with_tools(content, tool_calls);
            if let (Some(rc), Message::Assistant { thinking, .. }) = (reasoning, &mut msg) {
                *thinking = Some(serde_json::Value::String(rc));
            }
            msg
        } else {
            info!("收到 LLM 响应, 长度: {}", content.len());
            match reasoning {
                Some(rc) => Message::assistant_with_thinking(&content, Vec::new(), serde_json::Value::String(rc)),
                None => Message::assistant(content),
            }
        };

        Ok(MeteredMessage { message, usage })
    }
}

#[cfg(test)]
mod tests;

/// 将 OpenAI 的工具调用格式转换为内部 ToolCall 类型
impl From<ChatCompletionMessageToolCall> for ToolCall {
    fn from(tc: ChatCompletionMessageToolCall) -> Self {
        Self::new(tc.id, tc.function.name, serde_json::from_str(&tc.function.arguments).unwrap_or_default())
    }
}

/// 将内部 ToolCall 类型转换为 OpenAI 的工具调用格式
impl From<&ToolCall> for ChatCompletionMessageToolCall {
    fn from(tc: &ToolCall) -> Self {
        let arguments = tc.parse_arguments::<serde_json::Value>().unwrap_or_default();
        Self {
            id: tc.id.clone(),
            r#type: ChatCompletionToolType::Function,
            function: FunctionCall {
                name: tc.name.clone(),
                arguments: serde_json::to_string(&arguments).unwrap_or_default(),
            },
        }
    }
}
