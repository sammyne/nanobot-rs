use async_trait::async_trait;

/// Callback function for executing tasks
///
/// This callback is invoked when the heartbeat service determines that
/// tasks need to be executed (action="run" from LLM decision).
#[async_trait]
pub trait OnExecuteCallback: Send + Sync {
    /// Execute tasks with the given task summary
    ///
    /// # Arguments
    ///
    /// * `task_summary` - A natural language summary of active tasks to execute
    ///
    /// # Returns
    ///
    /// A result string describing the execution outcome, or an error
    async fn execute(&self, task_summary: &str) -> Result<String, anyhow::Error>;
}

/// Callback function for sending notifications
///
/// This callback is invoked after task execution completes with a non-empty result.
#[async_trait]
pub trait OnNotifyCallback: Send + Sync {
    /// Send a notification with the execution result
    ///
    /// # Arguments
    ///
    /// * `result` - The execution result string to notify about
    ///
    /// # Returns
    ///
    /// Ok(()) if notification succeeds, or an error
    async fn notify(&self, result: &str) -> Result<(), anyhow::Error>;
}
