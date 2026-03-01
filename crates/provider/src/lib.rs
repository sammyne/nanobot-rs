//! LLM 提供者抽象层
//!
//! 提供统一的 LLM 提供者接口，支持 OpenAI 和兼容 OpenAI 的服务商。

mod base;
mod openai;

pub use base::{Message, Provider, ProviderError, ProviderResponse, ToolCall};
pub use openai::OpenAILike;
