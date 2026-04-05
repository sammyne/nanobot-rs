//! Progress 模块单元测试

use std::sync::Arc;

use tokio::sync::mpsc;

use super::*;

/// 测试 ChannelProgressTracker 发送消息成功
#[tokio::test]
async fn channel_tracker_sends_message() {
    let (tx, mut rx) = mpsc::channel(10);
    let tracker = ChannelProgressTracker::new(tx, "test_channel".to_string(), "test_chat_id".to_string());

    // 发送进度消息
    tracker.track("测试内容".to_string(), false).await.unwrap();

    // 接收并验证
    let msg = rx.recv().await.expect("应该收到消息");
    assert_eq!(msg.channel, "test_channel");
    assert_eq!(msg.chat_id, "test_chat_id");
    assert_eq!(msg.content, "测试内容");
    assert!(msg.is_progress());
    assert!(!msg.is_tool_hint());
}

/// 测试 ChannelProgressTracker 正确设置 metadata 字段
#[tokio::test]
async fn channel_tracker_sets_metadata() {
    let (tx, mut rx) = mpsc::channel(10);
    let tracker = ChannelProgressTracker::new(tx, "channel".to_string(), "chat".to_string());

    // 发送工具提示
    tracker.track("tool_hint".to_string(), true).await.unwrap();

    let msg = rx.recv().await.unwrap();
    assert!(msg.is_progress());
    assert!(msg.is_tool_hint());
}

/// 测试通道关闭时返回错误
#[tokio::test]
async fn channel_tracker_returns_error_on_closed_channel() {
    let (tx, rx) = mpsc::channel(10);
    // 显式关闭接收端
    drop(rx);

    let tracker = ChannelProgressTracker::new(tx, "channel".to_string(), "chat".to_string());

    // 发送应该失败
    let result = tracker.track("test".to_string(), false).await;
    assert!(result.is_err());
}

/// 测试闭包直接实现 trait
#[tokio::test]
async fn closure_tracker_works() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let callback = move |content: String, is_tool_hint: bool| {
        called_clone.store(true, Ordering::SeqCst);
        assert_eq!(content, "test");
        assert!(is_tool_hint);
    };

    let tracker: Arc<dyn ProgressTracker> = Arc::new(callback);
    tracker.track("test".to_string(), true).await.unwrap();

    assert!(called.load(Ordering::SeqCst));
}

/// 测试闭包参数正确传递
#[tokio::test]
async fn closure_tracker_receives_correct_args() {
    use std::sync::Mutex;

    let received = Arc::new(Mutex::new((String::new(), false)));
    let received_clone = received.clone();

    let callback = move |content: String, is_tool_hint: bool| {
        let mut guard = received_clone.lock().unwrap();
        *guard = (content, is_tool_hint);
    };

    let tracker: Arc<dyn ProgressTracker> = Arc::new(callback);
    tracker.track("hello".to_string(), true).await.unwrap();

    let guard = received.lock().unwrap();
    assert_eq!(guard.0, "hello");
    assert!(guard.1);
}
