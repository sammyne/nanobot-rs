//! Heartbeat 组件库
//!
//! 提供周期性心跳检查服务，通过两阶段决策机制（决策阶段+执行阶段）避免不必要的代理唤醒。

pub mod callback;
pub mod config;
pub mod error;
pub mod service;

// Re-export 主要类型
pub use callback::{OnExecuteCallback, OnNotifyCallback};
pub use config::HeartbeatConfig;
pub use error::HeartbeatError;
pub use service::HeartbeatService;

#[cfg(test)]
mod tests;
