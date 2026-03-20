//! Gateway 命令 - 启动 nanobot 后台服务
//!
//! Gateway 是 nanobot 的核心服务入口，负责初始化并协调所有后台服务的运行。
//! 主要功能包括：
//! - 初始化 LLM Provider
//! - 启动 AgentLoop 消息处理引擎
//! - 启动 ChannelManager 管理各渠道通道
//! - 启动 CronService 管理定时任务
//! - 提供优雅的启动和关闭机制

mod health_check;

use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use nanobot_agent::{AgentLoop, InboundMessage, OutboundMessage};
use nanobot_channels::ChannelManager;
use nanobot_config::{Config, HeartbeatConfig as GatewayHeartbeatConfig};
use nanobot_cron::{CronJob, CronService};
use nanobot_heartbeat::config::HeartbeatConfig as HeartbeatServiceConfig;
use nanobot_heartbeat::{HeartbeatService, OnExecuteCallback, OnNotifyCallback};
use nanobot_provider::{OpenAILike, Provider};
use nanobot_session::SessionInfo;
use nanobot_subagent::SubagentManager;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::utils::init_cron_service;

/// Gateway 命令
#[derive(Args, Debug)]
pub struct GatewayCmd {
    /// 服务端口（默认使用配置文件的 gateway.port，若未配置则使用 18790）
    #[arg(short, long)]
    pub port: Option<u16>,

    /// 健康检查服务端口（默认使用配置文件的 gateway.healthCheck.port，若未配置则使用 7860）
    #[arg(long)]
    pub health_check_port: Option<u16>,
}

