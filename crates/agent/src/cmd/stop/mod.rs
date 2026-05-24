//! Stop command implementation

use std::sync::Arc;

use nanobot_provider::Provider;
use nanobot_subagent::SubagentManager;
use tracing::info;

use super::Command;
use crate::InboundMessage;

/// Stop command structure
///
/// This command cancels all background subagent tasks for the current session.
pub struct StopCmd<P: Provider> {
    subagent_manager: Arc<SubagentManager<P>>,
}

impl<P: Provider> StopCmd<P> {
    /// Create a new StopCmd instance
    pub fn new(subagent_manager: Arc<SubagentManager<P>>) -> Self {
        Self { subagent_manager }
    }
}

impl<P: Provider> Command for StopCmd<P> {
    async fn run(self, _msg: InboundMessage, session_key: String) -> Result<String, String> {
        info!("Processing /stop command: session_key={session_key}");

        let cancelled = self.subagent_manager.cancel_by_session(&session_key).await;

        if cancelled > 0 {
            info!("Cancelled {cancelled} subagent task(s) for session {session_key}");
            Ok(format!("Stopped. Cancelled {cancelled} background task(s)."))
        } else {
            Ok("Stopped.".to_string())
        }
    }
}

#[cfg(test)]
mod tests;
