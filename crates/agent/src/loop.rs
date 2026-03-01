//! AgentLoop 核心实现
//!
//! AgentLoop 是 nanobot 的核心处理引擎，负责：
//! 1. 接收消息（通过入站消息接收端）
//! 2. 维护会话历史
//! 3. 调用 LLM
//! 4. 返回响应（通过出站消息发送端）

use crate::bus::{InboundMessage, OutboundMessage};
use anyhow::Result;
use nanobot_config::AgentDefaults;
use nanobot_provider::{Message, Provider};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info};

/// 会话键（channel:chat_id）
type SessionKey = String;

/// Agent 循环处理引擎
///
/// 负责管理消息处理和 LLM 调用的完整生命周期。
///
/// 消息流向：
/// - CLI -> AgentLoop: CLI 通过 inbound_tx 发送，AgentLoop 通过 inbound_rx 接收
/// - AgentLoop -> CLI: AgentLoop 通过 outbound_tx 发送，CLI 通过 outbound_rx 接收
pub struct AgentLoop {
    /// LLM 提供者实例
    provider: Arc<dyn Provider>,

    /// Agent 配置
    config: AgentDefaults,

    /// 入站消息接收端（从 CLI 接收）
    inbound_rx: mpsc::Receiver<InboundMessage>,

    /// 出站消息发送端（向 CLI 发送）
    outbound_tx: mpsc::Sender<OutboundMessage>,

    /// 会话历史（channel:chat_id -> messages）
    sessions: Arc<RwLock<HashMap<SessionKey, Vec<Message>>>>,
}

