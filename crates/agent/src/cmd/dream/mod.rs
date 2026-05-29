//! /dream, /dream-log, /dream-restore command implementations

use super::Command;
use crate::InboundMessage;

/// /dream command — triggers Dream memory consolidation.
///
/// The actual Dream::run() call happens in `try_handle_cmd` where the
/// provider and memory store are accessible. This struct receives the
/// pre-computed result string.
pub struct DreamCmd {
    /// Pre-computed result from Dream::run().
    pub result: String,
}

impl Command for DreamCmd {
    async fn run(self, _msg: InboundMessage, _session_key: String) -> Result<String, String> {
        Ok(self.result)
    }
}

/// /dream-log command — shows memory change history from GitStore.
pub struct DreamLogCmd {
    /// Pre-computed git log output.
    pub log_output: String,
}

impl Command for DreamLogCmd {
    async fn run(self, _msg: InboundMessage, _session_key: String) -> Result<String, String> {
        Ok(self.log_output)
    }
}

/// /dream-restore command — reverts memory to a previous commit.
pub struct DreamRestoreCmd {
    /// Pre-computed restore result.
    pub restore_output: String,
}

impl Command for DreamRestoreCmd {
    async fn run(self, _msg: InboundMessage, _session_key: String) -> Result<String, String> {
        Ok(self.restore_output)
    }
}

#[cfg(test)]
mod tests;
