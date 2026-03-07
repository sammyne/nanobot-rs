//! 通道管理器模块
//!
//! 提供统一的通道管理和消息路由功能。

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{error, info, warn};

use crate::config::{ChannelsConfig, DingTalkConfig};
use crate::dingtalk::DingTalk;
use crate::error::{ChannelError, ChannelResult};
use crate::messages::OutboundMessage;
use crate::traits::Channel;

/// 通道状态
#[derive(Debug, Clone)]
pub struct ChannelStatus {
    /// 通道名称
    pub name: String,
    /// 是否正在运行
    pub running: bool,
}

/// 消息回调类型
pub type MessageCallback = Arc<dyn Fn(String, crate::messages::InboundMessage) + Send + Sync>;

/// 通道管理器
///
/// 负责管理所有通道的生命周期和消息路由。
pub struct ChannelManager {
    /// 所有通道的集合
    channels: HashMap<String, Arc<dyn Channel>>,

    /// 配置
    config: ChannelsConfig,

    /// 消息回调
    message_callback: Option<MessageCallback>,
}

impl ChannelManager {
    /// 创建新的通道管理器
    ///
    /// # 参数
    ///
    /// * `config` - 通道配置
    ///
    /// # 返回
    ///
    /// 返回包含所有已创建通道的通道管理器
    pub async fn new(config: ChannelsConfig) -> ChannelResult<Self> {
        info!("初始化通道管理器");

        let mut manager = Self {
            channels: HashMap::new(),
            config,
            message_callback: None,
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

        let channel = DingTalk::new(config).await?;
        let name = channel.name().to_string();

        self.channels.insert(name.clone(), Arc::new(channel));

        info!("钉钉通道添加成功: {}", name);
        Ok(())
    }

    /// 设置消息回调
    ///
    /// 当通道接收到消息时，会调用此回调。
    pub fn set_message_callback<F>(&mut self, callback: F)
    where
        F: Fn(String, crate::messages::InboundMessage) + Send + Sync + 'static,
    {
        self.message_callback = Some(Arc::new(callback));
    }

    /// 启动所有通道
    ///
    /// 并发启动所有已配置的通道。
    pub async fn start_all(&mut self) -> ChannelResult<()> {
        info!("开始启动所有通道");

        let channel_names: Vec<String> = self.channels.keys().cloned().collect();
        let mut failed_channels = Vec::new();

        for name in channel_names {
            if let Some(channel) = self.channels.get(&name) {
                match channel.start().await {
                    Ok(_) => {
                        info!("通道 {} 启动成功", name);
                    }
                    Err(e) => {
                        error!("通道 {} 启动失败: {}", name, e);
                        failed_channels.push(name);
                    }
                }
            }
        }

        if !failed_channels.is_empty() {
            warn!("以下通道启动失败: {:?}", failed_channels);
        }

        let running_count = self.channels.len() - failed_channels.len();
        info!("所有通道启动完成，成功: {}/{}", running_count, self.channels.len());

        Ok(())
    }

    /// 停止所有通道
    ///
    /// 停止所有正在运行的通道并清理资源。
    pub async fn stop_all(&mut self) -> ChannelResult<()> {
        info!("开始停止所有通道");

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
            Err(ChannelError::SendFailed(format!("通道 {} 不存在", channel_name)))
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

    /// 处理接收到的消息
    ///
    /// 此方法供通道实现调用，将接收到的消息传递给管理器。
    pub fn handle_incoming_message(&self, channel_name: String, msg: crate::messages::InboundMessage) {
        if let Some(callback) = &self.message_callback {
            callback(channel_name, msg);
        }
    }
}

#[cfg(test)]
mod tests;
