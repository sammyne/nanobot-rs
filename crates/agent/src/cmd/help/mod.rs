//! Help command implementation

use super::Command;
use crate::InboundMessage;

/// Help command structure
///
/// This command displays available commands to the user.
pub struct HelpCmd;

impl Command for HelpCmd {
    async fn run(self, _msg: InboundMessage, _session_key: String) -> Result<String, String> {
        // Return help information (consistent with original implementation)
        Ok("🐈 nanobot commands:\n/new — Start a new conversation\n/stop — Stop current processing and cancel background tasks\n/restart — Restart the agent process\n/help — Show available commands".to_owned())
    }
}

#[cfg(test)]
mod tests;
