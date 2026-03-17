//! AgentLoop 核心实现
//!
//! AgentLoop 是 nanobot 的核心处理引擎，负责：
//! 1. 接收消息（通过入站消息接收端）
//! 2. 维护会话历史
//! 3. 调用 LLM
//! 4. 返回响应（通过出站消息发送端）

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use nanobot_config::{AgentDefaults, McpServerConfig};
use nanobot_context::ContextBuilder;
use nanobot_cron::{CronService, CronTool};
use nanobot_mcp::wrapper::connect;
use nanobot_provider::{Message, Provider};
use nanobot_session::SessionManager;
use nanobot_subagent::{SpawnTool, SubagentManager};
use nanobot_tools::{Tool, ToolContext, ToolRegistry};
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, error, info, warn};

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
pub struct AgentLoop<P: Provider + 'static> {
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
}

impl<P: Provider + 'static> AgentLoop<P> {
    /// 创建新的 AgentLoop 实例
    ///
    /// tool_registry 会根据 config 中的 workspace 参数自动构造。
    /// 如果提供了 mcp_configs，会自动连接 MCP 服务器并注册工具。
    ///
    /// # Arguments
    /// * `provider` - LLM 提供者实例
    /// * `config` - Agent 配置
    /// * `cron_service` - 可选的 Cron 服务实例
    /// * `subagent_manager` - 可选的子代理管理器
    /// * `mcp_configs` - 可选的 MCP 服务器配置
    pub async fn new(
        mut provider: P,
        config: AgentDefaults,
        cron_service: Option<Arc<CronService>>,
        subagent_manager: Option<Arc<SubagentManager<P>>>,
        mcp_configs: std::collections::HashMap<String, McpServerConfig>,
    ) -> Result<Self> {
        info!("初始化 AgentLoop: model={}, max_tool_iterations={}", config.model, config.max_tool_iterations);

        // 基于 config 构造 tool_registry
        let workspace_str = config.workspace.to_string_lossy();
        let mut tool_registry = ToolRegistry::new(&workspace_str, None);

        // 连接 MCP 服务器并注册工具
        let mcp_server_count = mcp_configs.len();
        let mut mcp_tool_count = 0;
        if !mcp_configs.is_empty() {
            info!("发现 {} 个 MCP 服务器配置", mcp_server_count);
            match connect(mcp_configs).await {
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

        // 如果提供了 subagent_manager，注册 SpawnTool
        if let Some(ref manager) = subagent_manager {
            let spawn_tool = SpawnTool::new(Arc::clone(manager));
            info!("注册 SpawnTool");
            tool_registry.register(spawn_tool);
        }

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
        let context = ContextBuilder::new(config.workspace.clone()).expect("Failed to initialize ContextBuilder");

        Ok(Self {
            provider,
            config,
            sessions,
            tool_registry,
            context,
            consolidating: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    /// 调用 LLM 并返回响应消息
    async fn call_llm(&self, messages: &[Message]) -> Result<Message> {
        debug!("调用 LLM: 消息数量={}", messages.len());

        let options = nanobot_provider::Options::default();
        let response = self.provider.chat(messages, &options).await?;

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
    ///
    /// # Returns
    /// ReActResult 包含最终结果、工具使用列表和消息历史
    pub async fn re_act(&self, mut messages: Vec<Message>, channel: &str, chat_id: &str) -> Result<ReActResult> {
        let max_iterations = self.config.max_tool_iterations;
        let mut iteration = 0;
        let mut tools_used: Vec<String> = Vec::new();

        info!("启动 ReAct 循环: max_iterations={}, 可用工具={:?}", max_iterations, self.tool_registry.tool_names());

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
                            error!("解析工具 {} 参数失败: {}, 参数内容: {}", tool_call.name, e, tool_call.arguments);
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

                info!("ReAct 循环完成: 迭代次数={}, 最终内容长度={} 字符", iteration, final_content.len());

                return Ok(ReActResult { content: final_content, tools_used, messages });
            }
        }

        // 达到最大迭代次数
        warn!("ReAct 循环达到最大迭代次数: {}", max_iterations);
        let warning_msg = format!(
            "I reached the maximum number of tool call iterations ({max_iterations}) without completing the task. You can try breaking the task into smaller steps."
        );

        messages.push(Message::assistant(&warning_msg));

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
    pub async fn process_direct(
        &self,
        content: &str,
        session_key: &str,
        channel: Option<&str>,
        chat_id: Option<&str>,
    ) -> Result<String> {
        info!("直接处理消息: {}", content);

        // 使用独立参数或默认值
        let channel = channel.unwrap_or("cli");
        let chat_id = chat_id.unwrap_or("direct");

        // 构造入站消息并复用 process_message
        let inbound = InboundMessage::new(channel, "user", chat_id, content);
        let outbound = self.process_message(inbound, Some(session_key)).await;

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
                match self.re_act(messages, target_channel, target_chat_id).await {
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
    ///
    /// # Returns
    /// - `Ok(OutboundMessage)`: 是命令（支持的或不支持的），并已正确处理
    /// - `Err(InboundMessage)`: 不是命令，返回入参的 InboundMessage 供后续处理
    async fn try_handle_cmd(&self, msg: InboundMessage) -> Result<OutboundMessage, InboundMessage> {
        // 检查是否以 `/` 开头
        if !msg.content.starts_with('/') {
            return Err(msg);
        }

        // 提取命令名称（去除前导 `/`），使用 to_lowercase() 和 trim() 处理
        let cmd = msg.content[1..].trim().to_lowercase();

        // 使用 match 结构处理已知命令，不支持的命令返回提示信息
        let response_content = match cmd.as_str() {
            "help" => {
                // 返回帮助信息（与 Python 版本一致）
                "🐈 nanobot commands:\n/new — Start a new conversation\n/help — Show available commands".to_owned()
            }
            // 不支持的命令返回提示信息
            _ => {
                format!("❌ Unsupported command: /{cmd}\nTry /help for available commands")
            }
        };

        Ok(OutboundMessage::new(msg.channel, msg.chat_id, response_content))
    }

    /// 处理入站消息并返回待发送的响应
    ///
    /// # Arguments
    /// * `inbound` - 入站消息
    /// * `session_key` - 可选的会话标识，格式为 "channel:chat_id"；不存在时从 inbound.session_key() 获取
    ///
    /// 注意：此方法总是返回 OutboundMessage，错误会被转换为错误消息内容
    async fn process_message(&self, inbound: InboundMessage, session_key: Option<&str>) -> OutboundMessage {
        // 系统消息：从 chat_id 解析目标路由（格式为 "channel:chat_id"）
        if inbound.channel == "system" {
            return self.process_system_message(inbound).await;
        }

        // 尝试处理命令
        let inbound = match self.try_handle_cmd(inbound.clone()).await {
            Ok(outbound) => return outbound,
            Err(msg) => msg,
        };

        // 获取或创建会话：优先使用传入的 session_key，否则从 inbound 获取
        let session_key = session_key.map(|s| s.to_string()).unwrap_or_else(|| inbound.session_key());
        let mut session = self.sessions.get_or_create(&session_key);

        let InboundMessage { channel, sender_id: _, chat_id, content, .. } = inbound;

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
            Arc::clone(&self.consolidating),
        ));

        // 使用 ContextBuilder 构建消息列表
        let messages = match self.context.build_messages(&history, &content, None, Some(&channel), Some(&chat_id)) {
            Ok(v) => v,
            Err(err) => {
                error!("构建消息失败: {err}");
                let error_msg = format!("构建消息失败: {err}");
                return OutboundMessage::new(&channel, &chat_id, &error_msg);
            }
        };

        let skip = messages.len() - 1; // 跳过系统消息 + 历史消息（不包括新消息）

        // 执行 ReAct 循环（支持工具调用）
        let result = match self.re_act(messages, &channel, &chat_id).await {
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
}

/// 执行记忆整合的异步函数
///
/// 此函数设计为可被 `tokio::spawn` 调用的独立异步任务。
/// 执行完成后会自动清理整合状态。
///
/// # Arguments
/// * `context` - 上下文构建器（包含 MemoryStore）
/// * `provider` - LLM 提供者
/// * `messages` - 会话消息列表
/// * `last_consolidated` - 上次整合位置
/// * `memory_window` - 记忆窗口大小
/// * `session_key` - 会话标识（用于状态清理）
/// * `consolidating` - 整合状态集合
///
/// # Returns
/// 如果整合成功且产生了新的 last_consolidated，返回 Some(new_last)；否则返回 None
/// 运行记忆整合任务
///
/// 接收 Session 所有权，执行整合后返回 Session。
/// 无论成功与否，都会返回 Session 所有权。
///
/// # Returns
/// - `Ok((session, Some(new_last)))`: 整合成功，last_consolidated 已更新
/// - `Ok(session)`: 整合成功，session.last_consolidated 已更新
/// - `Err((session, error))`: 整合失败，返回错误信息
async fn try_consolidate<P: Provider + 'static>(
    memory: Arc<nanobot_memory::MemoryStore>,
    provider: P,
    mut session: nanobot_session::Session,
    memory_window: usize,
    consolidating: Arc<Mutex<HashSet<String>>>,
) -> Result<nanobot_session::Session, (nanobot_session::Session, anyhow::Error)> {
    let session_key = session.key.clone();
    let last_consolidated = session.last_consolidated;

    // 条件1: 消息数量是否达到阈值
    let window_reached = session.messages.len() - last_consolidated >= memory_window;
    if !window_reached {
        debug!("消息数量未达到整合阈值: session_key={}", session_key);
        return Ok(session);
    }

    // 条件2: 会话没有进行中的整合任务
    if !consolidating.lock().await.insert(session_key.to_string()) {
        debug!("会话已有整合任务在进行中: session_key={}", session_key);
        return Ok(session);
    }

    info!("启动异步记忆整合任务: session_key={}", session_key);

    let messages = session.messages.clone();

    let result = memory.try_consolidate(&messages, last_consolidated, provider, false, memory_window).await;

    // 清除整合状态（确保在任务完成时清理）
    consolidating.lock().await.remove(&session_key);

    // 转换结果：成功时更新 session.last_consolidated
    match result {
        Ok(new_last) => {
            if new_last != last_consolidated {
                session.last_consolidated = new_last;
            }
            Ok(session)
        }
        Err(e) => {
            error!("Memory consolidation error: {}", e);
            Err((session, anyhow::anyhow!("Memory consolidation error: {e}")))
        }
    }
}

#[cfg(test)]
mod tests;
