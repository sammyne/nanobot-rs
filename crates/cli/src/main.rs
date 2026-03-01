//! Nanobot CLI 入口
//!
//! 极简复现 HKUDS/nanobot 的 onboard 和 agent 命令

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nanobot_cli::{AgentCmd, OnboardCmd, init_logging};

/// Nanobot - AI Agent 命令行工具
#[derive(Parser, Debug)]
#[command(name = "nanobot")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 子命令
    #[command(subcommand)]
    command: Commands,
}

/// 子命令枚举
#[derive(Subcommand, Debug)]
enum Commands {
    /// 配置 LLM 提供者
    Onboard(OnboardCmd),

    /// 启动 AI Agent 交互式对话
    Agent(AgentCmd),
}

#[tokio::main]
async fn main() -> ExitCode {
    // 初始化日志
    init_logging();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Onboard(cmd) => cmd.run(),
        Commands::Agent(cmd) => cmd.run().await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {}", e);
            ExitCode::FAILURE
        }
    }
}
