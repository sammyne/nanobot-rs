//! AgentLoop 核心实现
//!
//! AgentLoop 是 nanobot 的核心处理引擎，负责：
//! 1. 接收消息（通过入站消息接收端）
//! 2. 维护会话历史
//! 3. 调用 LLM
//! 4. 返回响应（通过出站消息发送端）

use std::sync::Arc;

use anyhow::Result;
use nanobot_config::AgentDefaults;
use nanobot_context::ContextBuilder;
use nanobot_cron::{CronService, CronTool};
use nanobot_provider::{Message, Provider};
use nanobot_session::SessionManager;
use nanobot_tools::{ToolContext, ToolRegistry};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{InboundMessage, OutboundMessage};

/// ReAct 运行结果
#[derive(Debug, Clone)]
pub struct ReActResult {
    /// 最终响应内容
    pub content: String,
    /// 使用的工具名称列表
    pub tools_used: Vec<String>,
    /// 完成时的完整消息历史
    pub messages: Vec<Message>,
}

/// Agent 循环处理引擎
///
/// 负责管理消息处理和 LLM 调用的完整生命周期。
pub struct AgentLoop<P: Provider> {
    /// LLM 提供者实例
    provider: P,

    /// Agent 配置
    config: AgentDefaults,

    /// 会话管理器
    sessions: Arc<SessionManager>,

    /// 工具注册表
    tool_registry: ToolRegistry,

    /// 上下文构建器（包含 MemoryStore）
    context: ContextBuilder,
}

