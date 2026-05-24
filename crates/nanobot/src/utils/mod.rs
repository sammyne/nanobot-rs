//! 工具函数模块
//!
//! 提供 CLI 命令共用的工具函数。

use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use nanobot_config::HOME;
use nanobot_cron::CronService;
use tracing::debug;

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

/// 同步工作空间模板文件
///
/// 遍历所有内置模板，仅创建不存在的文件（不覆盖已有文件）。
/// 返回新创建的文件相对路径列表。
pub fn sync_workspace_templates(workspace: &Path) -> Result<Vec<&'static str>> {
    let templates = nanobot_templates::all_templates();
    let mut created = Vec::new();

    for (relative_path, content) in &templates {
        let target = workspace.join(relative_path);

        // 确保父目录存在
        if let Some(parent) = target.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).with_context(|| format!("创建模板目录失败: {}", parent.display()))?;
        }

        if target.exists() {
            debug!("模板文件已存在，跳过: {relative_path}");
            continue;
        }

        fs::write(&target, content).with_context(|| format!("创建模板文件失败: {}", target.display()))?;
        created.push(*relative_path);
    }

    Ok(created)
}

#[cfg(test)]
mod tests;
