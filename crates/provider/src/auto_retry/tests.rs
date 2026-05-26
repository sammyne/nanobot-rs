use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::Result;
use nanobot_tools::ToolDefinition;

use super::*;
use crate::{Message, Options, Provider, ProviderError};

/// 可配置的 Mock Provider
///
/// 前 `fail_count` 次调用返回指定错误，之后返回成功。
#[derive(Clone)]
struct MockProvider {
    call_count: Arc<AtomicU32>,
    fail_count: u32,
    error_factory: fn(u32) -> anyhow::Error,
}

impl MockProvider {
    fn new(fail_count: u32, error_factory: fn(u32) -> anyhow::Error) -> Self {
        Self { call_count: Arc::new(AtomicU32::new(0)), fail_count, error_factory }
    }

    fn calls(&self) -> u32 {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn chat(&self, _messages: &[Message], _options: &Options) -> Result<Message> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        if n < self.fail_count { Err((self.error_factory)(n)) } else { Ok(Message::assistant("ok")) }
    }

    fn bind_tools(&mut self, _tools: Vec<ToolDefinition>) {}
}

#[tokio::test]
async fn transient_error_retries_then_succeeds() {
    tokio::time::pause();

    let mock = MockProvider::new(2, |_| ProviderError::RateLimit("rate limited".to_string()).into());
    let provider = AutoRetryProvider::new(mock.clone());

    let result = provider.chat(&[], &Options::default()).await;
    assert!(result.is_ok(), "should succeed after retries: {result:?}");
    assert_eq!(mock.calls(), 3, "should call inner 3 times (1 initial + 2 retries)");
}

#[tokio::test]
async fn permanent_error_no_retry() {
    tokio::time::pause();

    let mock = MockProvider::new(1, |_| ProviderError::Api("bad request".to_string()).into());
    let provider = AutoRetryProvider::new(mock.clone());

    let result = provider.chat(&[], &Options::default()).await;
    assert!(result.is_err(), "should fail immediately");
    assert_eq!(mock.calls(), 1, "should call inner only once");
}

#[tokio::test]
async fn retries_exhausted_returns_last_error() {
    tokio::time::pause();

    let mock = MockProvider::new(10, |_| ProviderError::ServerError("HTTP 500".to_string()).into());
    let provider = AutoRetryProvider::new(mock.clone());

    let result = provider.chat(&[], &Options::default()).await;
    assert!(result.is_err(), "should fail after retries exhausted");
    assert_eq!(mock.calls(), 4, "should call inner 4 times (1 initial + 3 retries)");
}

#[tokio::test]
async fn non_provider_error_no_retry() {
    tokio::time::pause();

    let mock = MockProvider::new(1, |_| anyhow::anyhow!("unknown error"));
    let provider = AutoRetryProvider::new(mock.clone());

    let result = provider.chat(&[], &Options::default()).await;
    assert!(result.is_err(), "should fail immediately");
    assert_eq!(mock.calls(), 1, "should call inner only once for non-ProviderError");
}
