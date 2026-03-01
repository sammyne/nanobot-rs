//! 日志配置模块
//!
//! 提供结构化日志输出，支持敏感信息脱敏。

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// 初始化日志系统
///
/// - 输出到 stderr
/// - 支持 RUST_LOG 环境变量控制日志级别
/// - 默认日志级别为 warn
/// - 敏感信息（如 API Key）会被脱敏
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        )
        .init();
}

/// 脱敏敏感信息
///
/// 对 API Key 等敏感信息进行脱敏处理，保留前4位和后4位
///
/// # 示例
///
/// ```
/// use nanobot_cli::logging::mask_sensitive;
///
/// let key = "sk-1234567890abcdefghijklmnop";
/// let masked = mask_sensitive(key);
/// assert!(masked.starts_with("sk-1"));
/// assert!(masked.ends_with("mnop"));
/// ```
pub fn mask_sensitive(s: &str) -> String {
    if s.len() <= 8 {
        return "*".repeat(s.len());
    }

    let start = &s[..4];
    let end = &s[s.len() - 4..];
    format!("{}****{}", start, end)
}

#[cfg(test)]
mod tests;
