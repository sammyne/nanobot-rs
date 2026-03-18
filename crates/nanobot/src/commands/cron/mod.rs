//! Cron 命令 - 管理定时任务

use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use clap::{Args, Subcommand};
use nanobot_cron::CronSchedule;

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
            CronSubcommand::List(cmd) => cmd.run().await,
            CronSubcommand::Add(cmd) => cmd.run().await,
            CronSubcommand::Remove(cmd) => cmd.run().await,
            CronSubcommand::Enable(cmd) => cmd.run().await,
            CronSubcommand::Run(cmd) => cmd.run().await,
        }
    }
}

/// Cron 子命令
#[derive(Subcommand, Debug)]
enum CronSubcommand {
    /// 列出所有定时任务
    List(ListCmd),

    /// 添加定时任务
    Add(AddCmd),

    /// 删除定时任务
    Remove(RemoveCmd),

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

/// 格式化调度规则为可读字符串
fn format_schedule(schedule: &CronSchedule) -> String {
    match schedule {
        CronSchedule::Every { every_ms } => format!("每 {} 秒", every_ms / 1000),
        CronSchedule::Cron { expr, tz } => {
            if let Some(tz) = tz {
                format!("Cron: {expr} ({tz})")
            } else {
                format!("Cron: {expr}")
            }
        }
        CronSchedule::At { at_ms } => {
            format!("一次: {}", format_time(Some(*at_ms)))
        }
    }
}

// ========== List Command ==========

/// 列出定时任务
#[derive(Args, Debug)]
struct ListCmd {
    /// 包含已禁用的任务
    #[arg(short, long)]
    all: bool,
}

impl ListCmd {
    async fn run(&self) -> Result<()> {
        let cron_service = init_cron_service().await?;
        let jobs = cron_service.list_jobs(self.all).await;

        if jobs.is_empty() {
            println!("没有定时任务");
            return Ok(());
        }

        println!();
        println!("定时任务列表 (共 {} 个):", jobs.len());
        println!();
        println!("{:<8} {:<20} {:<25} {:<8} {:<22}", "ID", "名称", "调度规则", "状态", "下次执行");
        println!("{}", "-".repeat(90));

        for job in jobs {
            let status = if job.enabled { "启用" } else { "禁用" };
            let next_run = format_time(job.state.next_run_at_ms);
            let schedule = format_schedule(&job.schedule);

            // 截断过长的名称
            let name = match nanobot_utils::strings::truncate(&job.name, 15) {
                Some(truncated) => format!("{truncated}..."),
                None => job.name.clone(),
            };

            println!("{:<8} {:<20} {:<25} {:<8} {}", job.id, name, schedule, status, next_run);
        }

        println!();
        Ok(())
    }
}

// ========== Add Command ==========

/// 添加定时任务
#[derive(Args, Debug)]
struct AddCmd {
    /// 任务名称
    #[arg(short, long)]
    name: String,

    /// 要执行的消息
    #[arg(short, long)]
    message: String,

    /// 间隔秒数（如：--every 60 表示每60秒执行一次）
    #[arg(long, value_name = "SECONDS")]
    every: Option<u64>,

    /// Cron 表达式（6字段格式：秒 分 时 日 月 周，如：--cron "0 0 9 * * 1-5" 表示工作日早9点）
    #[arg(long, value_name = "EXPR")]
    cron: Option<String>,

    /// 指定执行时间（ISO 8601 格式，如：--at "2024-12-25T09:00:00Z"）
    #[arg(long, value_name = "TIME")]
    at: Option<String>,

    /// 时区（仅与 --cron 配合使用，如：--tz "Asia/Shanghai"）
    #[arg(long, value_name = "TZ")]
    tz: Option<String>,
}

impl AddCmd {
    async fn run(&self) -> Result<()> {
        // 验证参数
        let schedule = self.build_schedule()?;

        let cron_service = init_cron_service().await?;

        let job = cron_service
            .add_job(
                self.name.clone(),
                schedule,
                self.message.clone(),
                false, // deliver
                None,  // channel
                None,  // to
                false, // delete_after_run
            )
            .await
            .map_err(|e| anyhow::anyhow!("添加任务失败: {e}"))?;

        println!();
        println!("✓ 定时任务已创建");
        println!();
        println!("  ID: {}", job.id);
        println!("  名称: {}", job.name);
        println!("  调度: {}", format_schedule(&job.schedule));
        println!("  下次执行: {}", format_time(job.state.next_run_at_ms));
        println!();

        Ok(())
    }

    fn build_schedule(&self) -> Result<CronSchedule> {
        // 检查互斥参数
        let schedule_count =
            [self.every.is_some(), self.cron.is_some(), self.at.is_some()].iter().filter(|&&x| x).count();

        if schedule_count == 0 {
            anyhow::bail!("请指定调度方式：--every、--cron 或 --at");
        }

        if schedule_count > 1 {
            anyhow::bail!("--every、--cron 和 --at 参数互斥，只能指定一个");
        }

        // --tz 只能与 --cron 配合使用
        if self.tz.is_some() && self.cron.is_none() {
            anyhow::bail!("--tz 参数只能与 --cron 配合使用");
        }

        if let Some(seconds) = self.every {
            if seconds == 0 {
                anyhow::bail!("--every 值必须大于 0");
            }
            return Ok(CronSchedule::Every { every_ms: (seconds as i64) * 1000 });
        }

        if let Some(ref expr) = self.cron {
            // 验证 cron 表达式
            if let Err(e) =
                nanobot_cron::validate_schedule(&CronSchedule::Cron { expr: expr.clone(), tz: self.tz.clone() })
            {
                anyhow::bail!("无效的 cron 表达式: {e}");
            }
            return Ok(CronSchedule::Cron { expr: expr.clone(), tz: self.tz.clone() });
        }

        if let Some(ref time_str) = self.at {
            // 解析 ISO 8601 时间
            let dt = chrono::DateTime::parse_from_rfc3339(time_str)
                .context("无效的时间格式，请使用 ISO 8601 格式（如：2024-12-25T09:00:00Z）")?;

            let at_ms = dt.timestamp_millis();

            // 检查时间是否已过
            if at_ms < Utc::now().timestamp_millis() {
                anyhow::bail!("指定的执行时间已过");
            }

            return Ok(CronSchedule::At { at_ms });
        }

        unreachable!()
    }
}

// ========== Remove Command ==========

/// 删除定时任务
#[derive(Args, Debug)]
struct RemoveCmd {
    /// 任务 ID
    job_id: String,
}

impl RemoveCmd {
    async fn run(&self) -> Result<()> {
        let cron_service = init_cron_service().await?;

        let removed = cron_service.remove_job(&self.job_id).await;

        if removed {
            println!();
            println!("✓ 任务 {} 已删除", self.job_id);
            println!();
        } else {
            println!();
            println!("⚠ 任务 {} 不存在", self.job_id);
            println!();
        }

        Ok(())
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
