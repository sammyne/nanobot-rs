//! 工作空间初始化器模块

use std::fs;

use anyhow::{Context, Result};
use tracing::{debug, info};

/// 工作空间初始化器
pub struct WorkspaceInitializer {
    /// 工作空间根目录
    workspace_path: std::path::PathBuf,
}

impl WorkspaceInitializer {
    /// 创建新的初始化器
    pub fn new(workspace_path: std::path::PathBuf) -> Self {
        Self { workspace_path }
    }

    /// 初始化工作空间
    pub fn initialize(&self) -> Result<()> {
        info!("初始化工作空间: {:?}", self.workspace_path);

        // 1. 创建工作空间根目录
        self.create_workspace_dir()?;

        // 2. 同步模板文件
        let created = crate::utils::sync_workspace_templates(&self.workspace_path)?;
        for name in &created {
            println!("\x1b[32m✓\x1b[0m Created file: {name}");
        }

        // 3. 创建 memory 子目录和非模板文件
        self.create_memory_extras()?;

        // 4. 创建 skills 子目录
        self.create_skills_dir()?;

        Ok(())
    }

    /// 创建工作空间根目录
    fn create_workspace_dir(&self) -> Result<()> {
        if !self.workspace_path.exists() {
            fs::create_dir_all(&self.workspace_path)
                .with_context(|| format!("创建工作空间目录失败: {:?}", self.workspace_path))?;
            println!("\x1b[32m✓\x1b[0m Created workspace at {}", self.workspace_path.display());
        } else {
            debug!("工作空间目录已存在: {:?}", self.workspace_path);
        }
        Ok(())
    }

    /// 创建 memory 子目录中的非模板文件（空的 HISTORY.md）
    ///
    /// memory/ 目录和 MEMORY.md 已由 `sync_workspace_templates` 处理。
    fn create_memory_extras(&self) -> Result<()> {
        let history_file = self.workspace_path.join("memory/HISTORY.md");

        if history_file.exists() {
            debug!("文件已存在，跳过: {:?}", history_file);
            return Ok(());
        }

        // 确保 memory/ 目录存在（sync 可能已创建，但以防万一）
        if let Some(parent) = history_file.parent() {
            fs::create_dir_all(parent).with_context(|| format!("创建 memory 目录失败: {}", parent.display()))?;
        }

        fs::write(&history_file, "").with_context(|| format!("创建文件失败: {}", history_file.display()))?;
        println!("\x1b[32m✓\x1b[0m Created file: memory/HISTORY.md");

        Ok(())
    }

    /// 创建 skills 子目录
    fn create_skills_dir(&self) -> Result<()> {
        let skills_dir = self.workspace_path.join("skills");

        if !skills_dir.exists() {
            fs::create_dir_all(&skills_dir).with_context(|| format!("创建 skills 目录失败: {skills_dir:?}"))?;
            println!("\x1b[32m✓\x1b[0m Created directory: skills/");
        }

        Ok(())
    }
}
