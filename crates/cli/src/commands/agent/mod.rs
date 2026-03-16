//! Agent 命令 - 启动 AI Agent 交互式对话

use std::io::{
    Write, {self},
};
use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use nanobot_agent::{AgentLoop, InboundMessage, OutboundMessage};
use nanobot_config::Config;
use nanobot_cron::CronService;
use nanobot_provider::OpenAILike;
use nanobot_subagent::SubagentManager;
use tracing::{debug, error, info};
/// 退出命令集合
const EXIT_COMMANDS: &[&str] = &["exit", "quit", "/exit", "/quit", ":q"];

/// Agent 命令
#[derive(Args, Debug)]
pub struct AgentCmd {
    /// 发送给 agent 的消息（不提供则进入交互模式）
    #[arg(short, long)]
    pub message: Option<String>,

    /// 会话 ID（格式: channel:chat_id）
    #[arg(short, long, default_value = "cli:direct")]
    pub session: String,
}

impl AgentCmd {
    /// 执行 agent 命令
    pub async fn run(&self) -> Result<()> {
        info!("启动 agent 命令");

        // 加载配置
        let config =
            Config::load().map_err(|e| anyhow::anyhow!("加载配置失败: {e}。请先运行 'nanobot onboard' 进行配置。"))?;

        let provider_config = config.provider();

        info!(
            "配置加载成功: model={}, base_url={}",
            config.agents.defaults.model,
            provider_config.api_base.as_deref().unwrap_or("(默认)")
        );

        // 初始化 LLM Provider
        let provider = OpenAILike::from_config(&config)?;
        debug!("LLM Provider 初始化成功");

        // 初始化 CronService（使用配置中的 workspace）
        let cron_service = self.init_cron_service(&config.agents.defaults.workspace).await?;

        if let Some(msg) = &self.message {
            // 单次消息模式
            self.run_once(provider, &config, &cron_service, msg).await
        } else {
            // 交互式模式 - 使用 MessageBus 和 AgentLoop
            self.run_interactive(provider, &config, &cron_service).await
        }
    }

    /// 单次消息模式
    async fn run_once(
        &self,
        provider: OpenAILike,
        config: &Config,
        cron_service: &Arc<CronService>,
        input: &str,
    ) -> Result<()> {
        debug!("单次消息模式");

        // 准备 MCP 配置
        let mcp_configs = config.tools.mcp_servers.clone();

        // 创建 AgentLoop 实例（不使用子代理功能）
        let agent =
            AgentLoop::new(provider, config.agents.defaults.clone(), Some(cron_service.clone()), None, mcp_configs)
                .await?;

        match agent.process_direct(input, &self.session, None, None).await {
            Ok(response) => {
                println!("{response}");
                Ok(())
            }
            Err(e) => {
                error!("Agent 处理失败: {}", e);
                Err(e)
            }
        }
    }

