//! Agent 循环处理库
//!
//! 提供 AgentLoop 核心实现，负责接收消息、构建上下文、调用 LLM 并返回响应。
//!
//! # 生命周期钩子
//!
//! 本库通过 [`Hook`] trait 提供 Agent 循环各阶段的扩展点：
//!
//! - [`Hook`] - 生命周期钩子 trait
//! - [`LoopHook`] - 交互式循环钩子（通过消息通道发送进度）
//! - [`CompositeHook`] - 组合多个钩子
//! - [`NoopHook`] - 空操作钩子

mod cmd;
mod hook;
mod r#loop;
pub(crate) mod tools;
mod utils;

pub use hook::{CompositeHook, Hook, HookCtx, LoopHook, NoopHook};
pub use r#loop::{AgentLoop, strip_think};
// Re-export 消息类型（来自 nanobot-channels）
pub use nanobot_channels::messages::{InboundMessage, OutboundMessage};
// Re-export 依赖 crate 的类型，方便使用
pub use nanobot_config::AgentDefaults;
pub use nanobot_provider::{Message, Provider};
pub use nanobot_session::{Session, SessionManager};
