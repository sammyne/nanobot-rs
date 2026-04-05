//! Agent 循环处理库
//!
//! 提供 AgentLoop 核心实现，负责接收消息、构建上下文、调用 LLM 并返回响应。
//!
//! # 进度通知功能
//!
//! 本库提供进度追踪功能，允许用户实时了解 Agent 的思考和工具调用过程：
//!
//! - [`ProgressTracker`] - 进度追踪器 trait
//! - [`ChannelProgressTracker`] - 通过消息通道发送进度的默认实现
//!
//! ## 使用示例
//!
//! ### 场景 1：交互式模式（使用默认回调）
//!
//! ```rust,ignore
//! use nanobot_agent::AgentLoop;
//! use nanobot_provider::MockProvider;
//! use nanobot_config::AgentDefaults;
//! use tokio::sync::mpsc;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = MockProvider::new();
//! let config = AgentDefaults::default();
//! // let agent = AgentLoop::new(provider, config, None, None, Default::default()).await?;
//!
//! // run 方法内部会自动创建默认进度回调（ChannelProgressTracker）
//! // agent.run(inbound_rx, outbound_tx).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 场景 2：直接调用模式（自定义闭包）
//!
//! ```rust,ignore
//! use nanobot_agent::{ProgressTracker, AgentLoop};
//! use nanobot_provider::MockProvider;
//! use nanobot_config::AgentDefaults;
//! use std::sync::Arc;
//! use anyhow::Result;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 自定义进度回调（闭包直接实现 trait）
//! let callback = |content: String, is_tool_hint: bool| {
//!     Box::pin(async move {
//!         println!("[Progress] {} (tool_hint={})", content, is_tool_hint);
//!         Ok(())
//!     })
//! };
//!
//! // let agent = AgentLoop::new(provider, config, None, None, Default::default()).await?;
//! // let result = agent.process_direct(
//! //     "帮我分析这个文件",
//! //     "cli:direct",
//! //     None,
//! //     None,
//! //     Some(Arc::new(callback)),
//! // ).await?;
//! # Ok(())
//! # }
//! ```

mod cmd;
mod r#loop;
mod progress;
mod utils;

pub use r#loop::AgentLoop;
// Re-export 消息类型（来自 nanobot-channels）
pub use nanobot_channels::messages::{InboundMessage, OutboundMessage};
// Re-export 依赖 crate 的类型，方便使用
pub use nanobot_config::AgentDefaults;
pub use nanobot_provider::{Message, Provider};
pub use nanobot_session::{Session, SessionManager};
pub use progress::{ChannelProgressTracker, ProgressTracker};
