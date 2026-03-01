//! Nanobot CLI 入口
//!
//! 极简复现 HKUDS/nanobot 的 onboard 和 agent 命令

use clap::{Parser, Subcommand};
use nanobot_cli::{AgentArgs, OnboardArgs, init_logging};
use std::process::ExitCode;

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
    Onboard(OnboardArgs),

    /// 启动 AI Agent 交互式对话
    Agent(AgentArgs),
}

#[tokio::main]
async fn main() -> ExitCode {
    // 初始化日志
    init_logging();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Onboard(args) => nanobot_cli::commands::onboard::run(args),
        Commands::Agent(args) => nanobot_cli::commands::agent::run(args).await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {}", e);
            ExitCode::FAILURE
        }
    }
}