impl AgentLoop {
    /// 创建新的 AgentLoop 实例（单次消息模式，无通道）
    ///
    /// 适用于 `process_direct` 等不需要后台循环的场景。
    ///
    /// # Arguments
    /// * `provider` - LLM 提供者实例
    /// * `config` - Agent 配置
    pub fn new_direct(provider: Arc<dyn Provider>, config: AgentDefaults) -> Self {
        info!(
            "初始化 AgentLoop (单次模式): model={}, max_tool_iterations={}",
            config.model, config.max_tool_iterations
        );

        Self {
            provider,
            config,
            inbound_rx: mpsc::channel(1).1, // 创建一个虚拟的接收端，不会使用
            outbound_tx: mpsc::channel(1).0, // 创建一个虚拟的发送端，不会使用
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新的 AgentLoop 实例（交互式模式，需要通道）
    ///
    /// 适用于需要启动后台消息循环的场景。
    ///
    /// # Arguments
    /// * `provider` - LLM 提供者实例
    /// * `config` - Agent 配置
    /// * `inbound_rx` - 入站消息接收端（CLI -> AgentLoop）
    /// * `outbound_tx` - 出站消息发送端（AgentLoop -> CLI）
    pub fn new(
        provider: Arc<dyn Provider>,
        config: AgentDefaults,
        inbound_rx: mpsc::Receiver<InboundMessage>,
        outbound_tx: mpsc::Sender<OutboundMessage>,
    ) -> Self {
        info!(
            "初始化 AgentLoop (交互式模式): model={}, max_tool_iterations={}",
            config.model, config.max_tool_iterations
        );

        Self {
            provider,
            config,
            inbound_rx,
            outbound_tx,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取会话键
    fn session_key(channel: &str, chat_id: &str) -> SessionKey {
        format!("{}:{}", channel, chat_id)
    }

    /// 获取或创建会话历史
    async fn get_or_create_session(&self, channel: &str, chat_id: &str) -> Vec<Message> {
        let key = Self::session_key(channel, chat_id);
        let sessions = self.sessions.read().await;

        if let Some(messages) = sessions.get(&key) {
            return messages.clone();
        }

        drop(sessions);

        // 创建新的会话，添加系统提示词
        let mut new_messages = Vec::new();
        new_messages.push(Message::system("你是一个有帮助的 AI 助手。"));

        let mut sessions = self.sessions.write().await;
        sessions.insert(key.clone(), new_messages.clone());

        new_messages
    }

    /// 更新会话历史
    async fn update_session(&self, channel: &str, chat_id: &str, messages: &[Message]) {
        let key = Self::session_key(channel, chat_id);
        let mut sessions = self.sessions.write().await;

        // 限制历史长度（保持最后memory_window条消息）
        let max_history = self.config.memory_window;
        let trimmed = if messages.len() > max_history && max_history > 1 {
            let mut result = vec![messages[0].clone()]; // 保留系统消息
            let start_idx = messages.len().saturating_sub(max_history - 1);
            result.extend_from_slice(&messages[start_idx..]);
            result
        } else {
            messages.to_vec()
        };

        sessions.insert(key, trimmed);
    }

    /// 调用 LLM 并返回响应
    async fn call_llm(&self, messages: &[Message]) -> Result<String> {
        debug!("调用 LLM: 消息数量={}", messages.len());

        let response = self.provider.chat(messages).await?;

        info!("收到 LLM 响应, 长度: {} 字符", response.len());

        Ok(response)
    }

    /// 直接处理消息（单次调用模式）
    ///
    /// 参考 Python 版 `process_direct` 函数实现。
    pub async fn process_direct(&self, content: &str, session_key: Option<&str>) -> Result<String> {
        info!("直接处理消息: {}", content);

        // 解析会话标识
        let (channel, chat_id) = if let Some(key) = session_key {
            let parts: Vec<&str> = key.splitn(2, ':').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("cli".to_string(), key.to_string())
            }
        } else {
            ("cli".to_string(), "direct".to_string())
        };

        // 获取或创建会话历史
        let mut messages = self.get_or_create_session(&channel, &chat_id).await;

        // 添加用户消息
        messages.push(Message::user(content));

        // 调用 LLM
        let response = self.call_llm(&messages).await?;

        // 添加助手回复到历史
        messages.push(Message::assistant(&response));
        self.update_session(&channel, &chat_id, &messages).await;

        Ok(response)
    }

    /// 启动后台消息处理循环
    ///
    /// 这是交互式模式的核心方法。从入站通道接收消息，
    /// 处理后发送给出站通道。
    ///
    /// 循环在以下情况下会退出：
    /// - 入站通道关闭
    /// - 发生错误
    pub async fn run(mut self) -> Result<()> {
        let outbound_tx = self.outbound_tx.clone();

        info!("AgentLoop 后台循环已启动");

        loop {
            // 消费入站消息
            match self.inbound_rx.recv().await {
                Some(msg) => {
                    debug!(
                        "收到入站消息: channel={}, chat_id={}",
                        msg.channel, msg.chat_id
                    );

                    // 处理消息
                    if let Err(e) = self.handle_message(msg, &outbound_tx).await {
                        error!("处理消息失败: {}", e);
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

    /// 处理入站消息并发送响应
    async fn handle_message(
        &self,
        inbound: InboundMessage,
        outbound_tx: &mpsc::Sender<OutboundMessage>,
    ) -> Result<()> {
        let InboundMessage {
            channel,
            sender_id: _,
            chat_id,
            content,
            ..
        } = inbound;

        // 获取或创建会话历史
        let mut messages = self.get_or_create_session(&channel, &chat_id).await;

        // 添加用户消息
        messages.push(Message::user(&content));

        // 调用 LLM
        match self.call_llm(&messages).await {
            Ok(response) => {
                // 添加助手回复到历史
                messages.push(Message::assistant(&response));
                self.update_session(&channel, &chat_id, &messages).await;

                // 发送出站消息
                let outbound = OutboundMessage::new(&channel, &chat_id, &response);
                if let Err(e) = outbound_tx.send(outbound).await {
                    error!("发送出站消息失败: {}", e);
                }
            }
            Err(e) => {
                error!("处理消息失败: {}", e);

                // 发送错误消息
                let error_msg = format!("处理失败: {}", e);
                let outbound = OutboundMessage::new(&channel, &chat_id, &error_msg);
                if let Err(e) = outbound_tx.send(outbound).await {
                    error!("发送错误消息失败: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 获取配置
    pub fn config(&self) -> &AgentDefaults {
        &self.config
    }
}
