//! 子代理管理器
//!
//! SubagentManager 负责创建、管理和监控子代理任务的执行。

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use nanobot_channels::messages::InboundMessage;
use nanobot_provider::{Message, Provider};
use nanobot_tools::{ToolContext, ToolRegistry};
use tokio::sync::mpsc;

use crate::error::SubagentResult;
use crate::task::Task;

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
    /// 运行中的任务数量
    running_tasks: AtomicUsize,
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
        let tool_registry =
            ToolRegistry::new(workspace.clone(), None::<String>, nanobot_config::ToolsConfig::default());

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
            running_tasks: AtomicUsize::new(0),
        })
    }

    /// 创建并启动一个新的子代理任务
    ///
    /// # Arguments
    /// * `task` - 任务描述
    /// * `label` - 任务标签（可选，默认使用描述的前 30 个字符）
    /// * `origin_channel` - 来源通道
    /// * `origin_chat_id` - 聊天标识
    ///
    /// # Returns
    /// 返回状态消息，指示子代理已启动
    pub async fn spawn(
        self: Arc<Self>,
        task: impl Into<String>,
        label: Option<String>,
        origin_channel: impl Into<String>,
        origin_chat_id: impl Into<String>,
    ) -> SubagentResult<String> {
        let task = task.into();
        let label = label.unwrap_or_else(|| Task::label_from_description(&task));
        let origin_channel = origin_channel.into();
        let origin_chat_id = origin_chat_id.into();

        let task_obj = Task::new(&task, &label, &origin_channel, &origin_chat_id);
        let task_id = task_obj.id.clone();

        tracing::info!(
            task_id = %task_obj.id,
            label = %task_obj.label,
            "Spawning subagent"
        );

        // 增加运行任务计数
        self.running_tasks.fetch_add(1, Ordering::SeqCst);

        // 克隆 Arc<Self> 以传递到异步任务
        let manager = self.clone();

        // 异步执行任务
        tokio::spawn(async move {
            let result = manager.run_subagent(&task_obj).await;

            // 减少运行任务计数
            manager.running_tasks.fetch_sub(1, Ordering::SeqCst);

            // 通知完成
            manager.announce_result(&task_obj, result).await;
        });

        Ok(format!("Subagent [{label}] started (id: {task_id}). I'll notify you when it completes."))
    }

    /// 获取当前运行中的子代理数量
    pub fn get_running_count(&self) -> usize {
        self.running_tasks.load(Ordering::SeqCst)
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
        let system_prompt = self.build_subagent_prompt(&task.description);

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
            let options = nanobot_provider::Options::default();
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

            // 检查是否有工具调用
            let tool_calls = response.tool_calls();
            if !tool_calls.is_empty() {
                // 添加助手消息（包含工具调用）到上下文
                messages.push(Message::assistant_with_tools(response.content().to_string(), tool_calls.to_vec()));

                // 执行每个工具调用
                for tool_call in tool_calls {
                    tracing::debug!(
                        task_id = %task.id,
                        tool = %tool_call.name,
                        "Executing tool call"
                    );

                    // 创建工具上下文
                    let tool_context = ToolContext::new(&task.channel, &task.chat_id);

                    // 解析参数
                    let args = tool_call.parse_arguments().unwrap_or(serde_json::Value::Null);

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
    fn build_subagent_prompt(&self, task_description: &str) -> String {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M (%A)").to_string();
        let tz = chrono::Local::now().format("%Z").to_string();
        let workspace = self.workspace.display();

        format!(
            r#"## Role

You are a subagent spawned by the main agent to complete a specific task: {task_description}

## Current Time

{now} ({tz})

## Rules

- Stay focused - complete only the assigned task, nothing else
- Your final response will be reported back to the main agent
- Do not initiate conversations or take on side tasks
- Be concise but informative in your findings

## What You Can Do

Read and write files in the workspace, Execute shell commands, Complete the task thoroughly

## What You Cannot Do

Send messages directly to users (no message tool available), Spawn other subagents, Access the main agent's conversation history

## Workspace

Your workspace is at: {workspace}

When you have completed the task, provide a clear summary of your findings or actions."#
        )
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
