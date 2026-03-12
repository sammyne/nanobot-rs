//! 配置管理 crate
//!
//! 提供统一的配置加载、验证和管理功能。

mod schema;

pub use schema::gateway::HeartbeatConfig;
pub use schema::{AgentDefaults, ChannelsConfig, Config, ConfigError, DingTalkConfig, GatewayConfig, ProviderConfig};
