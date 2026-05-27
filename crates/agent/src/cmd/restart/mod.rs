//! /restart 命令 — 重启 agent 进程

use tracing::info;

use super::Command;
use crate::InboundMessage;

/// 重启命令
pub struct RestartCmd;

impl Command for RestartCmd {
    async fn run(self, _msg: InboundMessage, session_key: String) -> Result<String, String> {
        info!("Processing /restart command: session_key={session_key}");

        let exe = std::env::current_exe().map_err(|e| format!("Failed to get current executable: {e}"))?;
        let args: Vec<String> = std::env::args().skip(1).collect();

        std::process::Command::new(&exe)
            .args(&args)
            .spawn()
            .map_err(|e| format!("Failed to spawn new process: {e}"))?;

        // 延迟退出，让调用方有机会发送响应
        tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            std::process::exit(0);
        });

        Ok("Restarting...".to_string())
    }
}

#[cfg(test)]
mod tests;
