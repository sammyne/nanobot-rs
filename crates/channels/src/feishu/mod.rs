//! 飞书通道实现
//!
//! 实现 Feishu 通道，支持 WebSocket 接收消息和 HTTP API 发送消息。
//! 使用 feishu-sdk v0.1.2 提供支持。

mod logger;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use feishu_sdk::client::Client;
use feishu_sdk::core::{Config, RequestOptions};
use feishu_sdk::event::{Event, EventDispatcher, EventDispatcherConfig, EventHandler, EventHandlerResult};
use feishu_sdk::ws::stream::{StreamClientBuilder, StreamConfig};
use nanobot_config::FeishuConfig;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

use crate::error::{ChannelError, ChannelResult};
use crate::messages::{InboundMessage, OutboundMessage};
use crate::traits::Channel;

/// 飞书通道
pub struct Feishu {
    /// 配置
    config: FeishuConfig,

    /// 飞书 SDK 配置（用于创建 WebSocket 客户端）
    feishu_config: Config,

    /// 飞书客户端（用于发送消息）
    client: Client,

    /// 运行状态
    running: Arc<RwLock<bool>>,

    /// 任务句柄
    task_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,

    /// 通道名称
    name: String,

    /// 入站消息发送端
    inbound_tx: mpsc::Sender<InboundMessage>,

    /// 消息上下文（chat_id -> 原始 Event）
    message_context: Arc<RwLock<HashMap<String, Event>>>,
}

impl Feishu {
    /// 创建新的飞书通道
    ///
    /// # 参数
    ///
    /// * `config` - 飞书配置
    /// * `inbound_tx` - 入站消息发送端，用于将接收到的消息发送给外部
    pub async fn new(config: FeishuConfig, inbound_tx: mpsc::Sender<InboundMessage>) -> ChannelResult<Self> {
        // 创建飞书客户端配置
        let feishu_config = Config::builder(&config.app_id, &config.app_secret).build();

        // 创建飞书客户端
        let client = Client::new(feishu_config.clone())
            .map_err(|e| ChannelError::StartFailed(format!("创建飞书客户端失败: {e}")))?;

        Ok(Self {
            config,
            feishu_config,
            client,
            running: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(RwLock::new(None)),
            name: "feishu".to_string(),
            inbound_tx,
            message_context: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 检查权限
    fn check_permission(&self, sender_id: &str) -> bool {
        if self.config.allow_from.is_empty() {
            return true;
        }

        self.config.allow_from.contains(&sender_id.to_string())
            || sender_id.contains('|')
                && sender_id.split('|').any(|part| self.config.allow_from.contains(&part.to_string()))
    }

    /// 处理消息事件
    async fn process_message(&self, event: Event) {
        // 提取事件数据
        let Some(ref event_data) = event.event else {
            warn!("事件数据为空");
            return;
        };

        // 尝试解析为消息事件
        let msg_event = match serde_json::from_value::<feishu_sdk::event::models::im::MessageEvent>(event_data.clone())
        {
            Ok(event) => event,
            Err(e) => {
                warn!("解析消息事件失败: {:?}", e);
                return;
            }
        };

        // 获取聊天 ID
        let Some(chat_id) = msg_event.message.chat_id.as_deref() else {
            warn!("消息事件缺少 chat_id");
            return;
        };
        let chat_id = chat_id.to_string();

        // 获取发送者信息
        let sender_id = msg_event.sender.sender_id.as_ref().and_then(|id| id.open_id.as_deref()).unwrap_or_default();
        let sender_type = msg_event.sender.sender_type.as_deref().unwrap_or_default();

        // 检查消息类型
        let message_type = msg_event.message.message_type.as_deref().unwrap_or("unknown");
        if message_type != "text" {
            warn!("忽略非文本消息类型: {}", message_type);
            return;
        }

        // 获取文本内容
        let Some(content_str) = msg_event.message.content.as_deref() else {
            warn!("收到空消息");
            return;
        };

        let content = match serde_json::from_str::<serde_json::Value>(content_str) {
            Ok(json) => json.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            Err(_) => String::new(),
        };

        let content = content.trim();
        if content.is_empty() {
            warn!("收到空消息");
            return;
        }

        // 权限检查
        if !self.check_permission(sender_id) {
            warn!("未授权的消息，发送者 ID: {}", sender_id);
            return;
        }

        info!("收到飞书消息，发送者: {} ({}), 聊天 ID: {}, 内容: {}", sender_id, sender_type, chat_id, content);

        // 保存原始事件到上下文
        self.message_context.write().await.insert(chat_id.clone(), event);

        // 构造入站消息
        let inbound_msg = InboundMessage::new("feishu", sender_id, &chat_id, content);

        // 发送入站消息
        if let Err(e) = self.inbound_tx.send(inbound_msg).await {
            error!("发送入站消息失败: {}", e);
        }
    }
}

/// 飞书消息事件处理器
struct FeishuEventHandler {
    channel: Feishu,
}

impl FeishuEventHandler {
    fn new(channel: Feishu) -> Self {
        Self { channel }
    }
}

impl EventHandler for FeishuEventHandler {
    fn event_type(&self) -> &str {
        "im.message.receive_v1"
    }

    fn handle(&self, event: Event) -> Pin<Box<dyn std::future::Future<Output = EventHandlerResult> + Send + '_>> {
        Box::pin(async move {
            debug!("收到飞书事件: {:?}", event.event_type());

            // 处理消息
            self.channel.process_message(event).await;

            Ok(None)
        })
    }
}

// 实现 Clone trait 以支持传递给事件处理器
impl Clone for Feishu {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            feishu_config: self.feishu_config.clone(),
            client: self.client.clone(),
            running: Arc::clone(&self.running),
            task_handle: Arc::clone(&self.task_handle),
            name: self.name.clone(),
            inbound_tx: self.inbound_tx.clone(),
            message_context: Arc::clone(&self.message_context),
        }
    }
}