    /// 交互式模式 - 使用 mpsc 通道
    ///
    /// 设计说明：
    /// - 使用 Tokio mpsc 通道分离发送端和接收端，避免锁竞争
    /// - CLI 持有 inbound_tx（发送用户输入）和 outbound_rx（接收助手回复）
    /// - AgentLoop 持有 inbound_rx（接收用户输入）和 outbound_tx（发送助手回复）
    async fn run_interactive(
        &self,
        provider: OpenAILike,
        config: &Config,
        cron_service: &Arc<CronService>,
    ) -> Result<()> {
        debug!("交互式模式（使用 mpsc 通道）");

        // 解析 session_id
        let (channel, chat_id) = Self::parse_session_id(&self.session);
        let session_key = format!("{channel}:{chat_id}");

        // 创建消息通道对
        // CLI 持有: inbound_tx, outbound_rx
        // AgentLoop 持有: inbound_rx, outbound_tx
        let (inbound_tx, inbound_rx) = tokio::sync::mpsc::channel::<InboundMessage>(100);
        let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::channel::<OutboundMessage>(100);

        // 创建 SubagentManager（使用 inbound_tx 用于子代理完成通知）
        let subagent_manager = SubagentManager::new(
            provider.clone(),
            config.agents.defaults.workspace.clone(),
            inbound_tx.clone(),
            config.agents.defaults.temperature as f32,
            config.agents.defaults.max_tokens as u32,
        );

        // 准备 MCP 配置
        let mcp_configs = config.tools.mcp_servers.clone();

        // 创建 AgentLoop（不再传递通道）
        let agent_loop = AgentLoop::new(
            provider,
            config.agents.defaults.clone(),
            Some(cron_service.clone()),
            Some(subagent_manager),
            mcp_configs,
        )
        .await?;

        // 打印欢迎信息
        println!("🤖 Nanobot Agent - 交互式 AI 助手");
        println!("模型: {}", config.agents.defaults.model);
        println!("会话: {session_key}");
        println!("输入 'exit' 或 'quit' 退出\n");

        // 启动 AgentLoop 后台任务（传递通道给 run）
        let agent_task = tokio::spawn(async move {
            if let Err(e) = agent_loop.run(inbound_rx, outbound_tx).await {
                error!("AgentLoop 运行失败: {}", e);
            }
        });

        // 主循环：读取用户输入，发送消息，并等待响应
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            // 读取用户输入
            print!("你: ");
            stdout.flush()?;

            let mut input = String::new();
            stdin.read_line(&mut input)?;
            let input = input.trim();

            // 检查退出命令
            if is_exit_command(input) {
                println!("再见！");
                break;
            }

            // 跳过空输入
            if input.is_empty() {
                continue;
            }

            // 发送入站消息到 AgentLoop
            let inbound_msg = InboundMessage::new(&channel, "user", &chat_id, input);
            if let Err(e) = inbound_tx.send(inbound_msg).await {
                error!("发送消息失败: {}", e);
                eprintln!("\n❌ 错误: 无法发送消息\n");
                continue;
            }

            // 立即显示 "thinking" 提示
            println!("  ↦ nanobot is thinking...");

            // 等待 AgentLoop 的响应
            loop {
                match outbound_rx.recv().await {
                    Some(msg) => {
                        if msg.is_progress() {
                            println!("  ↦ {}\n", msg.content);
                        } else {
                            println!("\n🤖 助手: {}\n", msg.content);
                            // 接收到完整响应，跳出内层循环
                            break;
                        }
                    }
                    None => {
                        // 通道已关闭，退出
                        eprintln!("\n⚠️  连接已断开\n");
                        return Ok(());
                    }
                }
            }
        }

        // 等待后台任务完成（通过关闭 inbound_tx 触发退出）
        drop(inbound_tx);
        agent_task.abort();

        Ok(())
    }

    /// 初始化 CronService（用于工具操作，不启动定时器）
    ///
    /// 设计说明：
    /// - CLI 进程是短期的，仅用于管理定时任务（通过 CronTool 进行 CRUD 操作）
    /// - 不需要实际触发定时任务执行，实际执行由长期运行的后端服务负责
    /// - 因此不需要设置 callback 或启动定时器
    /// - cron 任务文件存储在 workspace/cron/jobs.json
    async fn init_cron_service(&self, workspace: &std::path::Path) -> Result<Arc<CronService>> {
        // cron 任务存储路径: workspace/cron/jobs.json
        let cron_dir = workspace.join("cron");

        // 确保 cron 目录存在
        tokio::fs::create_dir_all(&cron_dir).await?;

        let cron_file = cron_dir.join("jobs.json");
        debug!("CronService 数据文件: {:?}", cron_file);

        // 创建 CronService（不启动定时器）
        let cron_service = Arc::new(CronService::new(cron_file).await?);
        debug!("CronService 已初始化（仅用于工具操作）");

        Ok(cron_service)
    }

    /// 解析 session_id 为 (channel, chat_id)
    fn parse_session_id(session_id: &str) -> (String, String) {
        let parts: Vec<&str> = session_id.splitn(2, ':').collect();
        if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("cli".to_string(), session_id.to_string())
        }
    }
}

/// 检查是否为退出命令
fn is_exit_command(input: &str) -> bool {
    EXIT_COMMANDS.contains(&input.to_lowercase().as_str())
}

#[cfg(test)]
mod tests;
