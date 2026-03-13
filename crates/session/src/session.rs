//! Session data model and core interfaces.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use nanobot_provider::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A conversation session.
///
/// Stores messages in JSONL format for easy reading and persistence.
///
/// Important: Messages are append-only for LLM cache efficiency.
/// The consolidation process writes summaries to MEMORY.md/HISTORY.md
/// but does NOT modify the messages list or get_history() output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session key (usually channel:chat_id)
    pub key: String,
    /// List of messages in the session
    pub messages: Vec<Message>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
    /// Number of messages already consolidated to files
    pub last_consolidated: usize,
}

impl Session {
    /// Create a new session with the given key.
    pub fn new(key: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            key: key.into(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            last_consolidated: 0,
        }
    }

    /// Add a message to the session.
    ///
    /// The message is appended to the messages list and the updated_at timestamp is updated.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Append unconsolidated messages to the provided buffer for LLM input.
    ///
    /// This method implements:
    /// 1. Returns only messages after `last_consolidated` index
    /// 2. Limits output to `max_messages` most recent messages
    /// 3. Drops leading non-user messages to avoid orphaned tool_result blocks
    ///
    /// # Arguments
    /// * `max_messages` - Maximum number of messages to append
    /// * `buf` - Buffer to append messages to
    ///
    /// # Returns
    /// The number of messages appended
    pub fn get_history(&self, max_messages: usize, buf: &mut Vec<Message>) -> usize {
        let total = self.messages.len();
        let start_unconsolidated = self.last_consolidated;

        // Calculate the range for unconsolidated messages
        if start_unconsolidated >= total {
            return 0;
        }

        // Calculate start index considering max_messages limit
        let available = total - start_unconsolidated;
        let start = if available > max_messages { total - max_messages } else { start_unconsolidated };

        // Find first user message to avoid orphaned tool_result blocks
        let first_user_idx = self.messages[start..].iter().position(|m| matches!(m, Message::User { .. }));

        let final_start = match first_user_idx {
            Some(idx) => start + idx,
            None => start,
        };

        let count = total - final_start;
        buf.extend(self.messages[final_start..].iter().cloned());
        count
    }

    /// Clear all messages and reset session to initial state.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.last_consolidated = 0;
        self.updated_at = Utc::now();
    }

    /// Touch the session to update the updated_at timestamp.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Metadata line in JSONL file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Type marker, always "metadata"
    #[serde(rename = "_type")]
    pub type_marker: String,
    /// Session key
    pub key: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
    /// Number of messages already consolidated
    pub last_consolidated: usize,
}

impl From<&Session> for SessionMetadata {
    fn from(session: &Session) -> Self {
        Self {
            type_marker: "metadata".to_string(),
            key: session.key.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            metadata: session.metadata.clone(),
            last_consolidated: session.last_consolidated,
        }
    }
}

/// Session info for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session key
    pub key: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// File path
    pub path: String,
}
