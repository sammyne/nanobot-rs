//! Hook 模块单元测试

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::mpsc;

use super::*;

/// 默认 Hook 所有方法为空操作（不 panic）
#[tokio::test]
async fn noop_hook_does_not_panic() {
    let hook = NoopHook;
    let ctx = HookCtx { content: "hello", tool_calls: &[], usage: None };

    hook.before_iteration(&ctx).await.unwrap();
    hook.before_execute_tools(&ctx).await.unwrap();
    hook.after_iteration(&ctx).await.unwrap();
    assert_eq!(hook.finalize_content(&ctx, Some("test".into())).await, Some("test".into()));
    assert_eq!(hook.finalize_content(&ctx, None).await, None);
}

/// 用于测试的计数 hook
struct CountingHook {
    before_iter: AtomicUsize,
    before_tools: AtomicUsize,
    after_iter: AtomicUsize,
}

impl CountingHook {
    fn new() -> Self {
        Self { before_iter: AtomicUsize::new(0), before_tools: AtomicUsize::new(0), after_iter: AtomicUsize::new(0) }
    }
}

#[async_trait]
impl Hook for CountingHook {
    async fn before_iteration(&self, _ctx: &HookCtx<'_>) -> Result<()> {
        self.before_iter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn before_execute_tools(&self, _ctx: &HookCtx<'_>) -> Result<()> {
        self.before_tools.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn after_iteration(&self, _ctx: &HookCtx<'_>) -> Result<()> {
        self.after_iter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// CompositeHook 按顺序调用所有 hook
#[tokio::test]
async fn composite_calls_all_hooks() {
    let h1 = Arc::new(CountingHook::new());
    let h2 = Arc::new(CountingHook::new());
    let composite = CompositeHook::new(vec![h1.clone() as Arc<dyn Hook>, h2.clone()]);

    let ctx = HookCtx { content: "", tool_calls: &[], usage: None };

    composite.before_iteration(&ctx).await.unwrap();
    composite.before_execute_tools(&ctx).await.unwrap();
    composite.after_iteration(&ctx).await.unwrap();

    assert_eq!(h1.before_iter.load(Ordering::SeqCst), 1);
    assert_eq!(h2.before_iter.load(Ordering::SeqCst), 1);
    assert_eq!(h1.before_tools.load(Ordering::SeqCst), 1);
    assert_eq!(h2.before_tools.load(Ordering::SeqCst), 1);
    assert_eq!(h1.after_iter.load(Ordering::SeqCst), 1);
    assert_eq!(h2.after_iter.load(Ordering::SeqCst), 1);
}

/// 用于测试错误隔离的 hook
struct FailingHook;

#[async_trait]
impl Hook for FailingHook {
    async fn before_execute_tools(&self, _ctx: &HookCtx<'_>) -> Result<()> {
        anyhow::bail!("intentional failure")
    }
}

/// CompositeHook 中一个 hook 失败不影响其他 hook
#[tokio::test]
async fn composite_isolates_errors() {
    let counter = Arc::new(CountingHook::new());
    let composite = CompositeHook::new(vec![Arc::new(FailingHook) as Arc<dyn Hook>, counter.clone()]);

    let ctx = HookCtx { content: "", tool_calls: &[], usage: None };

    // CompositeHook 本身不返回错误
    composite.before_execute_tools(&ctx).await.unwrap();

    // 第二个 hook 仍然被调用
    assert_eq!(counter.before_tools.load(Ordering::SeqCst), 1);
}

/// 用于测试 finalize_content 管道的 hook
struct AppendHook {
    suffix: &'static str,
}

#[async_trait]
impl Hook for AppendHook {
    async fn finalize_content(&self, _ctx: &HookCtx<'_>, content: Option<String>) -> Option<String> {
        content.map(|c| format!("{c}{}", self.suffix))
    }
}

/// finalize_content 管道式传递（hook A 输出 → hook B 输入）
#[tokio::test]
async fn composite_finalize_content_pipeline() {
    let composite = CompositeHook::new(vec![
        Arc::new(AppendHook { suffix: "_A" }) as Arc<dyn Hook>,
        Arc::new(AppendHook { suffix: "_B" }),
    ]);

    let ctx = HookCtx { content: "", tool_calls: &[], usage: None };
    let result = composite.finalize_content(&ctx, Some("start".into())).await;
    assert_eq!(result, Some("start_A_B".into()));
}

/// finalize_content 管道传递 None
#[tokio::test]
async fn composite_finalize_content_none_passthrough() {
    let composite = CompositeHook::new(vec![Arc::new(AppendHook { suffix: "_A" }) as Arc<dyn Hook>]);

    let ctx = HookCtx { content: "", tool_calls: &[], usage: None };
    let result = composite.finalize_content(&ctx, None).await;
    assert_eq!(result, None);
}

/// LoopHook 在 before_execute_tools 中通过 tx 发送正确的 OutboundMessage
#[tokio::test]
async fn loop_hook_sends_progress_messages() {
    let (tx, mut rx) = mpsc::channel(10);
    let hook = LoopHook::new(tx, "test_ch".into(), "test_chat".into());

    let tool_calls = vec![nanobot_provider::ToolCall::new("id1", "read_file", serde_json::json!({"path": "/tmp"}))];
    let ctx = HookCtx { content: "thinking about it", tool_calls: &tool_calls, usage: None };

    hook.before_execute_tools(&ctx).await.unwrap();

    // 第一条：思考内容
    let msg1 = rx.recv().await.unwrap();
    assert!(msg1.is_progress());
    assert!(!msg1.is_tool_hint());
    assert_eq!(msg1.content, "thinking about it");

    // 第二条：工具提示
    let msg2 = rx.recv().await.unwrap();
    assert!(msg2.is_progress());
    assert!(msg2.is_tool_hint());
    assert!(msg2.content.contains("read_file"));
}

/// LoopHook 的 finalize_content 剥离 <think> 标签
#[tokio::test]
async fn loop_hook_strips_think_tags() {
    let (tx, _rx) = mpsc::channel(10);
    let hook = LoopHook::new(tx, "ch".into(), "chat".into());

    let ctx = HookCtx { content: "", tool_calls: &[], usage: None };
    let result = hook.finalize_content(&ctx, Some("<think>internal</think>visible".into())).await;
    assert_eq!(result, Some("visible".into()));
}

/// LoopHook 不发送空思考内容
#[tokio::test]
async fn loop_hook_skips_empty_thinking() {
    let (tx, mut rx) = mpsc::channel(10);
    let hook = LoopHook::new(tx, "ch".into(), "chat".into());

    let tool_calls = vec![nanobot_provider::ToolCall::new("id1", "exec", serde_json::json!({"cmd": "ls"}))];
    let ctx = HookCtx { content: "", tool_calls: &tool_calls, usage: None };

    hook.before_execute_tools(&ctx).await.unwrap();

    // 只有工具提示，没有思考内容
    let msg = rx.recv().await.unwrap();
    assert!(msg.is_tool_hint());

    // 没有更多消息
    assert!(rx.try_recv().is_err());
}
