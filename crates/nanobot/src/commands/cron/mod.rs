//! Cron 命令 - 管理定时任务

use anyhow::Result;
use chrono::{TimeZone, Utc};
use clap::{Args, Subcommand};

use crate::utils::init_cron_service;

/// Cron 命令
#[derive(Args, Debug)]
pub struct CronCmd {
    /// 子命令
    #[command(subcommand)]
    command: CronSubcommand,
}

impl CronCmd {
    /// 执行 cron 命令
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            CronSubcommand::Enable(cmd) => cmd.run().await,
            CronSubcommand::Run(cmd) => cmd.run().await,
        }
    }
}

/// Cron 子命令
#[derive(Subcommand, Debug)]
enum CronSubcommand {
    /// 启用或禁用定时任务
    Enable(EnableCmd),

    /// 立即执行定时任务
    Run(RunCmd),
}

// ========== Helper Functions ==========

/// 格式化时间戳为可读字符串
fn format_time(ts_ms: Option<i64>) -> String {
    match ts_ms {
        Some(ts) => {
            let dt = Utc.timestamp_millis_opt(ts).single();
            match dt {
                Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                None => "无效时间".to_string(),
            }
        }
        None => "-".to_string(),
    }
}

// ========== Enable Command ==========

/// 启用或禁用定时任务
#[derive(Args, Debug)]
struct EnableCmd {
    /// 任务 ID
    job_id: String,

    /// 禁用任务
    #[arg(short, long)]
    disable: bool,
}

impl EnableCmd {
    async fn run(&self) -> Result<()> {
        let cron_service = init_cron_service().await?;

        let job = cron_service.enable_job(&self.job_id, !self.disable).await;

        match job {
            Some(job) => {
                let action = if self.disable { "禁用" } else { "启用" };
                println!();
                println!("✓ 任务 {} 已{}", job.name, action);
                if !self.disable {
                    println!("  下次执行: {}", format_time(job.state.next_run_at_ms));
                }
                println!();
            }
            None => {
                println!();
                println!("⚠ 任务 {} 不存在", self.job_id);
                println!();
            }
        }

        Ok(())
    }
}

// ========== Run Command ==========

/// 立即执行定时任务
#[derive(Args, Debug)]
struct RunCmd {
    /// 任务 ID
    job_id: String,

    /// 强制执行已禁用的任务
    #[arg(short, long)]
    force: bool,
}

impl RunCmd {
    async fn run(&self) -> Result<()> {
        let cron_service = init_cron_service().await?;

        // 获取任务
        let jobs = cron_service.list_jobs(true).await;
        let job = jobs.iter().find(|j| j.id == self.job_id);

        match job {
            Some(job) => {
                if !job.enabled && !self.force {
                    println!();
                    println!("⚠ 任务 {} 已禁用，使用 --force 强制执行", self.job_id);
                    println!();
                    return Ok(());
                }

                println!();
                println!("执行任务: {} ({})", job.name, job.id);
                println!("消息: {}", job.payload.message);
                println!();
                println!("注意：此命令仅标记任务为立即执行，实际执行由 CronService 调度器处理。");
                println!("      在 gateway 或 agent 交互模式下，任务会自动执行。");
                println!();

                // 启用任务（如果被禁用且使用 --force）
                if !job.enabled && self.force {
                    cron_service.enable_job(&self.job_id, true).await;
                    println!("✓ 任务已临时启用");
                }
            }
            None => {
                println!();
                println!("⚠ 任务 {} 不存在", self.job_id);
                println!();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
