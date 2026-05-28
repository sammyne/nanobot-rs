//! AgentLoop 核心实现
//!
//! AgentLoop 是 nanobot 的核心处理引擎，负责：
//! 1. 接收消息（通过入站消息接收端）
//! 2. 维护会话历史
//! 3. 调用 LLM
//! 4. 返回响应（通过出站消息发送端）

use std::collections::HashSet;
use std::sync::{Arc, LazyLock};
use std::time::Instant;

use anyhow::Result;
use nanobot_config::AgentDefaults;
use nanobot_context::ContextBuilder;
use nanobot_cron::{CronService, CronTool};
use nanobot_mcp::connect;
use nanobot_provider::{Message, MeteredMessage, Provider, TokenUsage};
use nanobot_session::SessionManager;
use nanobot_subagent::{SpawnTool, SubagentManager};
use nanobot_tools::{Tool, ToolContext, ToolRegistry};
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, error, info, warn};

use crate::cmd::{Command, HelpCmd, NewCmd, RestartCmd, StatusCmd, StopCmd};
use crate::utils::parse_system_message_target;
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

    /// 正在进行记忆整合的会话集合
    consolidating: Arc<Mutex<HashSet<String>>>,

    /// 子代理管理器（用于 /stop 命令取消子代理）
    subagent_manager: Arc<SubagentManager<P>>,

    /// 出站消息发送端（用于 run() 和 MessageTool）
    outbound_tx: mpsc::Sender<OutboundMessage>,

    /// 启动时间
    start_time: Instant,

    /// 最近一次 LLM 调用的 token 用量
    last_usage: std::sync::Mutex<Option<TokenUsage>>,
}

impl<P: Provider> AgentLoop<P> {
    /// 创建新的 AgentLoop 实例
    ///
    /// tool_registry 会根据 config 中的 workspace 参数自动构造。
    /// 如果 tools_config 中包含 MCP 服务器配置，会自动连接并注册工具。
    ///
    /// # Arguments
    /// * `provider` - LLM 提供者实例
    /// * `config` - Agent 配置
    /// * `cron_service` - 可选的 Cron 服务实例
    /// * `subagent_manager` - 可选的子代理管理器
    /// * `tools_config` - 工具配置（包含 MCP 服务器配置、ExecTool 配置等）
    /// * `outbound_tx` - 出站消息发送端（用于 run() 发送响应和 MessageTool 主动发送）
    pub async fn new(
        mut provider: P,
        config: AgentDefaults,
        cron_service: Option<Arc<CronService>>,
        subagent_manager: Arc<SubagentManager<P>>,
        tools_config: nanobot_config::ToolsConfig,
        outbound_tx: mpsc::Sender<OutboundMessage>,
    ) -> Result<Self> {
        info!("初始化 AgentLoop: model={}, max_tool_iterations={}", config.model, config.max_tool_iterations);

        // 基于 config 构造 tool_registry，传入 exec_config 和 restrict_to_workspace
        let mut tool_registry =
            ToolRegistry::new(config.workspace.clone(), tools_config.exec.clone(), tools_config.restrict_to_workspace);

        // 连接 MCP 服务器并注册工具
        let mcp_server_count = tools_config.mcp_servers.len();
        let mut mcp_tool_count = 0;
        if !tools_config.mcp_servers.is_empty() {
            info!("发现 {} 个 MCP 服务器配置", mcp_server_count);
            match connect(tools_config.mcp_servers).await {
                Ok(mcp_tools) => {
                    mcp_tool_count = mcp_tools.len();
                    for tool in mcp_tools {
                        let tool_name = tool.name().to_string();
                        tool_registry.register(tool);
                        info!("注册 MCP 工具: {}", tool_name);
                    }
                }
                Err(e) => {
                    error!("MCP 连接失败: {}，AgentLoop 将在无 MCP 工具的情况下继续初始化", e);
                }
            }
        }

        // 如果提供了 cron_service，注册 CronTool
        if let Some(ref service) = cron_service {
            let cron_tool = CronTool::new(Arc::clone(service));
            info!("注册 CronTool");
            tool_registry.register(cron_tool);
        }

        // 注册 SpawnTool
        let spawn_tool = SpawnTool::new(Arc::clone(&subagent_manager));
        info!("注册 SpawnTool");
        tool_registry.register(spawn_tool);

        // 注册 MessageTool
        let message_tool = crate::tools::message::MessageTool::new(
            outbound_tx.clone(),
            config.workspace.clone(),
            tools_config.restrict_to_workspace,
        );
        info!("注册 MessageTool");
        tool_registry.register(message_tool);

        // 从 tool_registry 导出工具列表并绑定到 provider
        let definitions = tool_registry.get_definitions();
        let tool_names = tool_registry.tool_names();
        provider.bind_tools(definitions);

        // 记录初始化统计信息
        info!("AgentLoop 初始化完成: MCP 服务器={}, 总工具={}", mcp_server_count, tool_names.len());
        if mcp_server_count > 0 {
            info!("已注册 {} 个 MCP 工具", mcp_tool_count);
        }

        // Initialize SessionManager
        let sessions = Arc::new(SessionManager::new(config.workspace.clone()));

        // Initialize ContextBuilder (which contains MemoryStore)
        let consolidation_options = nanobot_provider::Options {
            max_tokens: config.max_tokens as u16,
            temperature: config.temperature as f32,
            reasoning_effort: config.reasoning_effort,
            tool_choice: None,
        };
        let context = ContextBuilder::new(config.workspace.clone(), consolidation_options)
            .expect("Failed to initialize ContextBuilder");

        Ok(Self {
            provider,
            config,
            sessions,
            tool_registry,
            context,
            consolidating: Arc::new(Mutex::new(HashSet::new())),
            subagent_manager,
            outbound_tx,
            start_time: Instant::now(),
            last_usage: std::sync::Mutex::new(None),
        })
    }

