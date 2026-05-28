//! OpenAI Chat Completion 响应结构（用于 byot 反序列化）
//!
//! 只定义我们需要的字段，其余通过默认行为忽略。

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChoiceMessage,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ChoiceMessage {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<RawToolCall>>,
    /// 非标准字段：Kimi、DeepSeek-R1、MiMo 等模型的思考过程
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawToolCall {
    pub id: String,
    pub function: RawFunction,
}

#[derive(Debug, Deserialize)]
pub struct RawFunction {
    pub name: String,
    #[serde(default)]
    pub arguments: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct PromptTokensDetails {
    pub cached_tokens: Option<u64>,
}
