//! 配置管理 crate
//!
//! 提供统一的配置加载、验证和管理功能。

mod schema;

// 公开导出 HOME 全局变量
pub use schema::gateway::HeartbeatConfig;
pub use schema::mcp::McpServerConfig;
pub use schema::{
    AgentDefaults, ChannelsConfig, Config, ConfigError, DingTalkConfig, GatewayConfig, HOME, ProviderConfig,
    ToolsConfig,
};
