//! 日志配置模块
//!
//! 提供结构化日志输出，支持敏感信息脱敏。

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

/// 初始化日志系统
///
/// - 输出到 stderr
/// - 支持 RUST_LOG 环境变量控制日志级别
/// - 默认日志级别为 info
/// - 敏感信息（如 API Key）会被脱敏
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        )
        .init();
}
