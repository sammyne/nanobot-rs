use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::Result;
use nanobot_tools::ToolDefinition;

use super::*;
use crate::{ContentPart, Message, MeteredMessage, Options, Provider, ProviderError, UserContent};

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
    async fn chat(&self, _messages: &[Message], _options: &Options) -> Result<MeteredMessage> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        if n < self.fail_count { Err((self.error_factory)(n)) } else { Ok(Message::assistant("ok").into()) }
    }

    fn bind_tools(&mut self, _tools: Vec<ToolDefinition>) {}
}

#[tokio::test]
async fn transient_error_retries_then_succeeds() {
    tokio::time::pause();

    let mock = MockProvider::new(2, |_| {
        ProviderError::RateLimit { message: "rate limited".to_string(), retry_after: None }.into()
    });
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

/// 构造包含图片的消息列表
fn messages_with_image() -> Vec<Message> {
    vec![Message::User {
        content: UserContent::Parts(vec![
            ContentPart::Text { text: "describe this".to_string() },
            ContentPart::Image { media_type: "image/png".to_string(), data: "abc".to_string() },
        ]),
    }]
}

/// Mock Provider：首次返回图片拒绝错误，第二次根据消息内容决定
#[derive(Clone)]
struct ImageRejectMock {
    call_count: Arc<AtomicU32>,
    second_call_succeeds: bool,
}

#[async_trait::async_trait]
impl Provider for ImageRejectMock {
    async fn chat(&self, messages: &[Message], _options: &Options) -> Result<MeteredMessage> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        if n == 0 {
            return Err(ProviderError::Api("image_url is only supported by vision models".to_string()).into());
        }
        // 验证重试时图片已被 strip
        if let Some(Message::User { content: UserContent::Parts(parts) }) = messages.first() {
            for p in parts {
                if matches!(p, ContentPart::Image { .. }) {
                    panic!("retry should not contain Image parts");
                }
            }
        }
        if self.second_call_succeeds {
            Ok(Message::assistant("described without image").into())
        } else {
            Err(ProviderError::Api("still failing".to_string()).into())
        }
    }

    fn bind_tools(&mut self, _tools: Vec<ToolDefinition>) {}
}

#[tokio::test]
async fn image_unsupported_retries_without_images() {
    tokio::time::pause();

    let mock = ImageRejectMock { call_count: Arc::new(AtomicU32::new(0)), second_call_succeeds: true };
    let call_count = mock.call_count.clone();
    let provider = AutoRetryProvider::new(mock);

    let result = provider.chat(&messages_with_image(), &Options::default()).await;
    assert!(result.is_ok(), "should succeed after stripping images: {result:?}");
    assert_eq!(call_count.load(Ordering::SeqCst), 2, "should call inner 2 times");
}

#[tokio::test]
async fn image_unsupported_no_images_to_strip() {
    tokio::time::pause();

    let mock =
        MockProvider::new(1, |_| ProviderError::Api("image_url is only supported by vision models".to_string()).into());
    let provider = AutoRetryProvider::new(mock.clone());

    // 纯文本消息，无图片可 strip
    let result = provider.chat(&[Message::user("hello")], &Options::default()).await;
    assert!(result.is_err(), "should fail when no images to strip");
    assert_eq!(mock.calls(), 1, "should call inner only once");
}

#[tokio::test]
async fn image_unsupported_retry_also_fails() {
    tokio::time::pause();

    let mock = ImageRejectMock { call_count: Arc::new(AtomicU32::new(0)), second_call_succeeds: false };
    let call_count = mock.call_count.clone();
    let provider = AutoRetryProvider::new(mock);

    let result = provider.chat(&messages_with_image(), &Options::default()).await;
    assert!(result.is_err(), "should fail when retry also fails");
    assert_eq!(call_count.load(Ordering::SeqCst), 2, "should call inner 2 times");
}

#[tokio::test]
async fn non_image_error_no_image_retry() {
    tokio::time::pause();

    let mock = MockProvider::new(1, |_| ProviderError::Api("401 unauthorized".to_string()).into());
    let provider = AutoRetryProvider::new(mock.clone());

    let result = provider.chat(&messages_with_image(), &Options::default()).await;
    assert!(result.is_err(), "should fail without image retry");
    assert_eq!(mock.calls(), 1, "should call inner only once");
}

#[tokio::test]
async fn retry_uses_provider_retry_after_when_available() {
    tokio::time::pause();

    let mock = MockProvider::new(1, |_| {
        ProviderError::RateLimit {
            message: "rate limited".to_string(),
            retry_after: Some(std::time::Duration::from_secs(20)),
        }
        .into()
    });
    let provider = AutoRetryProvider::new(mock.clone());

    let start = tokio::time::Instant::now();
    let result = provider.chat(&[], &Options::default()).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // 第一次重试应等待 20s（来自 retry_after），而非 1s（指数退避）
    assert!(elapsed >= std::time::Duration::from_secs(20), "should wait at least 20s, got {elapsed:?}");
}

#[tokio::test]
async fn retry_falls_back_to_exponential_without_retry_after() {
    tokio::time::pause();

    let mock = MockProvider::new(1, |_| {
        ProviderError::RateLimit { message: "rate limited".to_string(), retry_after: None }.into()
    });
    let provider = AutoRetryProvider::new(mock.clone());

    let start = tokio::time::Instant::now();
    let result = provider.chat(&[], &Options::default()).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // 无 retry_after 时，第一次重试等待 1s（2^0）
    assert!(elapsed >= std::time::Duration::from_secs(1), "should wait at least 1s, got {elapsed:?}");
    assert!(elapsed < std::time::Duration::from_secs(5), "should not wait too long, got {elapsed:?}");
}
