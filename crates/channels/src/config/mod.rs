//! 配置管理模块
//!
//! 从 nanobot-config crate 重新导出通道配置类型。

// 从 nanobot-config crate 重新导出配置类型
pub use nanobot_config::{ChannelsConfig, DingTalkConfig};

#[cfg(test)]
mod tests;
