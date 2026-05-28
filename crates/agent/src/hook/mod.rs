//! Agent 生命周期钩子模块
//!
//! 提供 Agent 循环各阶段的扩展点，替代原有的 `ProgressTracker`。
//!
//! # 钩子方法
//!
//! - [`Hook::before_iteration`] -- 每次 LLM 调用前
//! - [`Hook::before_execute_tools`] -- 工具执行前（发送思考内容和工具提示）
//! - [`Hook::after_iteration`] -- 每轮迭代后（用于 usage 追踪等）
//! - [`Hook::finalize_content`] -- 最终内容变换（用于剥离 `<think>` 标签等）

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use nanobot_provider::{TokenUsage, ToolCall};
use tokio::sync::mpsc;
use tracing::error;

use crate::OutboundMessage;

/// Agent 生命周期钩子上下文
///
/// 携带当前迭代的上下文信息，在 `re_act` 循环中每次迭代构造，
/// 借用局部变量，零拷贝开销。
pub struct HookCtx<'a> {
    /// 当前迭代的 LLM 输出内容
    pub content: &'a str,
    /// 当前迭代的工具调用列表
    pub tool_calls: &'a [ToolCall],
    /// 当前迭代的 token 用量
    pub usage: Option<&'a TokenUsage>,
}

/// Agent 生命周期钩子
///
/// 定义 Agent 循环各阶段的扩展点。所有方法均有默认空实现，
/// 实现者可按需覆盖。
#[async_trait]
pub trait Hook: Send + Sync {
    /// 每次 LLM 调用前
    async fn before_iteration(&self, _ctx: &HookCtx<'_>) -> Result<()> {
        Ok(())
    }

    /// 工具执行前
    async fn before_execute_tools(&self, _ctx: &HookCtx<'_>) -> Result<()> {
        Ok(())
    }

    /// 每轮迭代后
    async fn after_iteration(&self, _ctx: &HookCtx<'_>) -> Result<()> {
        Ok(())
    }

    /// 最终内容变换
    ///
    /// 在返回最终结果前对内容进行变换，例如剥离 `<think>` 标签。
    /// 返回 `None` 表示内容为空。
    async fn finalize_content(&self, _ctx: &HookCtx<'_>, content: Option<String>) -> Option<String> {
        content
    }
}

/// 空操作钩子
///
/// 所有方法使用默认空实现，当不需要 hook 时作为占位符使用。
pub struct NoopHook;

#[async_trait]
impl Hook for NoopHook {}

/// 组合钩子
///
/// 将多个 hook 组合调用。异步方法逐个调用并记录失败；
/// `finalize_content` 串行管道传递（hook A 输出作为 hook B 输入）。
pub struct CompositeHook {
    hooks: Vec<Arc<dyn Hook>>,
}

impl CompositeHook {
    /// 创建组合钩子
    pub fn new(hooks: Vec<Arc<dyn Hook>>) -> Self {
        Self { hooks }
    }
}

#[async_trait]
impl Hook for CompositeHook {
    async fn before_iteration(&self, ctx: &HookCtx<'_>) -> Result<()> {
        for hook in &self.hooks {
            if let Err(e) = hook.before_iteration(ctx).await {
                error!("hook before_iteration failed: {e}");
            }
        }
        Ok(())
    }

    async fn before_execute_tools(&self, ctx: &HookCtx<'_>) -> Result<()> {
        for hook in &self.hooks {
            if let Err(e) = hook.before_execute_tools(ctx).await {
                error!("hook before_execute_tools failed: {e}");
            }
        }
        Ok(())
    }

    async fn after_iteration(&self, ctx: &HookCtx<'_>) -> Result<()> {
        for hook in &self.hooks {
            if let Err(e) = hook.after_iteration(ctx).await {
                error!("hook after_iteration failed: {e}");
            }
        }
        Ok(())
    }

    async fn finalize_content(&self, ctx: &HookCtx<'_>, content: Option<String>) -> Option<String> {
        let mut content = content;
        for hook in &self.hooks {
            content = hook.finalize_content(ctx, content).await;
        }
        content
    }
}

/// 交互式循环钩子
///
/// 直接持有 `mpsc::Sender<OutboundMessage>` 和通道/聊天标识，
/// 在 `before_execute_tools` 中发送思考内容和工具提示，
/// 在 `finalize_content` 中剥离 `<think>` 标签。
///
/// 行为与原 `ChannelProgressTracker` 完全一致。
pub struct LoopHook {
    /// 出站消息发送端
    tx: mpsc::Sender<OutboundMessage>,
    /// 通道名称
    channel: String,
    /// 聊天标识
    chat_id: String,
}

impl LoopHook {
    /// 创建新的 LoopHook
    pub fn new(tx: mpsc::Sender<OutboundMessage>, channel: String, chat_id: String) -> Self {
        Self { tx, channel, chat_id }
    }
}

#[async_trait]
impl Hook for LoopHook {
    async fn before_execute_tools(&self, ctx: &HookCtx<'_>) -> Result<()> {
        // 1. 发送清理后的思考内容（如果有）
        let cleaned = crate::r#loop::strip_think(ctx.content);
        if !cleaned.is_empty() {
            let msg = OutboundMessage::progress(&self.channel, &self.chat_id, cleaned, false);
            if let Err(e) = self.tx.send(msg).await {
                error!("发送思考进度失败: {e}");
            }
        }

        // 2. 发送工具提示
        let hint = crate::r#loop::format_tool_hint(ctx.tool_calls);
        let msg = OutboundMessage::progress(&self.channel, &self.chat_id, hint, true);
        if let Err(e) = self.tx.send(msg).await {
            error!("发送工具提示失败: {e}");
        }

        Ok(())
    }

    async fn finalize_content(&self, _ctx: &HookCtx<'_>, content: Option<String>) -> Option<String> {
        content.map(|c| crate::r#loop::strip_think(&c))
    }
}

#[cfg(test)]
mod tests;
