//! 自动重试 Provider 装饰器
//!
//! 为内部 Provider 添加指数退避重试能力，仅对瞬态错误（超时、限流、服务端错误）进行重试。

use anyhow::Result;
use nanobot_tools::ToolDefinition;
use tracing::warn;

use crate::{Message, MeteredMessage, Options, Provider, ProviderError, strip_images};

/// 默认最大重试次数
const DEFAULT_MAX_RETRIES: u32 = 3;

/// 自动重试 Provider 装饰器
///
/// 包装任意 `Provider` 实现，在 `chat()` 调用失败时对瞬态错误进行指数退避重试。
/// 重试间隔为 `2^attempt` 秒（1s, 2s, 4s）。
///
/// 仅对 [`ProviderError::is_transient()`] 返回 `true` 的错误进行重试，
/// 永久性错误和非 `ProviderError` 类型的错误立即返回。
#[derive(Clone)]
pub struct AutoRetryProvider<P: Provider> {
    inner: P,
    max_retries: u32,
}

impl<P: Provider> AutoRetryProvider<P> {
    /// 创建带默认重试次数（3 次）的自动重试 Provider
    pub fn new(inner: P) -> Self {
        Self { inner, max_retries: DEFAULT_MAX_RETRIES }
    }
}

#[async_trait::async_trait]
impl<P: Provider> Provider for AutoRetryProvider<P> {
    async fn chat(&self, messages: &[Message], options: &Options) -> Result<MeteredMessage> {
        let mut last_err = None;

        for attempt in 0..=self.max_retries {
            match self.inner.chat(messages, options).await {
                Ok(msg) => return Ok(msg),
                Err(e) => {
                    if attempt < self.max_retries
                        && let Some(pe) = e.downcast_ref::<ProviderError>()
                        && pe.is_transient()
                    {
                        // 优先使用 provider 返回的 retry_after，回退到指数退避
                        let delay = pe.retry_after().unwrap_or(std::time::Duration::from_secs(1u64 << attempt));
                        warn!("LLM 调用失败（瞬态错误），{}s 后重试（第 {} 次）: {pe}", delay.as_secs(), attempt + 1);
                        tokio::time::sleep(delay).await;
                        last_err = Some(e);
                        continue;
                    }

                    // 图片拒绝：strip images 后重试一次
                    if let Some(pe) = e.downcast_ref::<ProviderError>()
                        && pe.is_image_unsupported()
                        && let Some(stripped) = strip_images(messages)
                    {
                        warn!("LLM 拒绝图片输入，移除图片后重试");
                        return self.inner.chat(&stripped, options).await;
                    }

                    return Err(e);
                }
            }
        }

        // 重试耗尽，返回最后一次错误
        Err(last_err.expect("重试循环至少执行一次"))
    }

    fn bind_tools(&mut self, tools: Vec<ToolDefinition>) {
        self.inner.bind_tools(tools);
    }
}

#[cfg(test)]
mod tests;