/// 服务运行时的上下文
struct ServicesContext<P: Provider + Send + Sync + Clone + 'static> {
    /// AgentLoop 实例
    agent_loop: Arc<AgentLoop<P>>,
    /// ChannelManager 实例
    channel_manager: ChannelManager,
    /// CronService 实例
    cron_service: Arc<CronService>,
    /// HeartbeatService 实例
    heartbeat_service: HeartbeatService<P>,
    /// 入站消息接收端
    inbound_rx: mpsc::Receiver<InboundMessage>,
    /// 出站消息发送端
    outbound_tx: mpsc::Sender<OutboundMessage>,
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

        // 确定健康检查端口：命令行参数优先，否则使用配置文件值
        let health_check_port = self.health_check_port.or(config.gateway.health_check_port);

        info!("启动 nanobot gateway (port={})", actual_port);

        // 显示启动信息
        self.print_startup_banner(actual_port, port_source);

        // 初始化 LLM Provider
        let provider = self.init_provider(&config)?;

        // 创建消息通道
        let (inbound_tx, inbound_rx) = mpsc::channel::<InboundMessage>(100);
        let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundMessage>(100);

        // 初始化 CronService（不设置回调，后续复用 AgentLoop）
        let cron_service = init_cron_service().await?;

        // 创建 SubagentManager（使用 inbound_tx 用于子代理完成通知）
        let subagent_manager = SubagentManager::new(
            provider.clone(),
            config.agents.defaults.workspace.clone(),
            inbound_tx.clone(),
            config.agents.defaults.temperature as f32,
            config.agents.defaults.max_tokens as u32,
        );

        // 创建 AgentLoop
        let agent_loop = Arc::new(
            AgentLoop::new(
                provider.clone(),
                config.agents.defaults.clone(),
                Some(cron_service.clone()),
                Some(subagent_manager),
                config.tools.clone(),
            )
            .await?,
        );

        // 设置 cron 回调，复用同一个 AgentLoop
        self.setup_cron_callback(&cron_service, agent_loop.clone()).await;

        // 使用 Config 中的 channels 配置创建 ChannelManager
        let channel_manager = ChannelManager::new(config.channels.clone(), outbound_rx, inbound_tx)
            .await
            .context("创建通道管理器失败")?;

        // 初始化 HeartbeatService（在创建 AgentLoop 和 ChannelManager 之后）
        let heartbeat_service = self.setup_heartbeat_service(
            config.agents.defaults.workspace.clone(),
            provider,
            &config.gateway.heartbeat,
            agent_loop.clone(),
            outbound_tx.clone(),
        )?;

        // 启动健康检查服务（后台任务运行）
        if let Some(port) = health_check_port {
            tokio::spawn(health_check::serve(port));
        }

        // 启动服务并等待关闭信号
        self.run_services(
            ServicesContext { agent_loop, channel_manager, cron_service, heartbeat_service, inbound_rx, outbound_tx },
            &config.gateway.heartbeat,
            health_check_port,
        )
        .await?;

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
        println!("  🚀 启动 nanobot gateway on port {port}...");
        println!("  📋 端口来源: {port_source}");
    }

    /// 显示服务启动状态
    async fn print_service_status(
        &self,
        channel_manager: &ChannelManager,
        heartbeat_config: &GatewayHeartbeatConfig,
        health_check_port: Option<u16>,
    ) {
        println!();
        println!("  ┌─────────────────────────────────────┐");
        println!("  │           服务状态                   │");
        println!("  └─────────────────────────────────────┘");

        // 显示通道状态
        let status = channel_manager.get_status().await;

        if status.is_empty() {
            println!("  ⚠️  警告: 没有启用的通道");
            println!("     请在 ~/.nanobot/config.json 中配置 channels 字段");
        } else {
            println!("  ✓ 已启用的通道:");
            for s in status {
                let status_icon = if s.running { "🟢" } else { "🔴" };
                println!("    {} {} ({})", status_icon, s.name, if s.running { "运行中" } else { "已停止" });
            }
        }

        // 显示 HeartbeatService 状态
        if heartbeat_config.enabled {
            println!("  ✓ HeartbeatService: 已启用 (间隔: {}s)", heartbeat_config.interval_s);
        } else {
            println!("  ✓ HeartbeatService: 已禁用");
        }

        // 显示健康检查服务状态
        if let Some(port) = health_check_port {
            println!("  ✓ 健康检查服务: 已启用 (端口: {port})");
        } else {
            println!("  ✓ 健康检查服务: 未配置");
        }

        println!();
    }

    /// 加载配置
    fn load_config(&self) -> Result<Config> {
        Config::load()
            .context(
                "加载配置失败。请先运行 'nanobot onboard' 进行配置，\
                 或检查 ~/.nanobot/config.json 文件是否存在。",
            )?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "配置文件不存在。请先运行 'nanobot onboard' 进行配置，\
                     或检查 ~/.nanobot/config.json 文件是否存在。"
                )
            })
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

    /// 设置 HeartbeatService（包含回调）
    ///
    /// # 参数
    ///
    /// * `workspace_path` - 工作区路径
    /// * `provider` - LLM Provider
    /// * `config` - Gateway 配置中的 heartbeat 配置
    /// * `agent_loop` - AgentLoop 实例
    /// * `outbound_tx` - 出站消息发送端
    ///
    /// # 返回
    ///
    /// 返回配置好回调的 HeartbeatService 实例
    fn setup_heartbeat_service(
        &self,
        workspace_path: std::path::PathBuf,
        provider: OpenAILike,
        config: &GatewayHeartbeatConfig,
        agent_loop: Arc<AgentLoop<OpenAILike>>,
        outbound_tx: mpsc::Sender<OutboundMessage>,
    ) -> Result<HeartbeatService<OpenAILike>> {
        info!("设置 HeartbeatService: enabled={}, interval_s={}", config.enabled, config.interval_s);

        // 转换配置格式
        let heartbeat_config = HeartbeatServiceConfig::with_values(config.enabled, config.interval_s);

        // 创建 SessionManager 用于访问会话列表
        let sessions = Arc::new(nanobot_session::SessionManager::new(workspace_path.clone()));

        // 创建 on_execute 回调（函数指针形式）
        let on_execute: OnExecuteCallback = Arc::new({
            let agent = agent_loop.clone();
            let sessions = sessions.clone();
            move |task_summary: &str| {
                let agent = agent.clone();
                let sessions = sessions.clone();
                let task_summary = task_summary.to_string();
                Box::pin(async move {
                    info!("Heartbeat on_execute: {}", task_summary);

                    // 获取会话列表
                    let sessions_list = sessions.list_sessions();

                    // 获取已启用的渠道列表（目前暂时为空，后续可以从配置获取）
                    let enabled_channels = vec![];

                    // 选择目标渠道
                    let (channel, chat_id) = Self::pick_heartbeat_target(&enabled_channels, &sessions_list);

                    // 使用 "heartbeat" 作为 session_key
                    let session_key = "heartbeat";

                    // 调用 AgentLoop::process_direct
                    match agent.process_direct(&task_summary, session_key, Some(&channel), Some(&chat_id)).await {
                        Ok(response) => {
                            info!("Heartbeat 任务执行成功");
                            Ok(response)
                        }
                        Err(e) => {
                            error!("Heartbeat 任务执行失败: {}", e);
                            Err(anyhow::anyhow!(e))
                        }
                    }
                })
            }
        });

        // 创建 on_notify 回调（函数指针形式）
        let on_notify: OnNotifyCallback = Arc::new({
            let sessions = sessions.clone();
            move |result: &str| {
                let sessions = sessions.clone();
                let result = result.to_string();
                let outbound_tx = outbound_tx.clone();
                Box::pin(async move {
                    info!("Heartbeat on_notify: {}", result);

                    // 获取会话列表
                    let sessions_list = sessions.list_sessions();

                    // 获取已启用的渠道列表（目前暂时为空，后续可以从配置获取）
                    let enabled_channels = vec![];

                    // 选择目标渠道
                    let (channel, chat_id) = Self::pick_heartbeat_target(&enabled_channels, &sessions_list);

                    // 跳过 "cli" 目标
                    if channel == "cli" {
                        info!("跳过 cli 渠道的通知");
                        return Ok(());
                    }

                    // 发送 OutboundMessage
                    let msg = OutboundMessage::new(&channel, &chat_id, &result);
                    if let Err(e) = outbound_tx.send(msg).await {
                        error!("发送心跳通知失败: {}", e);
                        return Err(anyhow::anyhow!(e));
                    }

                    Ok(())
                })
            }
        });

        // 创建 HeartbeatService（带回调）
        let heartbeat_service =
            HeartbeatService::new(workspace_path, provider, heartbeat_config, Some(on_execute), Some(on_notify));

        Ok(heartbeat_service)
    }

    /// 设置 cron 回调，复用同一个 AgentLoop
    async fn setup_cron_callback(&self, cron_service: &Arc<CronService>, agent_loop: Arc<AgentLoop<OpenAILike>>) {
        let callback: nanobot_cron::JobCallback = Arc::new(move |job: CronJob| {
            let agent = agent_loop.clone();

            Box::pin(async move {
                let payload = &job.payload;

                // 使用 payload 中的消息作为输入
                let message = if payload.message.is_empty() {
                    // 如果没有消息，使用任务名称
                    job.name.clone()
                } else {
                    payload.message.clone()
                };

                // 执行消息
                match agent.process_direct(&message, "cli:direct", None, None).await {
                    Ok(response) => {
                        info!("Cron job '{}' executed successfully", job.id);
                        Ok(response)
                    }
                    Err(e) => {
                        error!("Cron job '{}' execution failed: {}", job.id, e);
                        Err(e.to_string())
                    }
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
        });

        cron_service.set_on_job_callback(callback).await;
    }

    /// 启动服务并等待关闭信号
    async fn run_services(
        &self,
        ctx: ServicesContext<OpenAILike>,
        heartbeat_config: &GatewayHeartbeatConfig,
        health_check_port: Option<u16>,
    ) -> Result<()> {
        // 启动 AgentLoop 后台任务（传递通道给 run）
        let agent_task = tokio::spawn(async move {
            if let Err(e) = ctx.agent_loop.run(ctx.inbound_rx, ctx.outbound_tx).await {
                error!("AgentLoop 运行失败: {}", e);
            }
        });

        // 启动所有通道
        let mut channel_manager = ctx.channel_manager;
        channel_manager.start_all().await.context("启动通道失败")?;

        // 启动 CronService
        ctx.cron_service.start().await;
        info!("CronService 已启动");

        // 启动 HeartbeatService
        let heartbeat_service = Arc::new(ctx.heartbeat_service);
        let heartbeat_interval = heartbeat_config.interval_s;
        match heartbeat_service.clone().start().await {
            Ok(()) => {
                info!("HeartbeatService 已启动 (间隔: {}s)", heartbeat_interval);
            }
            Err(nanobot_heartbeat::error::HeartbeatError::Disabled) => {
                info!("HeartbeatService 已禁用");
            }
            Err(e) => {
                error!("HeartbeatService 启动失败: {}", e);
            }
        }

        // HealthCheckService 已在 run 方法中启动

        // 显示服务状态（在启动所有通道后）
        self.print_service_status(&channel_manager, heartbeat_config, health_check_port).await;

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
        self.shutdown(agent_task, channel_manager, ctx.cron_service, heartbeat_service).await?;

        println!("  ✓ 服务已停止");
        println!();

        Ok(())
    }

    /// 优雅关闭所有服务
    async fn shutdown(
        &self,
        agent_task: tokio::task::JoinHandle<()>,
        mut channel_manager: ChannelManager,
        cron_service: Arc<CronService>,
        heartbeat_service: Arc<HeartbeatService<OpenAILike>>,
    ) -> Result<()> {
        println!("    ↦ 停止 HeartbeatService...");
        heartbeat_service.stop().await;

        println!("    ↦ 停止 AgentLoop...");
        agent_task.abort();

        println!("    ↦ 停止 CronService...");
        cron_service.stop().await;
        info!("CronService 已停止");

        println!("    ↦ 停止 ChannelManager...");
        if let Err(e) = channel_manager.stop_all().await {
            warn!("停止 ChannelManager 时发生错误: {}", e);
        }

        Ok(())
    }

    /// 选择心跳通知的目标渠道和聊天ID
    ///
    /// # 参数
    ///
    /// * `enabled_channels` - 已启用的渠道名称集合
    /// * `sessions` - 所有会话信息列表，按更新时间降序排列
    ///
    /// # 返回
    ///
    /// 返回 (channel, chat_id) 元组，用于指定心跳通知的目标
    fn pick_heartbeat_target(enabled_channels: &[String], sessions: &[SessionInfo]) -> (String, String) {
        // 优先选择最近更新的非内部会话
        for session in sessions {
            let key = &session.key;

            // 解析 session_key 格式: "channel:chat_id"
            if let Some((channel, chat_id)) = key.split_once(':') {
                // 跳过内部渠道 (cli, system)
                if matches!(channel, "cli" | "system") {
                    continue;
                }

                // 检查渠道是否已启用
                if enabled_channels.contains(&channel.to_string()) && !chat_id.is_empty() {
                    info!("选择心跳目标: {} ({})", channel, chat_id);
                    return (channel.to_string(), chat_id.to_string());
                }
            }
        }

        // 默认返回 cli 渠道
        info!("使用默认心跳目标: cli:direct");
        ("cli".to_string(), "direct".to_string())
    }
}

#[cfg(test)]
mod tests;
