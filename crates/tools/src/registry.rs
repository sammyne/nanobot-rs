//! 工具注册表
//!
//! 管理所有可用的工具，提供注册、查询和执行功能。

use std::collections::HashMap;

use serde_json::Value;
use tracing::{error, info};

use crate::core::{Tool, ToolContext, ToolDefinition, ToolError, ToolResult};

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// 创建注册表并注册默认工具
    pub fn new(workspace: &str, allowed_dir: Option<&str>) -> Self {
        use crate::fs::{EditFileTool, ListDirTool, ReadFileTool, WriteFileTool};
        use crate::shell::ShellTool;

        let mut registry = Self { tools: HashMap::new() };

        let read_tool = ReadFileTool::new(workspace, allowed_dir);
        info!("注册工具: {}", read_tool.name());
        registry
            .tools
            .insert(read_tool.name().to_string(), Box::new(read_tool) as Box<dyn Tool>);

        let write_tool = WriteFileTool::new(workspace, allowed_dir);
        info!("注册工具: {}", write_tool.name());
        registry
            .tools
            .insert(write_tool.name().to_string(), Box::new(write_tool) as Box<dyn Tool>);

        let edit_tool = EditFileTool::new(workspace, allowed_dir);
        info!("注册工具: {}", edit_tool.name());
        registry
            .tools
            .insert(edit_tool.name().to_string(), Box::new(edit_tool) as Box<dyn Tool>);

        let list_tool = ListDirTool::new(workspace, allowed_dir);
        info!("注册工具: {}", list_tool.name());
        registry
            .tools
            .insert(list_tool.name().to_string(), Box::new(list_tool) as Box<dyn Tool>);

        let shell_tool = ShellTool::new(workspace);
        info!("注册工具: {}", shell_tool.name());
        registry
            .tools
            .insert(shell_tool.name().to_string(), Box::new(shell_tool) as Box<dyn Tool>);

        info!("已注册 {} 个默认工具", registry.tools.len());

        registry
    }

    /// 注册工具
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        info!("注册工具: {}", name);
        self.tools.insert(name, Box::new(tool));
    }

    /// 注销工具
    pub fn unregister(&mut self, name: &str) -> bool {
        info!("注销工具: {}", name);
        self.tools.remove(name).is_some()
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| &**t)
    }

    /// 获取所有工具名称
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// 获取所有工具定义（用于 OpenAI Function Calling）
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.to_definition()).collect()
    }

    /// 异步执行指定工具
    pub async fn execute(&self, ctx: &ToolContext, name: &str, params: Value) -> ToolResult {
        let tool = self.tools.get(name).ok_or_else(|| {
            let available = self.tool_names().join(", ");
            ToolError::NotFound(format!("工具 '{name}' 不存在。可用工具: [{available}]"))
        })?;

        info!("执行工具: {} 参数: {:?}", name, params);

        let result = tool.execute(ctx, params).await;

        if let Err(ref e) = result {
            error!("工具 {} 执行失败: {:?}", name, e);
        }

        result
    }

    /// 检查是否包含指定工具
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}
