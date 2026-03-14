//! 工具函数模块
//!
//! 提供 CLI 命令共用的工具函数。

use std::sync::Arc;

use anyhow::{Context, Result};
use nanobot_config::HOME;
use nanobot_cron::CronService;

/// 初始化 CronService
///
/// 数据文件存储在 `$HOME/.nanobot/cron/jobs.json`。
/// 如果目录不存在，会自动创建。
pub async fn init_cron_service() -> Result<Arc<CronService>> {
    let data_dir = HOME.join(".nanobot");
    let cron_dir = data_dir.join("cron");

    // 确保 cron 目录存在
    tokio::fs::create_dir_all(&cron_dir).await.with_context(|| "创建 cron 目录失败")?;

    let cron_file = cron_dir.join("jobs.json");

    CronService::new(cron_file).await.context("初始化 CronService 失败").map(Arc::new)
}
