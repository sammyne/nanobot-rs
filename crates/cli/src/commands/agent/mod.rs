//! Agent 命令 - 启动 AI Agent 交互式对话

use anyhow::Result;
use clap::Args;
use nanobot_config::Config;
use nanobot_provider::{Message, OpenAIProvider, Provider};
use std::io::{self, BufRead, Write};
use tracing::{debug, error, info};

/// Agent 命令参数
#[derive(Args, Debug)]
pub struct AgentArgs {
    /// 系统提示词
    #[arg(short, long)]
    pub system: Option<String>,
}

/// 执行 agent 命令
pub async fn run(args: AgentArgs) -> Result<()> {
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
    let provider = OpenAIProvider::from_config(&config)?;
    debug!("LLM Provider 初始化成功");

    // 初始化对话历史
    let mut messages: Vec<Message> = Vec::new();

    // 添加系统提示词
    if let Some(system) = args.system {
        messages.push(Message::system(system));
    } else {
        messages.push(Message::system("你是一个有帮助的 AI 助手。"));
    }

    // 打印欢迎信息
    println!("Nanobot Agent - 交互式 AI 助手");
    println!("模型: {}", config.agents.defaults.model);
    println!("输入 'exit' 或 'quit' 退出\n");

    // REPL 循环
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
        if input == "exit" || input == "quit" {
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

#[cfg(test)]
mod tests;
