//! 子代理管理器
//!
//! SubagentManager 负责创建、管理和监控子代理任务的执行。

use std::collections::HashMap;
use std::sync::Arc;

use nanobot_channels::messages::InboundMessage;
use nanobot_provider::{Message, Provider};
use nanobot_tools::{ToolContext, ToolRegistry};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

use crate::error::SubagentResult;
use crate::task::Task;

/// 会话任务注册表：session_key → [(task_id, JoinHandle)]
type SessionTasks = Arc<Mutex<HashMap<String, Vec<(String, JoinHandle<()>)>>>>;

/// 子代理管理器
///
/// 负责创建后台运行的子代理任务，管理运行中的任务集合，并通过消息总线通知任务完成结果。
#[allow(dead_code)]
pub struct SubagentManager<P>
where
    P: Provider + Clone + Send + Sync + 'static,
{
    /// LLM 提供商
    provider: P,
    /// 工作区路径
    workspace: std::path::PathBuf,
    /// 消息总线发送端
    bus: mpsc::Sender<InboundMessage>,
    /// 工具注册表
    tool_registry: ToolRegistry,
    /// 温度参数
    temperature: f32,
    /// 最大令牌数
    max_tokens: u32,
    /// 工具执行超时时间（秒）
    tool_timeout_secs: u64,
    /// 按会话追踪运行中的子代理任务
    session_tasks: SessionTasks,
}

