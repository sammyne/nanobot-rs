//! 钉钉通道实现
//!
//! 实现 DingTalk 通道，支持 Stream Mode (WebSocket) 接收消息和 HTTP API 发送消息。
//! 使用 dingtalk-stream 库提供 Stream Mode 支持。

use std::sync::Arc;

use async_trait::async_trait;
use dingtalk_stream::messages::frames::MessageBody;
use dingtalk_stream::transport::http::HttpClient;
use dingtalk_stream::transport::token::TokenManager;
use dingtalk_stream::{
    AsyncChatbotHandler, ChatbotMessage, ChatbotReplier, ClientBuilder, Credential, DingTalkStreamClient,
};
use serde_json::Value;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

use crate::config::DingTalkConfig;
use crate::error::{ChannelError, ChannelResult};
use crate::messages::{InboundMessage, OutboundMessage};
use crate::traits::Channel;

/// 钉钉通道
pub struct DingTalk {
    /// 配置
    config: DingTalkConfig,

    /// HTTP 客户端（用于发送消息）
    http_client: HttpClient,

    /// 钉钉凭证
    credential: Credential,

    /// Token 管理器
    token_manager: Arc<TokenManager>,

    /// 运行状态
    running: Arc<RwLock<bool>>,

    /// 任务句柄
    task_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,

    /// 通道名称
    name: String,

    /// 入站消息发送端
    inbound_tx: mpsc::Sender<InboundMessage>,
}

impl DingTalk {
    /// 创建新的钉钉通道
    ///
    /// # 参数
    ///
    /// * `config` - 钉钉配置
    /// * `inbound_tx` - 入站消息发送端，用于将接收到的消息发送给外部
    pub async fn new(config: DingTalkConfig, inbound_tx: mpsc::Sender<InboundMessage>) -> ChannelResult<Self> {
        config.validate()?;

        // 注意：dingtalk-stream 的 HttpClient 目前不支持代理配置
        // 如果需要代理，可以设置环境变量 HTTP_PROXY/HTTPS_PROXY
        let http_client = HttpClient::new();

        // 创建钉钉凭证
        let credential = Credential::new(&config.client_id, &config.client_secret);

        // 创建 Token 管理器
        let token_manager = Arc::new(TokenManager::new(credential.clone(), http_client.clone()));

        Ok(Self {
            config,
            http_client,
            credential,
            token_manager,
            running: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(RwLock::new(None)),
            name: "dingtalk".to_string(),
            inbound_tx,
        })
    }

    /// 检查权限
    fn check_permission(&self, sender_id: &str) -> bool {
        if self.config.allow_from.is_empty() {
            return true;
        }

        if self.config.allow_from.contains(&sender_id.to_string()) {
            return true;
        }

        if sender_id.contains('|') {
            for part in sender_id.split('|') {
                if self.config.allow_from.contains(&part.to_string()) {
                    return true;
                }
            }
        }

        false
    }

    /// 处理消息
    fn process_message(&self, msg: ChatbotMessage) {
        // 获取文本内容
        let content = msg
            .get_text_list()
            .and_then(|list| list.into_iter().next())
            .unwrap_or_default();

        if content.trim().is_empty() {
            warn!("收到空消息");
            return;
        }

        // 获取发送者信息
        let sender_id = msg
            .sender_staff_id
            .clone()
            .unwrap_or_else(|| msg.sender_id.clone().unwrap_or_default());
        let sender_nick = msg.sender_nick.clone().unwrap_or_default();

        // 权限检查
        if !self.check_permission(&sender_id) {
            warn!("未授权的消息，发送者 ID: {}", sender_id);
            return;
        }

        info!(
            "收到钉钉消息，发送者: {} ({})，内容: {}",
            sender_nick, sender_id, content
        );

        // 获取聊天 ID（优先使用 conversation_id，其次使用 sender_id）
        let chat_id = msg.conversation_id.clone().unwrap_or_else(|| sender_id.clone());

        // 构造入站消息
        let inbound_msg = InboundMessage::new("dingtalk", &sender_id, &chat_id, &content);

        // 异步发送入站消息
        let inbound_tx = self.inbound_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = inbound_tx.send(inbound_msg).await {
                error!("发送入站消息失败: {}", e);
            }
        });
    }
}