impl<P: Provider + 'static> AgentLoop<P> {
    /// 创建新的 AgentLoop 实例
    ///
    /// tool_registry 会根据 config 中的 workspace 参数自动构造。
    ///
    /// # Arguments
    /// * `provider` - LLM 提供者实例
    /// * `config` - Agent 配置
    /// * `cron_service` - 可选的 Cron 服务实例
    pub fn new(mut provider: P, config: AgentDefaults, _cron_service: Option<Arc<CronService>>) -> Self {
        info!(
            "初始化 AgentLoop: model={}, max_tool_iterations={}",
            config.model, config.max_tool_iterations
        );

        // 基于 config 构造 tool_registry
        let workspace_str = config.workspace.to_string_lossy();
        let mut tool_registry = ToolRegistry::new(&workspace_str, None);

        // 如果提供了 cron_service，注册 CronTool
        if let Some(ref service) = _cron_service {
            let cron_tool = CronTool::new(Arc::clone(service));
            info!("注册 CronTool");
            tool_registry.register(cron_tool);
        }

        // 从 tool_registry 导出工具列表并绑定到 provider
        let definitions = tool_registry.get_definitions();
        provider.bind_tools(definitions);

        // Initialize SessionManager
        let sessions = Arc::new(SessionManager::new(config.workspace.clone()));

        // Initialize ContextBuilder (which contains MemoryStore)
        let context = ContextBuilder::new(config.workspace.clone()).expect("Failed to initialize ContextBuilder");

        Self {
            provider,
            config,
            sessions,
            tool_registry,
            context,
        }
    }

    /// 创建新的 AgentLoop 实例（单次消息模式）
    ///
    /// 直接复用 new 函数的逻辑，无需通道。
    ///
    /// # Arguments
    /// * `provider` - LLM 提供者实例
    /// * `config` - Agent 配置
    /// * `cron_service` - 可选的 Cron 服务实例
    pub fn new_direct(provider: P, config: AgentDefaults, _cron_service: Option<Arc<CronService>>) -> Self {
        Self::new(provider, config, _cron_service)
    }

    /// 获取或创建会话（与 Python 版本一致，返回 Session 对象）
    fn get_or_create_session(&self, session_key: &str) -> nanobot_session::Session {
        self.sessions.get_or_create(session_key)
    }

    /// 工具结果最大字符数（与 Python 版本一致）
    const TOOL_RESULT_MAX_CHARS: usize = 500;

    /// 保存本回合的消息到 session（增量追加，与 Python 版本的 _save_turn 一致）
    ///
    /// # Arguments
    /// * `session` - 会话对象
    /// * `messages` - 所有消息列表
    /// * `skip` - 跳过的消息数量（已存在于历史中的消息）
    fn save_turn(&self, session: &mut nanobot_session::Session, messages: &[Message], skip: usize) {
        // 只追加新消息
        for msg in messages.iter().skip(skip) {
            let msg_to_save = match msg {
                Message::Tool { content, tool_call_id } => {
                    // 截断过长的工具结果
                    let truncated = if content.len() > Self::TOOL_RESULT_MAX_CHARS {
                        format!("{}\n... (truncated)", &content[..Self::TOOL_RESULT_MAX_CHARS])
                    } else {
                        content.clone()
                    };
                    Message::Tool {
                        content: truncated,
                        tool_call_id: tool_call_id.clone(),
                    }
                }
                other => other.clone(),
            };
            session.add_message(msg_to_save);
        }
        session.touch();
    }

    /// 调用 LLM 并返回响应消息
    async fn call_llm(&self, messages: &[Message]) -> Result<Message> {
        debug!("调用 LLM: 消息数量={}", messages.len());

        let response = self.provider.chat(messages).await?;

        info!(
            "收到 LLM 响应, 角色={}, 内容长度={} 字符",
            response.role(),
            response.content().len()
        );

        Ok(response)
    }

    /// ReAct 推理-行动循环
    ///
    /// 参考 Python 版 `_run_agent_loop` 实现：
    /// 1. 循环调用 LLM，直到没有工具调用或达到最大迭代次数
    /// 2. 如果响应包含工具调用，执行工具并将结果添加回消息历史
    /// 3. 返回最终的内容、使用的工具列表和完整消息历史
    ///
    /// # Arguments
    /// * `initial_messages` - 初始消息列表
    /// * `channel` - 通道名称
    /// * `chat_id` - 聊天标识
    ///
    /// # Returns
    /// ReActResult 包含最终结果、工具使用列表和消息历史
    pub async fn re_act(&self, mut messages: Vec<Message>, channel: &str, chat_id: &str) -> Result<ReActResult> {
        let max_iterations = self.config.max_tool_iterations;
        let mut iteration = 0;
        let mut tools_used: Vec<String> = Vec::new();

        info!(
            "启动 ReAct 循环: max_iterations={}, 可用工具={:?}",
            max_iterations,
            self.tool_registry.tool_names()
        );

        while iteration < max_iterations {
            iteration += 1;
            debug!("ReAct 迭代 #{}", iteration);

            // 调用 LLM
            let response = self.call_llm(&messages).await?;

            // 检查是否有工具调用
            let tool_calls = response.tool_calls();
            if !tool_calls.is_empty() {
                // 提取文本内容
                let content = response.content().to_string();

                // 记录工具调用
                let tool_hints: Vec<String> = tool_calls
                    .iter()
                    .map(|tc| {
                        let first_arg = tc.arguments.chars().take(40).collect::<String>();
                        if tc.arguments.len() > 40 {
                            format!("{}({}...)", tc.name, first_arg)
                        } else {
                            format!("{}({})", tc.name, tc.arguments)
                        }
                    })
                    .collect();
                debug!("工具调用: {}", tool_hints.join(", "));

                // 将助手消息（带工具调用）添加到历史
                messages.push(Message::assistant_with_tools(&content, tool_calls.to_vec()));

                // 执行每个工具调用
                for tool_call in tool_calls {
                    tools_used.push(tool_call.name.clone());
                    info!("执行工具 {}: {}", tool_call.name, tool_call.arguments);

                    // 解析参数
                    let args = match serde_json::from_str::<serde_json::Value>(&tool_call.arguments) {
                        Ok(v) => v,
                        Err(e) => {
                            error!(
                                "解析工具 {} 参数失败: {}, 参数内容: {}",
                                tool_call.name, e, tool_call.arguments
                            );
                            serde_json::Value::String(tool_call.arguments.clone())
                        }
                    };

                    // 执行工具
                    let ctx = ToolContext::new(channel, chat_id);
                    let tool_result = self.tool_registry.execute(&ctx, &tool_call.name, args).await;

                    // 转换结果为字符串
                    let result_content = match tool_result {
                        Ok(output) => format!("Tool Call Result:\n{output}"),
                        Err(e) => {
                            error!("工具 {} 执行失败: {}", tool_call.name, e);
                            format!("Tool Call Error: {e}")
                        }
                    };

                    // 添加工具结果消息
                    messages.push(Message::tool(&tool_call.id, result_content));
                }
            } else {
                // 没有工具调用，返回最终结果
                let final_content = response.content().to_string();
                // 将助手消息添加到历史
                messages.push(Message::assistant(&final_content));

                info!(
                    "ReAct 循环完成: 迭代次数={}, 最终内容长度={} 字符",
                    iteration,
                    final_content.len()
                );

                return Ok(ReActResult {
                    content: final_content,
                    tools_used,
                    messages,
                });
            }
        }

        // 达到最大迭代次数
        warn!("ReAct 循环达到最大迭代次数: {}", max_iterations);
        let warning_msg = format!(
            "I reached the maximum number of tool call iterations ({max_iterations}) without completing the task. You can try breaking the task into smaller steps."
        );

        messages.push(Message::assistant(&warning_msg));

        Ok(ReActResult {
            content: warning_msg,
            tools_used,
            messages,
        })
    }

    /// 直接处理消息（单次调用模式）
    ///
    /// 参考 Python 版 `process_direct` 函数实现。
    /// 通过构造 InboundMessage 复用 process_message 方法。
    ///
    /// # Arguments
    /// * `content` - 消息内容
    /// * `session_key` - 会话标识，格式为 "channel:chat_id"，默认为 "cli:direct"
    /// * `channel` - 可选的通道名称，默认为 "cli"
    /// * `chat_id` - 可选的聊天标识，默认为 "direct"
    pub async fn process_direct(
        &self,
        content: &str,
        session_key: &str,
        channel: Option<&str>,
        chat_id: Option<&str>,
    ) -> Result<String> {
        info!("直接处理消息: {}", content);

        // 使用独立参数或默认值
        let channel = channel.unwrap_or("cli").to_string();
        let chat_id = chat_id.unwrap_or("direct").to_string();

        // 构造入站消息并复用 process_message
        let inbound = InboundMessage::new(&channel, "user", &chat_id, content);
        let outbound = self.process_message(inbound, Some(session_key)).await;

        Ok(outbound.content)
    }

    /// 尝试执行记忆整合
    ///
    /// 检查是否需要整合，如果需要则调用 LLM 进行记忆压缩。
    /// 整合失败不影响正常消息处理流程。
    async fn try_consolidate(&self, session: &mut nanobot_session::Session) -> Result<()> {
        match self
            .context
            .memory()
            .try_consolidate(
                &session.messages,
                session.last_consolidated,
                self.provider.clone(),
                false, // archive_all
                self.config.memory_window,
            )
            .await
        {
            Ok(new_last_consolidated) => {
                if new_last_consolidated != session.last_consolidated {
                    session.last_consolidated = new_last_consolidated;
                    // 持久化更新后的会话
                    if let Err(e) = self.sessions.save(session) {
                        error!("Failed to save session after consolidation: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Memory consolidation error: {}", e);
            }
        }

        Ok(())
    }
    /// 启动后台消息处理循环
    ///
    /// 这是交互式模式的核心方法。从入站通道接收消息，
    /// 处理后发送给出站通道。
    ///
    /// 循环在以下情况下会退出：
    /// - 入站通道关闭
    /// - 发生错误
    ///
    /// # Arguments
    /// * `inbound_rx` - 入站消息接收端（CLI -> AgentLoop）
    /// * `outbound_tx` - 出站消息发送端（AgentLoop -> CLI）
    pub async fn run(
        &self,
        mut inbound_rx: mpsc::Receiver<InboundMessage>,
        outbound_tx: mpsc::Sender<OutboundMessage>,
    ) -> Result<()> {
        info!("AgentLoop 后台循环已启动");

        loop {
            // 消费入站消息
            match inbound_rx.recv().await {
                Some(msg) => {
                    debug!("收到入站消息: channel={}, chat_id={}", msg.channel, msg.chat_id);

                    // 处理消息并发送响应（使用 inbound 的 session_key）
                    let outbound = self.process_message(msg, None).await;
                    if let Err(e) = outbound_tx.send(outbound).await {
                        error!("发送出站消息失败: {}", e);
                    }
                }
                None => {
                    // 通道关闭，退出循环
                    info!("入站通道已关闭，退出后台循环");
                    break;
                }
            }
        }

        info!("AgentLoop 后台循环已停止");
        Ok(())
    }

    /// 处理入站消息并返回待发送的响应
    ///
    /// # Arguments
    /// * `inbound` - 入站消息
    /// * `session_key` - 可选的会话标识，格式为 "channel:chat_id"；不存在时从 inbound.session_key() 获取
    ///
    /// 注意：此方法总是返回 OutboundMessage，错误会被转换为错误消息内容
    async fn process_message(&self, inbound: InboundMessage, session_key: Option<&str>) -> OutboundMessage {
        // 获取或创建会话：优先使用传入的 session_key，否则从 inbound 获取
        let session_key = session_key
            .map(|s| s.to_string())
            .unwrap_or_else(|| inbound.session_key());
        let mut session = self.get_or_create_session(&session_key);

        let InboundMessage {
            channel,
            sender_id: _,
            chat_id,
            content,
            ..
        } = inbound;

        // 获取历史消息
        let mut history = Vec::new();
        session.get_history(self.config.memory_window, &mut history);

        // 使用 ContextBuilder 构建消息列表
        let messages = self
            .context
            .build_messages(&history, &content, None, Some(&channel), Some(&chat_id));

        match messages {
            Ok(messages) => {
                let skip = messages.len() - 1; // 跳过系统消息 + 历史消息（不包括新消息）

                // 执行 ReAct 循环（支持工具调用）
                match self.re_act(messages, &channel, &chat_id).await {
                    Ok(result) => {
                        // 保存本回合消息（增量追加，跳过已存在的消息）
                        self.save_turn(&mut session, &result.messages, skip);
                        // 持久化会话
                        if let Err(e) = self.sessions.save(&session) {
                            error!("Failed to save session: {}", e);
                        }

                        // 记忆整合（在消息处理完成后）
                        if let Err(e) = self.try_consolidate(&mut session).await {
                            error!("Memory consolidation failed: {}", e);
                        }

                        OutboundMessage::new(&channel, &chat_id, &result.content)
                    }
                    Err(e) => {
                        error!("处理消息失败: {}", e);
                        let error_msg = format!("处理失败: {e}");
                        OutboundMessage::new(&channel, &chat_id, &error_msg)
                    }
                }
            }
            Err(e) => {
                error!("构建消息失败: {}", e);
                let error_msg = format!("构建消息失败: {e}");
                OutboundMessage::new(&channel, &chat_id, &error_msg)
            }
        }
    }

    /// 获取配置
    pub fn config(&self) -> &AgentDefaults {
        &self.config
    }
}
