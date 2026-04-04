//! Nanobot CLI 入口
//!
//! 极简复现 HKUDS/nanobot 的 onboard、agent 和 gateway 命令

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nanobot::{AgentCmd, CronCmd, GatewayCmd, OnboardCmd, VERSION, logging};

/// Nanobot - AI Agent 命令行工具
#[derive(Parser, Debug)]
#[command(name = "nanobot")]
#[command(author, version=VERSION, about, long_about = None)]
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

    /// 启动 nanobot 后台服务
    Gateway(GatewayCmd),

    /// 管理定时任务
    Cron(CronCmd),
}

#[tokio::main]
async fn main() -> ExitCode {
    // 初始化日志
    logging::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Onboard(cmd) => cmd.run(),
        Commands::Agent(cmd) => cmd.run().await,
        Commands::Gateway(cmd) => cmd.run().await,
        Commands::Cron(cmd) => cmd.run().await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {e}");
            ExitCode::FAILURE
        }
    }
}
