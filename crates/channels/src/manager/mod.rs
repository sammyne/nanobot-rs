//! 通道管理器模块
//!
//! 提供统一的通道管理和消息路由功能。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::config::{ChannelsConfig, DingTalkConfig};
use crate::dingtalk::DingTalk;
use crate::error::{ChannelError, ChannelResult};
use crate::messages::{InboundMessage, OutboundMessage};
use crate::traits::Channel;

/// 通道状态
#[derive(Debug, Clone)]
pub struct ChannelStatus {
    /// 通道名称
    pub name: String,
    /// 是否正在运行
    pub running: bool,
}

/// 通道管理器
///
/// 负责管理所有通道的生命周期和消息路由。
pub struct ChannelManager {
    /// 所有通道的集合
    channels: HashMap<String, Arc<dyn Channel>>,

    /// 配置
    config: ChannelsConfig,

    /// 出站消息接收端
    outbound_rx: Option<mpsc::Receiver<OutboundMessage>>,

    /// 入站消息发送端
    inbound_tx: mpsc::Sender<InboundMessage>,

    /// 出站消息监听任务句柄
    outbound_task_handle: Option<JoinHandle<()>>,

    /// 通道启动任务句柄
    channel_task_handles: HashMap<String, JoinHandle<()>>,
}

impl ChannelManager {
    /// 创建新的通道管理器
    ///
    /// # 参数
    ///
    /// * `config` - 通道配置
    /// * `outbound_rx` - 出站消息接收端，用于监听从外部发来的出站消息
    /// * `inbound_tx` - 入站消息发送端，用于将接收到的消息发送给外部
    ///
    /// # 返回
    ///
    /// 返回包含所有已创建通道的通道管理器
    pub async fn new(
        config: ChannelsConfig,
        outbound_rx: mpsc::Receiver<OutboundMessage>,
        inbound_tx: mpsc::Sender<InboundMessage>,
    ) -> ChannelResult<Self> {
        info!("初始化通道管理器");

        let mut manager = Self {
            channels: HashMap::new(),
            config,
            outbound_rx: Some(outbound_rx),
            inbound_tx,
            outbound_task_handle: None,
            channel_task_handles: HashMap::new(),
        };

        // 创建启用的通道
        if let Some(dingtalk_config) = &manager.config.dingtalk
            && dingtalk_config.enabled
        {
            manager.add_dingtalk_channel(dingtalk_config.clone()).await?;
        }

        info!("通道管理器初始化完成，共 {} 个通道", manager.channels.len());
        Ok(manager)
    }

    /// 添加钉钉通道
    async fn add_dingtalk_channel(&mut self, config: DingTalkConfig) -> ChannelResult<()> {
        info!("添加钉钉通道");

        let channel = DingTalk::new(config, self.inbound_tx.clone()).await?;
        let name = channel.name().to_string();

        self.channels.insert(name.clone(), Arc::new(channel));

        info!("钉钉通道添加成功: {}", name);
        Ok(())
    }

    /// 启动所有通道
    ///
    /// 以非阻塞方式并发启动所有已配置的通道。
    /// 每个通道在独立的 tokio 任务中启动，避免相互阻塞。
    /// 同时启动出站消息监听任务，自动转发消息到目标通道。
    pub async fn start_all(&mut self) -> ChannelResult<()> {
        info!("开始启动所有通道");

        // 启动出站消息监听任务
        if let Some(outbound_rx) = self.outbound_rx.take() {
            let channels = Arc::new(self.channels.clone());

            let handle = tokio::spawn(async move {
                info!("出站消息监听任务启动");
                let mut rx = outbound_rx;

                while let Some(msg) = rx.recv().await {
                    let channel_name = msg.channel.clone();

                    if let Some(channel) = channels.get(&channel_name) {
                        if let Err(e) = channel.send(msg).await {
                            error!("转发消息到通道 {} 失败: {}", channel_name, e);
                        }
                    } else {
                        warn!("目标通道 {} 不存在，无法转发消息", channel_name);
                    }
                }

                info!("出站消息监听任务退出");
            });

            self.outbound_task_handle = Some(handle);
        }

        // 启动各个通道
        let channel_names: Vec<String> = self.channels.keys().cloned().collect();

        for name in channel_names {
            if let Some(channel) = self.channels.get(&name) {
                let channel = Arc::clone(channel);
                let name_clone = name.clone();

                // 为每个通道创建独立的 tokio 任务
                let handle = tokio::spawn(async move {
                    match channel.start().await {
                        Ok(_) => {
                            info!("通道 {} 启动成功", name_clone);
                        }
                        Err(e) => {
                            error!("通道 {} 启动失败: {}", name_clone, e);
                        }
                    }
                });

                self.channel_task_handles.insert(name, handle);
            }
        }

        info!("所有通道启动任务已创建，共 {} 个", self.channel_task_handles.len());
        Ok(())
    }

    /// 停止所有通道
    ///
    /// 停止所有正在运行的通道并清理资源。
    /// 先停止出站消息监听任务，再停止各通道。
    pub async fn stop_all(&mut self) -> ChannelResult<()> {
        info!("开始停止所有通道");

        // 先停止出站消息监听任务
        if let Some(handle) = self.outbound_task_handle.take() {
            handle.abort();
            info!("出站消息监听任务已停止");
        }

        // 等待并清理通道启动任务句柄
        let task_handles: Vec<(String, JoinHandle<()>)> = self.channel_task_handles.drain().collect();
        for (name, handle) in task_handles {
            // 不等待启动任务，直接中止
            handle.abort();
            debug!("通道 {} 启动任务已中止", name);
        }

        // 停止所有通道
        let channel_names: Vec<String> = self.channels.keys().cloned().collect();

        for name in channel_names {
            if let Some(channel) = self.channels.get(&name) {
                match channel.stop().await {
                    Ok(_) => {
                        info!("通道 {} 停止成功", name);
                    }
                    Err(e) => {
                        error!("通道 {} 停止失败: {}", name, e);
                    }
                }
            }
        }

        self.channels.clear();
        info!("所有通道已停止");

        Ok(())
    }

    /// 路由消息到指定通道
    ///
    /// 根据消息的 channel 字段将消息路由到对应的通道。
    ///
    /// # 参数
    ///
    /// * `msg` - 要发送的消息
    pub async fn route_message(&self, msg: OutboundMessage) -> ChannelResult<()> {
        let channel_name = &msg.channel;

        if let Some(channel) = self.channels.get(channel_name) {
            channel.send(msg).await
        } else {
            warn!("目标通道 {} 不存在，无法发送消息", channel_name);
            Err(ChannelError::SendFailed(format!("通道 {channel_name} 不存在")))
        }
    }

    /// 获取所有通道的状态
    ///
    /// 返回所有通道的当前运行状态。
    pub async fn get_status(&self) -> Vec<ChannelStatus> {
        let mut status_list = Vec::new();

        for (name, channel) in &self.channels {
            status_list.push(ChannelStatus {
                name: name.clone(),
                running: channel.is_running(),
            });
        }

        status_list
    }

    /// 获取指定通道的状态
    ///
    /// # 参数
    ///
    /// * `name` - 通道名称
    pub async fn get_channel_status(&self, name: &str) -> Option<ChannelStatus> {
        self.channels.get(name).map(|channel| ChannelStatus {
            name: name.to_string(),
            running: channel.is_running(),
        })
    }
}

#[cfg(test)]
mod tests;
