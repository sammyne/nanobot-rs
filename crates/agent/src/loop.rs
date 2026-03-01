//! AgentLoop 核心实现
//!
//! AgentLoop 是 nanobot 的核心处理引擎，负责：
//! 1. 接收消息
//! 2. 调用 LLM
//! 3. 返回响应

use anyhow::Result;
use nanobot_config::AgentDefaults;
use nanobot_provider::{Message, Provider};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Agent 循环处理引擎
///
/// 负责管理消息处理和 LLM 调用的完整生命周期。
pub struct AgentLoop {
    /// LLM 提供者实例
    provider: Arc<dyn Provider>,

    /// Agent 配置
    config: AgentDefaults,
}

impl AgentLoop {
    /// 创建新的 AgentLoop 实例
    ///
    /// # Arguments
    /// * `provider` - LLM 提供者实例
    /// * `config` - Agent 配置（必选项）
    ///
    /// # Returns
    /// 返回初始化完成的 AgentLoop 实例
    pub fn new(provider: Arc<dyn Provider>, config: AgentDefaults) -> Self {
        info!(
            "初始化 AgentLoop: model={}, max_tool_iterations={}",
            config.model, config.max_tool_iterations
        );

        Self { provider, config }
    }

    /// 调用 LLM 并返回响应
    ///
    /// # Arguments
    /// * `messages` - 消息列表
    ///
    /// # Returns
    /// 返回 LLM 的响应内容
    async fn call_llm(&self, messages: &[Message]) -> Result<String> {
        debug!("调用 LLM: 消息数量={}", messages.len());

        let response = self.provider.chat(messages).await?;

        info!("收到 LLM 响应, 长度: {} 字符", response.len());

        Ok(response)
    }

    /// 运行 Agent 迭代循环
    ///
    /// 参考 Python 版 `_run_agent_loop` 函数实现。
    /// 由于当前 Provider trait 不支持工具调用，
    /// 本实现简化为单次 LLM 调用。
    ///
    /// # Arguments
    /// * `initial_messages` - 初始消息列表
    ///
    /// # Returns
    /// 返回最终响应内容
    async fn re_act(&self, initial_messages: Vec<Message>) -> Result<String> {
        let messages = initial_messages;
        let mut iteration = 0;
        let mut final_content = None;

        // 迭代循环（参考 Python 版 _run_agent_loop）
        while iteration < self.config.max_tool_iterations {
            iteration += 1;

            debug!("迭代循环: 第 {} 次", iteration);

            // 调用 LLM
            let response = self.call_llm(&messages).await?;

            // 当前版本不支持工具调用，直接返回响应
            // 当工具系统实现后，这里需要检查响应是否包含工具调用
            final_content = Some(response);
            break;
        }

        // 达到最大迭代次数的处理
        if final_content.is_none() && iteration >= self.config.max_tool_iterations {
            warn!("达到最大迭代次数限制: {}", self.config.max_tool_iterations);
            final_content = Some(format!(
                "I reached the maximum number of tool call iterations ({}) without completing the task. You can try breaking the task into smaller steps.",
                self.config.max_tool_iterations
            ));
        }

        final_content.ok_or_else(|| anyhow::anyhow!("未能获取响应内容"))
    }

    /// 直接处理消息
    ///
    /// 参考 Python 版 `process_direct` 函数实现。
    /// 用于 CLI 或 cron 等直接调用场景。
    ///
    /// # Arguments
    /// * `content` - 用户消息内容
    ///
    /// # Returns
    /// 返回响应内容
    pub async fn process_direct(&self, content: &str) -> Result<String> {
        info!("直接处理消息: {}", content);

        // 构建消息列表
        let messages = vec![Message::user(content)];

        // 运行迭代循环
        self.re_act(messages).await
    }

    /// 获取配置
    pub fn config(&self) -> &AgentDefaults {
        &self.config
    }
}
