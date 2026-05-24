//! LLM 提供者抽象层
//!
//! 提供统一的 LLM 提供者接口，支持 OpenAI 兼容服务和 Anthropic Messages API。

mod anthropic;
mod any;
mod base;
mod openai;

pub use anthropic::AnthropicLike;
pub use any::AnyProvider;
pub use base::{ContentPart, Message, Options, Provider, ProviderError, ProviderResponse, ToolCall, UserContent};
pub use openai::OpenAILike;