    /// 调用 LLM 并返回响应消息
    async fn call_llm(&self, messages: &[Message]) -> Result<MeteredMessage> {
        debug!("调用 LLM: 消息数量={}", messages.len());

        let options = nanobot_provider::Options {
            max_tokens: self.config.max_tokens as u16,
            temperature: self.config.temperature as f32,
            reasoning_effort: self.config.reasoning_effort,
            tool_choice: None,
        };
        let response = self.provider.chat(messages, &options).await?;

        // 记录最近一次 token 用量
        *self.last_usage.lock().unwrap() = response.usage.clone();

        info!("收到 LLM 响应, 角色={}, 内容长度={} 字符", response.role(), response.content().len());

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
    /// * `hook` - 生命周期钩子
    ///
    /// # Returns
    /// ReActResult 包含最终结果、工具使用列表和消息历史
    pub async fn re_act(
        &self,
        mut messages: Vec<Message>,
        channel: &str,
        chat_id: &str,
        hook: &dyn crate::Hook,
        scheduled: bool,
    ) -> Result<ReActResult> {
        let max_iterations = self.config.max_tool_iterations;
        let mut iteration = 0;
        let mut tools_used: Vec<String> = Vec::new();
        let tool_ctx =
            if scheduled { ToolContext::scheduled(channel, chat_id) } else { ToolContext::new(channel, chat_id) };

        info!("启动 ReAct 循环: max_iterations={}, 可用工具={:?}", max_iterations, self.tool_registry.tool_names());

        while iteration < max_iterations {
            iteration += 1;
            debug!("ReAct 迭代 #{}", iteration);

            // hook: before_iteration
            let usage = self.last_usage.lock().unwrap().clone();
            let hook_ctx = crate::HookCtx { content: "", tool_calls: &[], usage: usage.as_ref() };
            if let Err(e) = hook.before_iteration(&hook_ctx).await {
                error!("hook before_iteration failed: {e}");
            }

            // 调用 LLM
            let response = self.call_llm(&messages).await?;

            // 在消费 response 之前提取后续需要的数据
            let content = response.content().to_string();
            let tool_calls = response.tool_calls().to_vec();
            let usage = self.last_usage.lock().unwrap().clone();
            let response = response.message;

            if !tool_calls.is_empty() {
                // hook: before_execute_tools
                let hook_ctx = crate::HookCtx { content: &content, tool_calls: &tool_calls, usage: usage.as_ref() };
                if let Err(e) = hook.before_execute_tools(&hook_ctx).await {
                    error!("hook before_execute_tools failed: {e}");
                }

                // 记录工具调用
                let tool_hints: Vec<String> = tool_calls.iter().map(|tc| tc.preview()).collect();
                debug!("工具调用: {}", tool_hints.join(", "));

                // 将原始 LLM 响应直接添加到历史（保留 thinking 等 provider 特定字段）
                messages.push(response);

                // 按只读/副作用分批执行工具
                let session_key = format!("{channel}:{chat_id}");
                let batches = partition_tool_batches(&tool_calls, &self.tool_registry);
                for batch in batches {
                    if batch.len() > 1 {
                        // 并行执行只读工具批次
                        let futs: Vec<_> =
                            batch.iter().map(|tc| self.execute_one_tool(tc, &tool_ctx, &session_key)).collect();
                        let results = futures::future::join_all(futs).await;
                        for (tc, (name, result_content)) in batch.iter().zip(results) {
                            tools_used.push(name);
                            messages.push(Message::tool(&tc.id, result_content));
                        }
                    } else {
                        // 串行执行单个工具
                        let tc = batch[0];
                        let (name, result_content) = self.execute_one_tool(tc, &tool_ctx, &session_key).await;
                        tools_used.push(name);
                        messages.push(Message::tool(&tc.id, result_content));
                    }
                }

                // hook: after_iteration
                let hook_ctx = crate::HookCtx { content: &content, tool_calls: &tool_calls, usage: usage.as_ref() };
                if let Err(e) = hook.after_iteration(&hook_ctx).await {
                    error!("hook after_iteration failed: {e}");
                }
            } else {
                // 没有工具调用，返回最终结果
                // 防御性处理：空内容替换为 "(empty)"，避免 session 历史中出现空 assistant 消息
                let (content, response) = if content.is_empty() {
                    warn!("LLM returned empty content without tool calls, replacing with (empty)");
                    let placeholder = "(empty)".to_string();
                    let fixed = match response {
                        Message::Assistant { thinking: Some(t), .. } => {
                            Message::assistant_with_thinking(&placeholder, Vec::new(), t)
                        }
                        _ => Message::assistant(&placeholder),
                    };
                    (placeholder, fixed)
                } else {
                    (content, response)
                };

                messages.push(response);

                // hook: finalize_content (content passed separately, ctx uses empty)
                let hook_ctx = crate::HookCtx { content: "", tool_calls: &[], usage: usage.as_ref() };
                let content = hook.finalize_content(&hook_ctx, Some(content)).await.unwrap_or_default();

                info!("ReAct 循环完成: 迭代次数={}, 最终内容长度={} 字符", iteration, content.len());

                return Ok(ReActResult { content, tools_used, messages });
            }
        }

        // 达到最大迭代次数
        warn!("ReAct 循环达到最大迭代次数: {}", max_iterations);
        let warning_msg = format!(
            "I reached the maximum number of tool call iterations ({max_iterations}) without completing the task. You can try breaking the task into smaller steps."
        );

        messages.push(Message::assistant(&warning_msg));

        // hook: finalize_content (content passed separately, ctx uses empty)
        let usage = self.last_usage.lock().unwrap().clone();
        let hook_ctx = crate::HookCtx { content: "", tool_calls: &[], usage: usage.as_ref() };
        let warning_msg = hook.finalize_content(&hook_ctx, Some(warning_msg)).await.unwrap_or_default();

        Ok(ReActResult { content: warning_msg, tools_used, messages })
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
    /// * `media` - 可选的媒体文件路径列表
    /// * `hook` - 可选的生命周期钩子
    pub async fn process_direct(
        &self,
        content: &str,
        session_key: &str,
        channel: Option<&str>,
        chat_id: Option<&str>,
        media: Option<&[std::path::PathBuf]>,
        hook: Option<Arc<dyn crate::Hook>>,
    ) -> Result<String> {
        info!("直接处理消息: {}", content);

        // 使用独立参数或默认值
        let channel = channel.unwrap_or("cli");
        let chat_id = chat_id.unwrap_or("direct");

        // 构造入站消息并复用 process_message
        let mut inbound = InboundMessage::new(channel, "user", chat_id, content);
        if let Some(paths) = media {
            for path in paths {
                inbound = inbound.add_media(path.display().to_string());
            }
        }
        let outbound = self.process_message(inbound, Some(session_key), hook).await;

        Ok(outbound.content)
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
    pub async fn run(self: Arc<Self>, mut inbound_rx: mpsc::Receiver<InboundMessage>) -> Result<()> {
        info!("AgentLoop 后台循环已启动");

        // 缓冲在处理期间到达的非 /stop 消息
        let mut pending: std::collections::VecDeque<InboundMessage> = std::collections::VecDeque::new();

        loop {
            // 优先从 pending 缓冲区取消息，否则从 inbound_rx recv
            let msg = if let Some(msg) = pending.pop_front() {
                msg
            } else {
                match inbound_rx.recv().await {
                    Some(msg) => msg,
                    None => {
                        info!("入站通道已关闭，退出后台循环");
                        break;
                    }
                }
            };

            debug!("收到入站消息: channel={}, chat_id={}", msg.channel, msg.chat_id);

            // /stop 在空闲时（无任务运行）直接处理
            if is_stop_cmd(&msg.content) {
                self.handle_stop(&msg).await;
                continue;
            }

            // 将 process_message 作为 tokio task 启动，以便 /stop 可以中断
            let self_clone = Arc::clone(&self);
            let channel = msg.channel.clone();
            let chat_id = msg.chat_id.clone();
            let hook: Arc<dyn crate::Hook> = Arc::new(crate::LoopHook::new(self.outbound_tx.clone(), channel, chat_id));

            let mut handle = tokio::spawn(async move { self_clone.process_message(msg, None, Some(hook)).await });

            // 等待任务完成，同时监听 /stop 命令
            loop {
                tokio::select! {
                    result = &mut handle => {
                        // 处理完成
                        match result {
                            Ok(outbound) => {
                                if let Err(e) = self.outbound_tx.send(outbound).await {
                                    error!("发送出站消息失败: {e}");
                                }
                            }
                            Err(e) if e.is_cancelled() => {
                                debug!("处理任务已被 /stop 取消");
                            }
                            Err(e) => {
                                error!("处理任务 panic: {e}");
                            }
                        }
                        break;
                    }
                    new_msg = inbound_rx.recv() => {
                        match new_msg {
                            Some(new_msg) if is_stop_cmd(&new_msg.content) => {
                                // /stop 到达：abort 主任务并取消子代理
                                handle.abort();
                                self.handle_stop(&new_msg).await;
                                let _ = handle.await; // 等待 abort 完成
                                break;
                            }
                            Some(new_msg) => {
                                // 非 /stop 消息：缓冲到下一轮处理
                                pending.push_back(new_msg);
                            }
                            None => {
                                // 入站通道关闭：abort 当前任务并退出
                                handle.abort();
                                let _ = handle.await;
                                info!("入站通道已关闭，退出后台循环");
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        info!("AgentLoop 后台循环已停止");
        Ok(())
    }

    /// 处理 /stop 命令：取消子代理并发送响应
    async fn handle_stop(&self, msg: &InboundMessage) {
        let session_key = msg.session_key();
        let cancelled = self.subagent_manager.cancel_by_session(&session_key).await;

        let response = if cancelled > 0 {
            format!("Stopped. Cancelled {cancelled} background task(s).")
        } else {
            "Stopped.".to_string()
        };

        info!("处理 /stop 命令: session_key={session_key}, cancelled={cancelled}");

        if let Err(e) = self.outbound_tx.send(OutboundMessage::new(&msg.channel, &msg.chat_id, &response)).await {
            error!("发送 /stop 响应失败: {e}");
        }
    }

    /// 处理系统消息
    /// 处理系统消息
    ///
    /// 系统消息是一种特殊的消息类型，其 `chat_id` 字段包含实际的目标通道和聊天 ID。
    /// 此方法解析目标路由信息，并返回带有正确路由信息的 OutboundMessage。
    ///
    /// # Arguments
    /// * `inbound` - 入站的系统消息
    ///
    /// # Returns
    /// 带有解析后目标路由信息的 OutboundMessage
    async fn process_system_message(&self, inbound: InboundMessage) -> OutboundMessage {
        info!("处理系统消息: sender_id={}", inbound.sender_id);

        // 解析目标路由信息
        let (target_channel, target_chat_id, session_key) = parse_system_message_target(&inbound.chat_id);

        // 获取或创建会话
        let mut session = self.sessions.get_or_create(&session_key);

        // 获取历史消息
        let mut history = Vec::new();
        session.get_history(self.config.memory_window, &mut history);

        // 使用 ContextBuilder 构建消息列表
        let messages =
            self.context.build_messages(&history, &inbound.content, None, Some(target_channel), Some(target_chat_id));

        match messages {
            Ok(messages) => {
                let skip = messages.len() - 1;

                // 执行 ReAct 循环
                let noop = crate::NoopHook;
                match self.re_act(messages, target_channel, target_chat_id, &noop, false).await {
                    Ok(result) => {
                        // 保存本回合消息
                        session.save_turn(&result.messages, skip);
                        // 持久化会话
                        if let Err(e) = self.sessions.save(&session) {
                            error!("Failed to save session: {}", e);
                        }

                        OutboundMessage::new(target_channel, target_chat_id, result.content)
                    }
                    Err(e) => {
                        error!("处理系统消息失败: {}", e);
                        let error_msg = format!("处理失败: {e}");
                        OutboundMessage::new(target_channel, target_chat_id, &error_msg)
                    }
                }
            }
            Err(e) => {
                error!("构建系统消息失败: {}", e);
                let error_msg = format!("构建消息失败: {e}");
                OutboundMessage::new(target_channel, target_chat_id, &error_msg)
            }
        }
    }

    /// 尝试处理命令
    ///
    /// 检查消息是否为已知命令（以 `/` 开头），如果是则处理并返回结果。
    ///
    /// # Arguments
    /// * `msg` - 入站消息
    /// * `session_key` - 会话标识
    ///
    /// # Returns
    /// - `Ok(OutboundMessage)`: 是命令（支持的或不支持的），并已正确处理
    /// - `Err(InboundMessage)`: 不是命令，返回入参的 InboundMessage 供后续处理
    async fn try_handle_cmd(&self, msg: InboundMessage, session_key: &str) -> Result<OutboundMessage, InboundMessage> {
        let cmd = match msg.content.strip_prefix('/') {
            None => return Err(msg),
            Some(cmd) => cmd.trim_end().to_lowercase(),
        };

        // 提取 channel 和 chat_id 供后续使用
        let channel = msg.channel.clone();
        let chat_id = msg.chat_id.clone();

        // 使用 match 结构构建对应的命令实例并执行
        let response_content = match cmd.as_str() {
            "help" => HelpCmd.run(msg, session_key.to_string()).await,
            "new" => {
                // NewCmd needs access to AgentLoop components
                // Create NewCmd with necessary dependencies
                let new_cmd = NewCmd::new(
                    self.sessions.clone(),
                    self.context.memory(),
                    self.provider.clone(),
                    self.consolidating.clone(),
                );
                new_cmd.run(msg, session_key.to_string()).await
            }
            "stop" => {
                let stop_cmd = StopCmd::new(self.subagent_manager.clone());
                stop_cmd.run(msg, session_key.to_string()).await
            }
            "restart" => RestartCmd.run(msg, session_key.to_string()).await,
            "status" => {
                let session = self.sessions.get_or_create(session_key);
                let status_cmd = StatusCmd {
                    model: self.config.model.clone(),
                    start_time: self.start_time,
                    last_usage: self.last_usage.lock().unwrap().clone(),
                    session_message_count: session.messages.len(),
                };
                status_cmd.run(msg, session_key.to_string()).await
            }
            // 不支持的命令返回提示信息
            _ => Err(format!("❌ Unsupported command: /{cmd}\nTry /help for available commands")),
        };

        // 处理命令执行结果
        let response_content = match response_content {
            Ok(content) => content,
            Err(error) => error,
        };

        Ok(OutboundMessage::new(&channel, &chat_id, response_content))
    }

    /// 处理入站消息并返回待发送的响应
    ///
    /// # Arguments
    /// * `inbound` - 入站消息
    /// * `session_key` - 可选的会话标识，格式为 "channel:chat_id"；不存在时从 inbound.session_key() 获取
    /// * `hook` - 可选的生命周期钩子
    ///
    /// 注意：此方法总是返回 OutboundMessage，错误会被转换为错误消息内容
    async fn process_message(
        &self,
        inbound: InboundMessage,
        session_key: Option<&str>,
        hook: Option<Arc<dyn crate::Hook>>,
    ) -> OutboundMessage {
        // 系统消息：从 chat_id 解析目标路由（格式为 "channel:chat_id"）
        if inbound.channel == "system" {
            return self.process_system_message(inbound).await;
        }

        let noop = crate::NoopHook;
        let hook: &dyn crate::Hook = match hook.as_deref() {
            Some(h) => h,
            None => &noop,
        };

        // 获取或创建会话：优先使用传入的 session_key，否则从 inbound 获取
        let session_key = session_key.map(|s| s.to_string()).unwrap_or_else(|| inbound.session_key());

        // 尝试处理命令
        let inbound = match self.try_handle_cmd(inbound, &session_key).await {
            Ok(outbound) => return outbound,
            Err(msg) => msg,
        };

        let mut session = self.sessions.get_or_create(&session_key);

        let InboundMessage { channel, sender_id: _, chat_id, content, media, .. } = inbound;

        // 获取历史消息
        let mut history = Vec::new();
        session.get_history(self.config.memory_window, &mut history);

        // 保存旧的 last_consolidated 用于判断是否发生变化
        let old_last_consolidated = session.last_consolidated;

        // 在 build_messages 之前启动异步记忆整合任务
        let consolidation_handle = tokio::spawn(try_consolidate(
            self.context.memory(),
            self.provider.clone(),
            session,
            self.config.memory_window,
            self.config.max_input_tokens,
            Arc::clone(&self.consolidating),
        ));

        // 使用 ContextBuilder 构建消息列表
        let media_paths: Vec<std::path::PathBuf> = media.iter().map(std::path::PathBuf::from).collect();
        let media_ref = if media_paths.is_empty() { None } else { Some(media_paths.as_slice()) };
        let messages = match self.context.build_messages(&history, &content, media_ref, Some(&channel), Some(&chat_id))
        {
            Ok(v) => v,
            Err(err) => {
                error!("构建消息失败: {err}");
                let error_msg = format!("构建消息失败: {err}");
                return OutboundMessage::new(&channel, &chat_id, &error_msg);
            }
        };

        let skip = messages.len() - 1; // 跳过系统消息 + 历史消息（不包括新消息）

        let scheduled = session_key.starts_with("cron:");

        // 执行 ReAct 循环（支持工具调用）
        let result = match self.re_act(messages, &channel, &chat_id, hook, scheduled).await {
            Ok(v) => v,
            Err(e) => {
                error!("处理消息失败: {}", e);
                let error_msg = format!("处理失败: {e}");
                return OutboundMessage::new(&channel, &chat_id, &error_msg);
            }
        };

        // 等待记忆整合任务完成，获取更新后的 Session
        debug!("等待记忆整合任务完成: session_key={}", session_key);
        match consolidation_handle.await {
            Ok(Ok(consolidated_session)) => {
                if consolidated_session.last_consolidated != old_last_consolidated {
                    info!("记忆整合完成: last_consolidated={}", consolidated_session.last_consolidated);
                }
                session = consolidated_session;
            }
            Ok(Err((consolidated_session, e))) => {
                error!("记忆整合失败: {}", e);
                session = consolidated_session;
            }
            Err(e) => {
                error!("记忆整合任务 join 失败: {}", e);
                return OutboundMessage::new(&channel, &chat_id, format!("记忆整合任务失败: {e}"));
            }
        }
        // ReAct 结果需要合并到整合后的 Session
        session.save_turn(&result.messages, skip);

        // 持久化会话（无论整合是否成功）
        if let Err(e) = self.sessions.save(&session) {
            error!("Failed to save session: {}", e);
        }

        OutboundMessage::new(&channel, &chat_id, &result.content)
    }

    /// 获取配置
    pub fn config(&self) -> &AgentDefaults {
        &self.config
    }

    /// 执行单个工具调用，返回 (工具名, 结果内容)
    async fn execute_one_tool(
        &self,
        tool_call: &nanobot_provider::ToolCall,
        tool_ctx: &ToolContext,
        session_key: &str,
    ) -> (String, String) {
        info!("执行工具 {}: {}", tool_call.name, tool_call.arguments);

        let args = match serde_json::from_str::<serde_json::Value>(&tool_call.arguments) {
            Ok(v) => v,
            Err(e) => {
                error!("解析工具 {} 参数失败: {}, 参数内容: {}", tool_call.name, e, tool_call.arguments);
                serde_json::Value::String(tool_call.arguments.clone())
            }
        };

        let tool_result = self.tool_registry.execute(tool_ctx, &tool_call.name, args).await;

        let result_content = match tool_result {
            Ok(output) => format!("Tool Call Result:\n{output}"),
            Err(e) => {
                error!("工具 {} 执行失败: {}", tool_call.name, e);
                format!("Tool Call Error: {e}")
            }
        };

        let result_content = crate::utils::maybe_persist_tool_result(
            &result_content,
            self.config.max_tool_result_chars,
            &tool_call.id,
            session_key,
            &self.config.workspace,
        );

        (tool_call.name.clone(), result_content)
    }
}

/// 检查消息内容是否为 /stop 命令
fn is_stop_cmd(content: &str) -> bool {
    content.trim_end().eq_ignore_ascii_case("/stop")
}

/// 清理思考内容中的特殊标记
///
/// 移除某些模型在内容中嵌入的 `<think>…</think>` 标签块。
/// 参考 Python 版本的 `_strip_think` 方法。
pub fn strip_think(text: &str) -> String {
    /// 用于匹配 think 标签的正则表达式（含孤立闭合标签）
    static THINK_REGEX: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"<think>[\s\S]*?</think>|</think>").expect("Invalid regex pattern"));

    THINK_REGEX.replace_all(text, "").trim().to_string()
}

/// 格式化工具调用为简洁提示
///
/// 将工具调用列表格式化为易读的字符串，例如 `web_search("query")`。
/// 参考 Python 版本的 `_tool_hint` 方法。
pub(crate) fn format_tool_hint(tool_calls: &[nanobot_provider::ToolCall]) -> String {
    tool_calls.iter().map(|tc| tc.preview()).collect::<Vec<_>>().join(", ")
}

/// 将工具调用分批：连续的只读工具分为一批（可并行），其余每个单独一批（串行）
fn partition_tool_batches<'a>(
    tool_calls: &'a [nanobot_provider::ToolCall],
    registry: &ToolRegistry,
) -> Vec<Vec<&'a nanobot_provider::ToolCall>> {
    let mut batches: Vec<Vec<&nanobot_provider::ToolCall>> = Vec::new();
    let mut current_batch: Vec<&nanobot_provider::ToolCall> = Vec::new();

    for tc in tool_calls {
        let is_read_only = registry.get(&tc.name).is_some_and(|t| t.read_only());
        if is_read_only {
            current_batch.push(tc);
        } else {
            if !current_batch.is_empty() {
                batches.push(std::mem::take(&mut current_batch));
            }
            batches.push(vec![tc]);
        }
    }
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }
    batches
}

/// 运行记忆整合任务（token-based 多轮整合）
///
/// 接收 Session 所有权，执行整合后返回 Session。
/// 无论成功与否，都会返回 Session 所有权。
///
/// 整合策略（对齐 Python 上游 `maybe_consolidate_by_tokens`）：
/// 1. 估算未整合消息的 token 总量
/// 2. 如果超过 `max_input_tokens`，触发整合
/// 3. 目标：压缩到 `max_input_tokens / 2`
/// 4. 最多 5 轮，每轮在 user 消息边界切割一批消息
/// 5. 连续 3 次 LLM 失败后降级为原文转储
async fn try_consolidate<P: Provider>(
    memory: Arc<nanobot_memory::MemoryStore>,
    provider: P,
    mut session: nanobot_session::Session,
    memory_window: usize,
    max_input_tokens: usize,
    consolidating: Arc<Mutex<HashSet<String>>>,
) -> Result<nanobot_session::Session, (nanobot_session::Session, anyhow::Error)> {
    let session_key = session.key.clone();

    // 条件1: 检查是否达到整合阈值
    let unconsolidated_tokens: usize =
        session.messages[session.last_consolidated..].iter().map(|m| m.token_len()).sum();

    let should_consolidate = if max_input_tokens > 0 {
        unconsolidated_tokens >= max_input_tokens
    } else {
        session.messages.len() - session.last_consolidated >= memory_window
    };

    if !should_consolidate {
        debug!("未达到整合阈值: session_key={session_key}");
        return Ok(session);
    }

    // 条件2: 会话没有进行中的整合任务
    if !consolidating.lock().await.insert(session_key.to_string()) {
        debug!("会话已有整合任务在进行中: session_key={session_key}");
        return Ok(session);
    }

    info!("启动 token-based 记忆整合: session_key={session_key}");

    let target = max_input_tokens / 2;
    let mut consecutive_failures = 0usize;

    for round in 0..nanobot_memory::MAX_CONSOLIDATION_ROUNDS {
        // 重新估算未整合消息 token
        let estimated: usize = session.messages[session.last_consolidated..].iter().map(|m| m.token_len()).sum();
        if estimated <= target {
            debug!("整合完成: estimated={estimated} <= target={target}");
            break;
        }

        // 找切割点
        let tokens_to_remove = estimated.saturating_sub(target).max(1);
        let boundary = nanobot_memory::MemoryStore::pick_consolidation_boundary(
            &session.messages,
            session.last_consolidated,
            tokens_to_remove,
        );

        let Some((end_idx, _removed)) = boundary else {
            debug!("无法找到合适的切割点 (round {round})");
            break;
        };

        let chunk: Vec<_> = session.messages[session.last_consolidated..end_idx].to_vec();
        if chunk.is_empty() {
            break;
        }

        info!(
            "整合 round {round}: estimated={estimated}, target={target}, chunk={} msgs, boundary={end_idx}",
            chunk.len()
        );

        // 调用 LLM 做摘要
        let result = memory
            .try_consolidate(&session.messages, session.last_consolidated, provider.clone(), false, memory_window)
            .await;

        match result {
            Ok(new_last) => {
                session.last_consolidated = new_last;
                consecutive_failures = 0;
            }
            Err(e) => {
                warn!("整合 round {round} 失败: {e}");
                consecutive_failures += 1;
                if consecutive_failures >= nanobot_memory::MAX_FAILURES_BEFORE_RAW_ARCHIVE {
                    warn!("连续 {consecutive_failures} 次失败，降级为原文转储");
                    if let Err(e) = memory.raw_archive(&chunk) {
                        error!("原文转储失败: {e}");
                    }
                    session.last_consolidated = end_idx;
                    consecutive_failures = 0;
                }
            }
        }
    }

    // 清除整合状态
    consolidating.lock().await.remove(&session_key);

    Ok(session)
}

#[cfg(test)]
mod tests;
