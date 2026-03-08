//! Agent 命令 - 启动 AI Agent 交互式对话

use std::io::{
    Write, {self},
};

use anyhow::Result;
use clap::Args;
use nanobot_agent::{AgentLoop, InboundMessage, OutboundMessage};
use nanobot_config::Config;
use nanobot_provider::OpenAILike;
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

        if let Some(msg) = &self.message {
            // 单次消息模式
            self.run_once(provider, &config, msg).await?;
        } else {
            // 交互式模式 - 使用 MessageBus 和 AgentLoop
            self.run_interactive(provider, &config).await?;
        }

        Ok(())
    }

    /// 单次消息模式
    async fn run_once(&self, provider: OpenAILike, config: &Config, input: &str) -> Result<()> {
        debug!("单次消息模式");

        // 创建 AgentLoop 实例（简单模式，直接调用）
        let mut agent = AgentLoop::new_direct(provider, config.agents.defaults.clone());

        match agent.process_direct(input, Some(&self.session)).await {
            Ok(response) => {
                println!("{response}");
            }
            Err(e) => {
                error!("Agent 处理失败: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// 交互式模式 - 使用 mpsc 通道
    ///
    /// 设计说明：
    /// - 使用 Tokio mpsc 通道分离发送端和接收端，避免锁竞争
    /// - CLI 持有 inbound_tx（发送用户输入）和 outbound_rx（接收助手回复）
    /// - AgentLoop 持有 inbound_rx（接收用户输入）和 outbound_tx（发送助手回复）
    async fn run_interactive(&self, provider: OpenAILike, config: &Config) -> Result<()> {
        debug!("交互式模式（使用 mpsc 通道）");

        // 解析 session_id
        let (channel, chat_id) = Self::parse_session_id(&self.session);
        let session_key = format!("{channel}:{chat_id}");

        // 创建消息通道对
        // CLI 持有: inbound_tx, outbound_rx
        // AgentLoop 持有: inbound_rx, outbound_tx
        let (inbound_tx, inbound_rx) = tokio::sync::mpsc::channel::<InboundMessage>(100);
        let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::channel::<OutboundMessage>(100);

        // 创建 AgentLoop（传递通道）
        let agent_loop = AgentLoop::new(provider, config.agents.defaults.clone(), inbound_rx, outbound_tx);

        // 打印欢迎信息
        println!("🤖 Nanobot Agent - 交互式 AI 助手");
        println!("模型: {}", config.agents.defaults.model);
        println!("会话: {session_key}");
        println!("输入 'exit' 或 'quit' 退出\n");

        // 启动 AgentLoop 后台任务
        let agent_task = tokio::spawn(async move {
            if let Err(e) = agent_loop.run().await {
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