impl<P> SubagentManager<P>
where
    P: Provider + Clone + Send + Sync + 'static,
{
    /// 创建新的子代理管理器
    ///
    /// # Arguments
    /// * `provider` - LLM 提供商
    /// * `workspace` - 工作区路径
    /// * `bus` - 消息总线发送端
    /// * `temperature` - 温度参数（默认 0.7）
    /// * `max_tokens` - 最大令牌数（默认 4096）
    pub fn new(
        provider: P,
        workspace: std::path::PathBuf,
        bus: mpsc::Sender<InboundMessage>,
        temperature: f32,
        max_tokens: u32,
    ) -> Arc<Self> {
        let tool_registry = ToolRegistry::new(workspace.clone(), nanobot_config::ExecToolConfig::default(), false);

        // 绑定工具到 provider
        let tool_definitions = tool_registry.get_definitions();
        let mut provider = provider;
        if !tool_definitions.is_empty() {
            provider.bind_tools(tool_definitions);
        }

        Arc::new(Self {
            provider,
            workspace,
            bus,
            tool_registry,
            temperature,
            max_tokens,
            tool_timeout_secs: 30,
            session_tasks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 创建并启动一个新的子代理任务
    ///
    /// # Arguments
    /// * `task` - 任务描述
    /// * `session_key` - 会话标识，用于按会话追踪和取消任务
    /// * `label` - 任务标签（可选，默认使用描述的前 30 个字符）
    /// * `origin_channel` - 来源通道
    /// * `origin_chat_id` - 聊天标识
    ///
    /// # Returns
    /// 返回状态消息，指示子代理已启动
    pub async fn spawn(
        self: Arc<Self>,
        task: impl Into<String>,
        session_key: impl Into<String>,
        label: Option<String>,
        origin_channel: impl Into<String>,
        origin_chat_id: impl Into<String>,
    ) -> SubagentResult<String> {
        let task = task.into();
        let session_key = session_key.into();
        let label = label.unwrap_or_else(|| Task::label_from_description(&task));
        let origin_channel = origin_channel.into();
        let origin_chat_id = origin_chat_id.into();

        let task_obj = Task::new(&task, &label, &origin_channel, &origin_chat_id);
        let task_id = task_obj.id.clone();

        tracing::info!(
            task_id = %task_obj.id,
            label = %task_obj.label,
            session_key = %session_key,
            "Spawning subagent"
        );

        // 克隆用于 spawned task 内部清理的引用
        let session_tasks = Arc::clone(&self.session_tasks);
        let task_id_for_cleanup = task_id.clone();
        let session_key_for_cleanup = session_key.clone();

        // 克隆 Arc<Self> 以传递到异步任务
        let manager = self.clone();

        // 异步执行任务
        let handle = tokio::spawn(async move {
            let result = manager.run_subagent(&task_obj).await;

            // 通知完成
            manager.announce_result(&task_obj, result).await;

            // 从 session_tasks 中移除自身条目
            let mut tasks = session_tasks.lock().await;
            if let Some(entries) = tasks.get_mut(&session_key_for_cleanup) {
                entries.retain(|(id, _)| id != &task_id_for_cleanup);
                if entries.is_empty() {
                    tasks.remove(&session_key_for_cleanup);
                }
            }
        });

        // 将 JoinHandle 存入 session_tasks
        self.session_tasks.lock().await.entry(session_key).or_default().push((task_id.clone(), handle));

        Ok(format!("Subagent [{label}] started (id: {task_id}). I'll notify you when it completes."))
    }

    /// 取消指定会话的所有子代理任务
    ///
    /// # Arguments
    /// * `session_key` - 会话标识
    ///
    /// # Returns
    /// 取消的任务数量
    pub async fn cancel_by_session(&self, session_key: &str) -> usize {
        let entries = self.session_tasks.lock().await.remove(session_key).unwrap_or_default();
        let count = entries.len();
        for (task_id, handle) in entries {
            tracing::info!(task_id = %task_id, session_key = %session_key, "Cancelling subagent");
            handle.abort();
        }
        count
    }

    /// 获取当前运行中的子代理数量
    pub async fn get_running_count(&self) -> usize {
        self.session_tasks.lock().await.values().map(|v| v.len()).sum()
    }

    /// 执行子代理任务（与 Python 版本的 _run_subagent 对应）
    ///
    /// 此方法构建工具集、系统提示，并执行 LLM 循环直到任务完成或达到最大迭代次数。
    async fn run_subagent(&self, task: &Task) -> Result<String, String> {
        /// 最大执行迭代次数
        const MAX_ITERATIONS: usize = 15;

        tracing::info!(
            task_id = %task.id,
            description = %task.description,
            "Starting task execution"
        );

        // 构建系统提示
        let system_prompt = self.build_subagent_prompt();

        // 初始化消息上下文
        let mut messages = vec![Message::system(system_prompt)];
        messages.push(Message::user(task.description.clone()));

        // 执行循环
        let mut iteration = 0;
        let mut final_result: Option<String> = None;

        while iteration < MAX_ITERATIONS {
            iteration += 1;

            tracing::debug!(
                task_id = %task.id,
                iteration = iteration,
                "Executing task iteration"
            );

            // 调用LLM
            let options = nanobot_provider::Options {
                max_tokens: self.max_tokens as u16,
                temperature: self.temperature,
                reasoning_effort: None,
            };
            let response = match self.provider.chat(&messages, &options).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(
                        task_id = %task.id,
                        error = %e,
                        "LLM call failed"
                    );
                    return Err(format!("LLM call failed: {e}"));
                }
            };

            // 在消费 response 之前提取工具调用数据
            let tool_calls = response.tool_calls().to_vec();
            if !tool_calls.is_empty() {
                // 将原始 LLM 响应直接添加到上下文（保留 thinking 等 provider 特定字段）
                messages.push(response);

                // 执行每个工具调用
                for tool_call in &tool_calls {
                    tracing::debug!(
                        task_id = %task.id,
                        tool = %tool_call.name,
                        "Executing tool call"
                    );

                    // 创建工具上下文
                    let tool_context = ToolContext::new(&task.channel, &task.chat_id);

                    // 解析参数
                    let args = tool_call.parse_arguments::<serde_json::Value>().unwrap_or(serde_json::Value::Null);

                    // 使用 tool_registry 执行工具
                    let result = match self.tool_registry.execute(&tool_context, &tool_call.name, args).await {
                        Ok(r) => r,
                        Err(e) => format!("Tool execution failed: {e}"),
                    };

                    // 添加工具结果到上下文
                    messages.push(Message::tool(tool_call.id.clone(), result));
                }
            } else {
                // 没有工具调用，保存最终结果
                final_result = Some(response.content().to_string());
                break;
            }
        }

        match final_result {
            Some(result) => {
                tracing::info!(
                    task_id = %task.id,
                    "Task completed successfully"
                );
                Ok(result)
            }
            None => {
                tracing::warn!(
                    task_id = %task.id,
                    "Task completed without final response"
                );
                Ok("Task completed but no final response was generated.".to_string())
            }
        }
    }

    /// 构建子代理系统提示
    fn build_subagent_prompt(&self) -> String {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M (%A)").to_string();
        let tz = chrono::Local::now().format("%Z").to_string();
        let workspace = self.workspace.display();

        let mut parts = vec![format!(
            r#"# Subagent

## Current Time

{now} ({tz})

You are a subagent spawned by the main agent to complete a specific task.
Stay focused on the assigned task. Your final response will be reported back to the main agent.

## Workspace

{workspace}"#
        )];

        // 动态发现工作空间 skills
        match nanobot_skills::SkillsLoader::new(self.workspace.clone()).build_skills_summary() {
            Ok(summary) if !summary.is_empty() => {
                parts.push(format!("## Skills\n\nRead SKILL.md with read_file to use a skill.\n\n{summary}"));
            }
            Err(e) => {
                tracing::warn!("Failed to build skills summary for subagent: {e}");
            }
            _ => {}
        }

        parts.join("\n\n")
    }

    /// 通知任务完成（与 Python 版本的 _announce_result 对应）
    async fn announce_result(&self, task: &Task, result: Result<String, String>) {
        let (status_text, content) = match &result {
            Ok(result) => {
                let content = format!(
                    "[Subagent '{}' completed successfully]\n\nTask: {}\n\nResult:\n{}\n\nSummarize this naturally for the user. Keep it brief (1-2 sentences). Do not mention technical details like \"subagent\" or task IDs.",
                    task.label, task.description, result
                );
                ("completed successfully", content)
            }
            Err(error) => {
                let content = format!(
                    "[Subagent '{}' failed]\n\nTask: {}\n\nError:\n{}\n\nSummarize this naturally for the user. Keep it brief (1-2 sentences). Do not mention technical details like \"subagent\" or task IDs.",
                    task.label, task.description, error
                );
                ("failed", content)
            }
        };

        // 构建chat_id，格式为 channel:chat_id
        let chat_id = format!("{}:{}", task.channel, task.chat_id);

        let message = InboundMessage::new(
            "system",   // channel
            "subagent", // sender_id
            &chat_id, content,
        );

        tracing::debug!(
            task_id = %task.id,
            status = %status_text,
            "Announcing result"
        );

        let _ = self.bus.send(message).await;
    }
}
