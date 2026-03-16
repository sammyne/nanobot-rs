//! Agent 循环处理库
//!
//! 提供 AgentLoop 核心实现，负责接收消息、构建上下文、调用 LLM 并返回响应。

mod r#loop;
mod utils;

// Re-export 主要类型
pub use r#loop::AgentLoop;
// Re-export 消息类型（来自 nanobot-channels）
pub use nanobot_channels::messages::{InboundMessage, OutboundMessage};
// Re-export 依赖 crate 的类型，方便使用
pub use nanobot_config::AgentDefaults;
pub use nanobot_provider::{Message, Provider};
pub use nanobot_session::{Session, SessionManager};
