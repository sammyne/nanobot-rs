//! 工作空间初始化器模块

use std::fs;
use std::path::Path;

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

        // 2. 创建根级别模板文件
        self.create_root_templates()?;

        // 3. 创建 memory 子目录和文件
        self.create_memory_dir()?;

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

    /// 创建根级别模板文件
    fn create_root_templates(&self) -> Result<()> {
        let templates = vec![
            ("USER.md", nanobot_templates::user_template()),
            ("AGENTS.md", nanobot_templates::agents_template()),
            ("SOUL.md", nanobot_templates::soul_template()),
            ("TOOLS.md", nanobot_templates::tools_template()),
            ("HEARTBEAT.md", nanobot_templates::heartbeat_template()),
        ];

        for (filename, content) in templates {
            self.create_file_if_not_exists(&self.workspace_path.join(filename), content)?;
        }

        Ok(())
    }

    /// 创建 memory 子目录和文件
    fn create_memory_dir(&self) -> Result<()> {
        let memory_dir = self.workspace_path.join("memory");

        if !memory_dir.exists() {
            fs::create_dir_all(&memory_dir).with_context(|| format!("创建 memory 目录失败: {memory_dir:?}"))?;
            println!("\x1b[32m✓\x1b[0m Created directory: memory/");
        }

        // 创建 MEMORY.md
        self.create_file_if_not_exists(&memory_dir.join("MEMORY.md"), nanobot_templates::memory_template())?;

        // 创建空的 HISTORY.md
        self.create_file_if_not_exists(&memory_dir.join("HISTORY.md"), "")?;

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

    /// 如果文件不存在则创建文件
    fn create_file_if_not_exists(&self, path: &Path, content: &str) -> Result<()> {
        if path.exists() {
            debug!("文件已存在，跳过: {:?}", path);
            return Ok(());
        }

        fs::write(path, content).with_context(|| format!("创建文件失败: {path:?}"))?;

        let relative_path = path.strip_prefix(&self.workspace_path).unwrap_or(path);
        println!("\x1b[32m✓\x1b[0m Created file: {}", relative_path.display());

        Ok(())
    }
}
