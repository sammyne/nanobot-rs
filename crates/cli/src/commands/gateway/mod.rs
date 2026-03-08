//! Gateway 命令 - 启动 nanobot 后台服务
//!
//! Gateway 是 nanobot 的核心服务入口，负责初始化并协调所有后台服务的运行。
//! 主要功能包括：
//! - 初始化 LLM Provider
//! - 启动 AgentLoop 消息处理引擎
//! - 启动 ChannelManager 管理各渠道通道
//! - 提供优雅的启动和关闭机制

use anyhow::{Context, Result};
use clap::Args;
use nanobot_agent::{AgentLoop, InboundMessage, OutboundMessage};
use nanobot_channels::ChannelManager;
use nanobot_config::Config;
use nanobot_provider::OpenAILike;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Gateway 命令
#[derive(Args, Debug)]
pub struct GatewayCmd {
    /// 服务端口（默认使用配置文件的 gateway.port，若未配置则使用 18790）
    #[arg(short, long)]
    pub port: Option<u16>,
}

impl GatewayCmd {
    /// 执行 gateway 命令
    pub async fn run(&self) -> Result<()> {
        // 加载配置（先加载配置以获取端口）
        let config = self.load_config()?;

        // 确定实际使用的端口：命令行参数优先，否则使用配置文件值
        let (actual_port, port_source) = match self.port {
            Some(port) => (port, "命令行"),
            None => (config.gateway.port, "配置文件"),
        };

        info!("启动 nanobot gateway (port={})", actual_port);

        // 显示启动信息
        self.print_startup_banner(actual_port, port_source);

        // 初始化 LLM Provider
        let provider = self.init_provider(&config)?;

        // 创建消息通道
        let (inbound_tx, inbound_rx) = mpsc::channel::<InboundMessage>(100);
        let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundMessage>(100);

        // 创建 AgentLoop
        let agent_loop = AgentLoop::new(provider, config.agents.defaults.clone(), inbound_rx, outbound_tx);

        // 使用 Config 中的 channels 配置创建 ChannelManager
        let channel_manager = ChannelManager::new(config.channels.clone(), outbound_rx, inbound_tx)
            .await
            .context("创建通道管理器失败")?;

        // 启动服务并等待关闭信号
        self.run_services(agent_loop, channel_manager).await?;

        info!("Gateway 服务已停止");
        Ok(())
    }

    /// 显示启动横幅
    fn print_startup_banner(&self, port: u16, port_source: &str) {
        println!();
        println!("  ╔═══════════════════════════════════════╗");
        println!("  ║         🤖 Nanobot Gateway            ║");
        println!("  ╚═══════════════════════════════════════╝");
        println!();
        println!("  🚀 启动 nanobot gateway on port {}...", port);
        println!("  📋 端口来源: {}", port_source);
    }

    /// 加载配置
    fn load_config(&self) -> Result<Config> {
        Config::load().context(
            "加载配置失败。请先运行 'nanobot onboard' 进行配置，\
             或检查 ~/.nanobot/config.json 文件是否存在。",
        )
    }

    /// 初始化 LLM Provider
    fn init_provider(&self, config: &Config) -> Result<OpenAILike> {
        let provider_config = config.provider();

        // 验证 API Key
        if provider_config.api_key.is_empty() {
            anyhow::bail!(
                "API Key 未配置。请在 ~/.nanobot/config.json 中设置 provider.api_key。\n\
                 获取 API Key: https://openrouter.ai/keys"
            );
        }

        info!(
            "LLM Provider 初始化: model={}, base_url={}",
            config.agents.defaults.model,
            provider_config.api_base.as_deref().unwrap_or("(默认)")
        );

        OpenAILike::from_config(config).context("初始化 LLM Provider 失败")
    }

    /// 显示通道状态
    async fn print_channel_status(&self, channel_manager: &ChannelManager) {
        let status = channel_manager.get_status().await;

        if status.is_empty() {
            println!();
            println!("  ⚠️  警告: 没有启用的通道");
            println!("     请在 ~/.nanobot/config.json 中配置 channels 字段");
            println!();
        } else {
            println!();
            println!("  ✓ 已启用的通道:");
            for s in status {
                let status_icon = if s.running { "🟢" } else { "🔴" };
                println!(
                    "    {} {} ({})",
                    status_icon,
                    s.name,
                    if s.running { "运行中" } else { "已停止" }
                );
            }
            println!();
        }
    }

    /// 启动服务并等待关闭信号
    async fn run_services(&self, agent_loop: AgentLoop<OpenAILike>, mut channel_manager: ChannelManager) -> Result<()> {
        // 启动 AgentLoop 后台任务
        let agent_task = tokio::spawn(async move {
            if let Err(e) = agent_loop.run().await {
                error!("AgentLoop 运行失败: {}", e);
            }
        });

        // 启动所有通道
        channel_manager.start_all().await.context("启动通道失败")?;

        // 显示通道状态（在启动所有通道后）
        self.print_channel_status(&channel_manager).await;

        println!("  ✓ 服务已启动，按 Ctrl+C 停止");
        println!();

        // 等待 Ctrl+C 信号
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("收到中断信号，开始优雅关闭...");
                println!("\n  🛑 正在关闭服务...");
            }
            Err(e) => {
                error!("信号监听失败: {}", e);
            }
        }

        // 优雅关闭
        self.shutdown(agent_task, channel_manager).await?;

        println!("  ✓ 服务已停止");
        println!();

        Ok(())
    }

    /// 优雅关闭所有服务
    async fn shutdown(
        &self,
        agent_task: tokio::task::JoinHandle<()>,
        mut channel_manager: ChannelManager,
    ) -> Result<()> {
        println!("    ↦ 停止 AgentLoop...");
        agent_task.abort();

        println!("    ↦ 停止 ChannelManager...");
        if let Err(e) = channel_manager.stop_all().await {
            warn!("停止 ChannelManager 时发生错误: {}", e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
