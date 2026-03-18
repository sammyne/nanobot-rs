//! New command implementation

use std::collections::HashSet;
use std::sync::Arc;

use nanobot_provider::Provider;
use nanobot_session::SessionManager;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use super::Command;
use crate::InboundMessage;

/// New command structure
///
/// This command starts a new conversation by archiving memory and clearing the session.
pub struct NewCmd<P: Provider> {
    sessions: Arc<SessionManager>,
    memory: Arc<nanobot_memory::MemoryStore>,
    provider: P,
    consolidating: Arc<Mutex<HashSet<String>>>,
}

impl<P: Provider> NewCmd<P> {
    /// Create a new NewCmd instance with necessary dependencies
    pub fn new(
        sessions: Arc<SessionManager>,
        memory: Arc<nanobot_memory::MemoryStore>,
        provider: P,
        consolidating: Arc<Mutex<HashSet<String>>>,
    ) -> Self {
        Self { sessions, memory, provider, consolidating }
    }
}

impl<P: Provider> Command for NewCmd<P> {
    async fn run(self, _msg: InboundMessage, session_key: String) -> Result<String, String> {
        // Spawn async task to handle /new command
        let handle = tokio::spawn(handle_new_cmd_logic(
            self.sessions,
            self.memory,
            self.provider.clone(),
            self.consolidating,
            session_key,
        ));

        match handle.await {
            Ok(result) => result,
            Err(e) => {
                error!("Spawn task failed for /new command: error={}", e);
                Err(format!("Internal error: {e}"))
            }
        }
    }
}

/// Internal logic for handling the /new command
///
/// This function performs the actual work of:
/// 1. Checking consolidation status (concurrency control)
/// 2. Archiving all unconsolidated messages to long-term memory (archive_all=true)
/// 3. Clearing the current session
/// 4. Saving the updated session
///
/// # Arguments
/// * `sessions` - Session manager
/// * `memory` - Memory store
/// * `provider` - LLM provider
/// * `consolidating` - Consolidation status set
/// * `session_key` - Session identifier
///
/// # Returns
/// - `Ok(String)`: Success message "New session started."
/// - `Err(String)`: Error message with failure details
async fn handle_new_cmd_logic<P: Provider>(
    sessions: Arc<SessionManager>,
    memory: Arc<nanobot_memory::MemoryStore>,
    provider: P,
    consolidating: Arc<Mutex<HashSet<String>>>,
    session_key: String,
) -> Result<String, String> {
    info!("Starting /new command: session_key={}", session_key);

    // Get session (load from cache/disk or create new)
    let mut session = sessions.get_or_create(&session_key);

    // Check consolidation status
    let session_key_clone = session.key.clone();
    if !consolidating.lock().await.insert(session_key_clone.clone()) {
        warn!("Session already being consolidated: {}", session_key);
        return Err("Session is already being consolidated. Please try again later.".to_string());
    }

    info!("Starting memory consolidation for /new command: session_key={}", session_key);

    // Perform consolidation with archive_all=true
    let result = memory
        .consolidate(
            &session.messages,
            session.last_consolidated,
            provider,
            true, // archive_all=true
            0,    // memory_window not used when archive_all=true
        )
        .await;

    // Clean up consolidation status
    consolidating.lock().await.remove(&session_key_clone);

    // Handle result
    match result {
        Ok(_) => {
            info!("Memory consolidation successful for /new command: session_key={}", session_key);

            // Clear session
            session.clear();

            // Save updated session
            if let Err(e) = sessions.save(&session) {
                error!("Failed to save session after /new command: {}", e);
                return Err(format!("Failed to save session: {e}"));
            }

            // Invalidate session cache
            sessions.invalidate(&session_key);

            Ok("New session started.".to_string())
        }
        Err(e) => {
            error!("Memory consolidation failed for /new command: session_key={}, error={}", session_key, e);
            Err(format!("Memory archival failed, session not cleared. Please try again: {e}"))
        }
    }
}

#[cfg(test)]
mod tests;
