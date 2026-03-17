//! 日志适配器
//!
//! 将 feishu-sdk 的日志桥接到 tracing 生态

use feishu_sdk::core::LogLevel;

/// Custom logger that forwards SDK logs to tracing
#[derive(Debug)]
pub struct TracingLogger;

impl feishu_sdk::core::Logger for TracingLogger {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Debug => tracing::debug!("[Feishu] {}", message),
            LogLevel::Info => tracing::info!("[Feishu] {}", message),
            LogLevel::Warn => tracing::warn!("[Feishu] {}", message),
            LogLevel::Error => tracing::error!("[Feishu] {}", message),
        }
    }

    fn is_enabled(&self, level: LogLevel) -> bool {
        match level {
            LogLevel::Debug => tracing::level_enabled!(tracing::Level::DEBUG),
            LogLevel::Info => tracing::level_enabled!(tracing::Level::INFO),
            LogLevel::Warn => tracing::level_enabled!(tracing::Level::WARN),
            LogLevel::Error => tracing::level_enabled!(tracing::Level::ERROR),
        }
    }
}
