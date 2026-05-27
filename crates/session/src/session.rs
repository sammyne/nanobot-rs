//! Session data model and core interfaces.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use nanobot_provider::{ContentPart, Message, UserContent};
use nanobot_utils::strings::truncate;
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

    /// Maximum characters for tool result before truncation.
    const TOOL_RESULT_MAX_CHARS: usize = 16_000;

    /// Save messages from a conversation turn (incremental append).
    ///
    /// Truncates tool results exceeding `TOOL_RESULT_MAX_CHARS` to prevent
    /// excessively long messages from being persisted.
    ///
    /// # Arguments
    /// * `messages` - All messages from the conversation turn
    /// * `skip` - Number of messages to skip (already persisted in history)
    pub fn save_turn(&mut self, messages: &[Message], skip: usize) {
        for msg in messages.iter().skip(skip) {
            let msg_to_save = match msg {
                Message::Tool { content, tool_call_id } => {
                    let truncated = truncate(content, Self::TOOL_RESULT_MAX_CHARS)
                        .map(|truncated_content| format!("{truncated_content}\n... (truncated)"))
                        .unwrap_or_else(|| content.clone());
                    Message::Tool { content: truncated, tool_call_id: tool_call_id.clone() }
                }
                Message::User { content } => Message::User { content: strip_runtime_context(&strip_images(content)) },
                other => other.clone(),
            };
            self.add_message(msg_to_save);
        }
        self.touch();
    }
}

/// Strip image data from `UserContent` to prevent base64 bloat in session JSONL.
///
/// Replaces `ContentPart::Image` with `ContentPart::Text { text: "[image]" }`.
/// If all parts become text after stripping, merges into `UserContent::Text`.
fn strip_images(content: &UserContent) -> UserContent {
    match content {
        UserContent::Text(_) => content.clone(),
        UserContent::Parts(parts) => {
            let stripped: Vec<ContentPart> = parts
                .iter()
                .map(|part| match part {
                    ContentPart::Image { .. } => ContentPart::Text { text: "[image]".to_string() },
                    other => other.clone(),
                })
                .collect();

            // If all parts are text, merge into a single Text
            let all_text = stripped.iter().all(|p| matches!(p, ContentPart::Text { .. }));
            if all_text {
                let merged: Vec<&str> = stripped
                    .iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect();
                UserContent::Text(merged.join("\n"))
            } else {
                UserContent::Parts(stripped)
            }
        }
    }
}

// 须与 crates/context/src/builder/mod.rs 中 inject_runtime_context 的格式保持一致
const RUNTIME_CONTEXT_MARKER: &str = "\n\n[Runtime Context]\n";

/// Strip runtime context block from `UserContent` to prevent accumulation in session history.
///
/// The context builder appends `\n\n[Runtime Context]\n...` to user messages for each LLM request.
/// This data is per-request and should not be persisted.
fn strip_runtime_context(content: &UserContent) -> UserContent {
    match content {
        UserContent::Text(text) => match text.find(RUNTIME_CONTEXT_MARKER) {
            Some(pos) => UserContent::Text(text[..pos].to_string()),
            None => content.clone(),
        },
        UserContent::Parts(parts) => {
            // inject_runtime_context appends a Text part starting with "\n\n[Runtime Context]\n"
            if let Some(ContentPart::Text { text }) = parts.last()
                && text.contains("[Runtime Context]\n")
            {
                let mut stripped = parts[..parts.len() - 1].to_vec();
                // If the part has user text before the marker, preserve it
                if let Some(pos) = text.find(RUNTIME_CONTEXT_MARKER)
                    && pos > 0
                {
                    stripped.push(ContentPart::Text { text: text[..pos].to_string() });
                }
                return if stripped.is_empty() {
                    UserContent::Text(String::new())
                } else {
                    UserContent::Parts(stripped)
                };
            }
            content.clone()
        }
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
