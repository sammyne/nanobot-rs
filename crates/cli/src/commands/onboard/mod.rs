//! Onboard 命令 - 配置 LLM 提供者

use anyhow::Result;
use clap::Args;
use dialoguer::Confirm;
use nanobot_config::Config;
use tracing::info;

/// Onboard 命令参数
#[derive(Args, Debug)]
pub struct OnboardArgs {}

/// 执行 onboard 命令
pub fn run(_args: OnboardArgs) -> Result<()> {
    info!("开始 onboard 配置");

    // 获取配置文件路径
    let config_path = Config::config_path()?;

    if config_path.exists() {
        println!("\x1b[33mConfig already exists at {}\x1b[0m", config_path.display());
        println!("  \x1b[1my\x1b[0m = overwrite with defaults (existing values will be lost)");
        println!("  \x1b[1mN\x1b[0m = refresh config, keeping existing values and adding new fields");

        let overwrite = Confirm::new()
            .with_prompt("Overwrite?")
            .default(false)
            .interact()?;

        if overwrite {
            let config = Config::default();
            config.save()?;
            println!("\x1b[32m✓\x1b[0m Config reset to defaults at {}", config_path.display());
        } else {
            let config = Config::load()?;
            config.save()?;
            println!(
                "\x1b[32m✓\x1b[0m Config refreshed at {} (existing values preserved)",
                config_path.display()
            );
        }
    } else {
        Config::default().save()?;
        println!("\x1b[32m✓\x1b[0m Created config at {}", config_path.display());
    }

    // 输出准备就绪信息
    println!("\n⚡ nanobot is ready!");
    println!("\nNext steps:");
    println!("  1. Add your API key to \x1b[36m~/.nanobot/config.json\x1b[0m");
    println!("     Get one at: https://openrouter.ai/keys");
    println!("  2. Chat: \x1b[36mnanobot agent -m \"Hello!\"\x1b[0m");
    println!(
        "\n\x1b[2mWant Telegram/WhatsApp? See: https://github.com/HKUDS/nanobot#-chat-apps\x1b[0m"
    );

    Ok(())
}

#[cfg(test)]
mod tests;
