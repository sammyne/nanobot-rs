//! Agent 命令 - 启动 AI Agent 交互式对话

use anyhow::Result;
use clap::Args;
use nanobot_agent::AgentLoop;
use nanobot_config::Config;
use nanobot_provider::{Message, OpenAIProvider, Provider};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
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
        let config = Config::load().map_err(|e| {
            anyhow::anyhow!("加载配置失败: {}。请先运行 'nanobot onboard' 进行配置。", e)
        })?;

        let provider_config = config.provider();

        info!(
            "配置加载成功: model={}, base_url={}",
            config.agents.defaults.model,
            provider_config.api_base.as_deref().unwrap_or("(默认)")
        );

        // 初始化 LLM Provider
        let provider = Arc::new(OpenAIProvider::from_config(&config)?);
        debug!("LLM Provider 初始化成功");

        // 初始化对话历史
        let mut messages: Vec<Message> = Vec::new();

        // 添加系统提示词
        messages.push(Message::system("你是一个有帮助的 AI 助手。"));

        if let Some(msg) = &self.message {
            // 单次消息模式
            self.run_once(provider.clone(), &config, msg).await?;
        } else {
            // 交互式模式
            self.run_interactive(provider.clone(), &mut messages, &config.agents.defaults.model).await?;
        }

        Ok(())
    }

    /// 单次消息模式
    async fn run_once(
        &self,
        provider: Arc<dyn Provider>,
        config: &Config,
        input: &str,
    ) -> Result<()> {
        debug!("单次消息模式");

        // 创建 AgentLoop 实例
        let agent = AgentLoop::new(provider, config.agents.defaults.clone());

        match agent.process_direct(input).await {
            Ok(response) => {
                println!("{}", response);
            }
            Err(e) => {
                error!("Agent 处理失败: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// 交互式模式
    async fn run_interactive(
        &self,
        provider: Arc<dyn Provider>,
        messages: &mut Vec<Message>,
        model: &str,
    ) -> Result<()> {
        debug!("交互式模式");

        // 打印欢迎信息
        println!("🤖 Nanobot Agent - 交互式 AI 助手");
        println!("模型: {}", model);
        println!("输入 'exit' 或 'quit' 退出\n");

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            // 读取用户输入
            print!("你: ");
            stdout.flush()?;

            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;
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

            // 添加用户消息到历史
            messages.push(Message::user(input));

            // 调用 LLM
            debug!("发送请求到 LLM, 消息数量: {}", messages.len());

            match provider.chat(&messages).await {
                Ok(response) => {
                    println!("\n助手: {}\n", response);

                    // 添加助手回复到历史
                    messages.push(Message::assistant(response));
                }
                Err(e) => {
                    error!("LLM 调用失败: {}", e);
                    eprintln!("\n错误: {}\n", e);

                    // 移除失败的用户消息
                    messages.pop();
                }
            }
        }

        Ok(())
    }
}

/// 检查是否为退出命令
fn is_exit_command(input: &str) -> bool {
    EXIT_COMMANDS.contains(&input.to_lowercase().as_str())
}

#[cfg(test)]
mod tests;
