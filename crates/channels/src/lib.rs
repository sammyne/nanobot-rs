//! # nanobot-channels
//!
//! 通道实现 crate
//!
//! 提供统一的通道抽象接口和多种消息通道的实现。
//!
//! ## 模块结构
//!
//! - [`traits`][]: 核心通道抽象
//! - [`messages`][]: 消息类型定义
//! - [`error`][]: 错误处理
//! - [`config`][]: 配置管理
//! - [`manager`][]: 通道管理器
//! - [`dingtalk`][]: 钉钉通道实现
//!
//! [`traits`]: traits
//! [`messages`]: messages
//! [`error`]: error
//! [`config`]: config
//! [`manager`]: manager
//! [`dingtalk`]: dingtalk
//!
//! ## 示例
//!
//! ```rust,no_run
//! use nanobot_channels::{
//!     manager::ChannelManager,
//!     config::ChannelsConfig,
//!     traits::Channel,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 加载配置（使用 serde 反序列化）
//! let yaml_content = std::fs::read_to_string("config.yaml")?;
//! let config: ChannelsConfig = serde_yaml::from_str(&yaml_content)?;
//!
//! // 创建通道管理器
//! let mut manager = ChannelManager::new(config).await?;
//!
//! // 启动所有通道
//! manager.start_all().await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod dingtalk;
pub mod error;
pub mod manager;
pub mod messages;
pub mod traits;

// 重新导出常用类型
pub use config::DingTalkConfig;
pub use dingtalk::DingTalk;
pub use error::{ChannelError, ChannelResult};
pub use manager::ChannelManager;
pub use messages::{InboundMessage, OutboundMessage};
pub use traits::Channel;
