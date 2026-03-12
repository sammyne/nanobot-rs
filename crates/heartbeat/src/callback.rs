use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Callback function type for executing tasks
///
/// This callback is invoked when the heartbeat service determines that
/// tasks need to be executed (action="run" from LLM decision).
pub type OnExecuteCallback =
    Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<String, anyhow::Error>> + Send>> + Send + Sync>;

/// Callback function type for sending notifications
///
/// This callback is invoked after task execution completes with a non-empty result.
pub type OnNotifyCallback =
    Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>> + Send + Sync>;