impl AsyncChatbotHandler for DingTalk {
    fn process(&self, callback_message: &MessageBody) {
        // 解析 data 字段为 JSON Value
        if let Ok(data_value) = serde_json::from_str::<Value>(&callback_message.data) {
            if let Ok(msg) = ChatbotMessage::from_value(&data_value) {
                self.process_message(msg);
            } else {
                warn!("无法解析钉钉消息");
            }
        } else {
            warn!("无法解析消息 JSON");
        }
    }
}

#[async_trait]
impl Channel for DingTalk {
    async fn start(&self) -> ChannelResult<()> {
        info!("启动钉钉通道");

        *self.running.write().await = true;

        // 创建 Stream 客户端
        let mut client: DingTalkStreamClient = ClientBuilder::new(self.credential.clone())
            .register_async_chatbot_handler(ChatbotMessage::TOPIC, self.clone())
            .build();

        let running = self.running.clone();

        // 启动客户端（在后台任务中）
        let handle = tokio::spawn(async move {
            info!("钉钉 Stream Mode 客户端启动");

            loop {
                match client.start().await {
                    Ok(()) => {
                        info!("钉钉 Stream Mode 连接关闭");
                    }
                    Err(e) => {
                        error!("钉钉 Stream Mode 错误: {}，将自动重连", e);
                    }
                }

                // 检查是否应该停止
                if !*running.read().await {
                    info!("钉钉 Stream Mode 停止");
                    break;
                }

                // 等待重连
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        *self.task_handle.write().await = Some(handle);

        info!("钉钉通道启动成功");
        Ok(())
    }

    async fn stop(&self) -> ChannelResult<()> {
        info!("停止钉钉通道");

        *self.running.write().await = false;

        // 取消后台任务
        if let Some(handle) = self.task_handle.write().await.take() {
            handle.abort();
        }

        info!("钉钉通道已停止");
        Ok(())
    }

    async fn send(&self, msg: OutboundMessage) -> ChannelResult<()> {
        debug!("发送钉钉消息到: {}, 内容: {}", msg.chat_id, msg.content);

        // ChatbotReplier 会自动从 TokenManager 获取 token
        let replier = ChatbotReplier::new(
            self.http_client.clone(),
            Arc::clone(&self.token_manager),
            self.config.client_id.clone(),
        );

        // 构造一个模拟的 ChatbotMessage 用于回复
        let incoming_msg = ChatbotMessage {
            sender_staff_id: Some(msg.chat_id.clone()),
            sender_id: Some(msg.chat_id.clone()),
            conversation_type: Some("1".to_string()),
            conversation_id: Some(msg.chat_id.clone()),
            ..Default::default()
        };

        // 发送 Markdown 消息
        let markdown_content = format!("**Nanobot Reply**\n\n{}", msg.content);

        replier
            .reply_markdown("Nanobot", &markdown_content, &incoming_msg)
            .await
            .map_err(|e| ChannelError::SendFailed(format!("发送钉钉消息失败: {}", e)))?;

        debug!("钉钉消息发送成功");
        Ok(())
    }

    fn is_running(&self) -> bool {
        if let Ok(running) = self.running.try_read() {
            *running
        } else {
            false
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// 实现 Clone 以便可以传递给 handler
impl Clone for DingTalk {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            credential: self.credential.clone(),
            token_manager: self.token_manager.clone(),
            running: Arc::clone(&self.running),
            task_handle: Arc::clone(&self.task_handle),
            name: self.name.clone(),
            inbound_tx: self.inbound_tx.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
