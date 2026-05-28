//! /status 命令实现

use std::time::Instant;

use nanobot_provider::Usage;

use super::Command;
use crate::InboundMessage;

/// /status 命令
pub struct StatusCmd {
    /// 模型名称
    pub model: String,
    /// 启动时间
    pub start_time: Instant,
    /// 最近一次 LLM 调用的 token 用量
    pub last_usage: Option<Usage>,
    /// 当前会话消息数
    pub session_message_count: usize,
}

impl Command for StatusCmd {
    async fn run(self, _msg: InboundMessage, _session_key: String) -> Result<String, String> {
        let version = env!("CARGO_PKG_VERSION");
        let secs = self.start_time.elapsed().as_secs();
        let uptime = format_uptime(secs);

        let (tokens_in, tokens_out) = match &self.last_usage {
            Some(u) => (format!("{}", u.input_tokens), format!("{}", u.output_tokens)),
            None => ("N/A".to_string(), "N/A".to_string()),
        };

        Ok(format!(
            "\
🐈 nanobot v{version}
Model: {model}
Tokens (last call): {tokens_in} in / {tokens_out} out
Session messages: {msg_count}
Uptime: {uptime}",
            model = self.model,
            msg_count = self.session_message_count,
        ))
    }
}

fn format_uptime(total_secs: u64) -> String {
    let days = total_secs / 86400;
    let hours = total_secs % 86400 / 3600;
    let mins = total_secs % 3600 / 60;
    let secs = total_secs % 60;

    if days > 0 {
        format!("{days}d {hours}h {mins}m {secs}s")
    } else if hours > 0 {
        format!("{hours}h {mins}m {secs}s")
    } else if mins > 0 {
        format!("{mins}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

#[cfg(test)]
mod tests;