#[async_trait]
impl Channel for Feishu {
    async fn start(&self) -> ChannelResult<()> {
        info!("启动飞书通道");

        *self.running.write().await = true;

        // 创建流客户端配置
        let stream_config = StreamConfig::new()
            .locale("zh")
            .auto_reconnect(true)
            .reconnect_interval(tokio::time::Duration::from_secs(5))
            .ping_interval(tokio::time::Duration::from_secs(30));

        // 创建事件分发器配置
        let dispatcher_config =
            EventDispatcherConfig::new().verification_token("").encrypt_key("").skip_signature_verification(true);

        // 创建事件分发器
        let dispatcher = EventDispatcher::new(dispatcher_config, Arc::new(logger::TracingLogger));

        // 注册消息事件处理器
        dispatcher.register_handler(Box::new(FeishuEventHandler::new(self.clone()))).await;

        // 创建流客户端
        let stream_client = StreamClientBuilder::new(self.feishu_config.clone())
            .stream_config(stream_config)
            .event_dispatcher(dispatcher)
            .build()
            .map_err(|e| ChannelError::StartFailed(format!("创建 WebSocket 客户端失败: {e}")))?;

        // 启动客户端（在后台任务中）
        let handle = tokio::spawn(async move {
            info!("飞书 WebSocket 客户端启动");

            // 使用 spawn 方法启动连接
            match stream_client.spawn().await {
                Ok(Ok(())) => info!("飞书 WebSocket 连接关闭"),
                Ok(Err(e)) => error!("飞书 WebSocket 错误: {}，将自动重连", e),
                Err(e) => error!("WebSocket 任务错误: {:?}", e),
            }

            info!("飞书 WebSocket 停止");
        });

        *self.task_handle.write().await = Some(handle);

        info!("飞书通道启动成功");
        Ok(())
    }

    async fn stop(&self) -> ChannelResult<()> {
        info!("停止飞书通道");

        *self.running.write().await = false;

        // 取消后台任务
        if let Some(handle) = self.task_handle.write().await.take() {
            handle.abort();
        }

        info!("飞书通道已停止");
        Ok(())
    }

    async fn send(&self, msg: OutboundMessage) -> ChannelResult<()> {
        debug!("发送飞书消息到: {}, 内容: {}", msg.chat_id, msg.content);

        // 从上下文中获取原始消息
        let context = self.message_context.read().await;
        let Some(original_event) = context.get(&msg.chat_id).cloned() else {
            warn!("找不到 chat_id {} 的消息上下文，可能消息已过期", msg.chat_id);
            return Err(ChannelError::SendFailed(format!("找不到消息上下文: {}", msg.chat_id)));
        };
        drop(context);

        // 提取原始消息事件的数据
        let Some(event_data) = original_event.event else {
            return Err(ChannelError::SendFailed("原始事件数据为空".to_string()));
        };

        // 尝试解析为消息事件以获取 chat_id 和其他信息
        let msg_event = match serde_json::from_value::<feishu_sdk::event::models::im::MessageEvent>(event_data) {
            Ok(event) => event,
            Err(e) => {
                return Err(ChannelError::SendFailed(format!("解析原始消息事件失败: {e}")));
            }
        };

        let chat_id = msg_event.message.chat_id.unwrap_or_default();

        // 使用 Markdown 格式化消息
        let markdown_content = format!("**Nanobot Reply**\n\n{}", msg.content);

        // 构建消息内容 JSON
        let content_json = serde_json::json!({
            "config": {
                "wide_screen_mode": true
            },
            "elements": [
                {
                    "tag": "div",
                    "text": {
                        "content": markdown_content,
                        "tag": "lark_md"
                    }
                }
            ]
        });

        use feishu_sdk::api::{SendMessageBody, SendMessageQuery};

        // 构建消息体
        let body = SendMessageBody {
            receive_id: chat_id,
            msg_type: "interactive".to_string(),
            content: serde_json::to_string(&content_json).unwrap_or_default(),
            uuid: None,
        };

        // 构建查询参数
        let query = SendMessageQuery { receive_id_type: Some("chat_id".to_string()) };

        // 发送消息
        let response = self
            .client
            .im_v1_message()
            .send_typed(&query, &body, RequestOptions::default())
            .await
            .map_err(|e| ChannelError::SendFailed(format!("发送飞书消息失败: {e}")))?;

        if response.code != 0 {
            error!("发送飞书消息失败: code={}, msg={}", response.code, response.msg);
            return Err(ChannelError::SendFailed(format!("发送飞书消息失败: {}", response.msg)));
        }

        debug!("飞书消息发送成功");
        Ok(())
    }

    fn is_running(&self) -> bool {
        if let Ok(running) = self.running.try_read() { *running } else { false }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests;
