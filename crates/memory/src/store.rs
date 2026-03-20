//! Memory store implementation for persistent agent memory.

use std::path::PathBuf;

use anyhow::Result;
use nanobot_provider::{Message, Provider};
use nanobot_tools::ToolDefinition;
use serde_json::json;
use tracing::{info, warn};

use crate::MemoryError;

/// Save memory tool name
const SAVE_MEMORY_TOOL: &str = "save_memory";

/// Create the save_memory tool definition for LLM function calling.
fn create_save_memory_tool() -> ToolDefinition {
    ToolDefinition {
        name: SAVE_MEMORY_TOOL.to_string(),
        description: "Save the memory consolidation result to persistent storage.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "history_entry": {
                    "type": "string",
                    "description": "A paragraph (2-5 sentences) summarizing key events/decisions/topics. Start with [YYYY-MM-DD HH:MM]. Include detail useful for grep search."
                },
                "memory_update": {
                    "type": "string",
                    "description": "Full updated long-term memory as markdown. Include all existing facts plus new ones. Return unchanged if nothing new."
                }
            },
            "required": ["history_entry", "memory_update"]
        }),
    }
}

/// Two-layer memory store: MEMORY.md (long-term facts) + HISTORY.md (grep-searchable log).
pub struct MemoryStore {
    /// Path to MEMORY.md file
    memory_file: PathBuf,
    /// Path to HISTORY.md file
    history_file: PathBuf,
}

impl MemoryStore {
    /// Create a new MemoryStore instance.
    ///
    /// Creates the memory/ directory under the workspace if it doesn't exist.
    pub fn new(workspace: PathBuf) -> Result<Self, MemoryError> {
        let memory_dir = workspace.join("memory");
        std::fs::create_dir_all(&memory_dir)?;

        let memory_file = memory_dir.join("MEMORY.md");
        let history_file = memory_dir.join("HISTORY.md");

        Ok(Self { memory_file, history_file })
    }

    /// Read long-term memory content from MEMORY.md.
    ///
    /// Returns an empty string if the file doesn't exist.
    pub fn read_long_term(&self) -> Result<String, MemoryError> {
        if self.memory_file.exists() { Ok(std::fs::read_to_string(&self.memory_file)?) } else { Ok(String::new()) }
    }

    /// Write content to MEMORY.md file.
    pub fn write_long_term(&self, content: &str) -> Result<(), MemoryError> {
        std::fs::write(&self.memory_file, content)?;
        Ok(())
    }

    /// Append an entry to HISTORY.md file.
    ///
    /// Each entry is separated by double newlines.
    pub fn append_history(&self, entry: &str) -> Result<(), MemoryError> {
        use std::io::Write;

        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&self.history_file)?;

        writeln!(file, "{}\n", entry.trim_end())?;
        Ok(())
    }

    /// Get formatted memory context for LLM input.
    ///
    /// Returns a string starting with "## Long-term Memory" header.
    pub fn get_memory_context(&self) -> Result<String, MemoryError> {
        let long_term = self.read_long_term()?;
        if long_term.is_empty() { Ok(String::new()) } else { Ok(format!("## Long-term Memory\n{long_term}")) }
    }

    /// Check if memory consolidation should be triggered.
    ///
    /// # Arguments
    /// * `message_count` - Total number of messages in session
    /// * `last_consolidated` - Index of last consolidated message
    /// * `memory_window` - Total message window size
    /// * `archive_all` - If true, always trigger consolidation
    ///
    /// # Returns
    /// * `Some(new_last_consolidated)` - Should consolidate, returns new index
    /// * `None` - No need to consolidate
    pub fn should_consolidate(
        &self,
        message_count: usize,
        last_consolidated: usize,
        memory_window: usize,
        archive_all: bool,
    ) -> Option<usize> {
        if archive_all {
            return Some(0);
        }

        let keep_count = memory_window / 2;
        if message_count <= keep_count {
            return None;
        }
        if message_count - last_consolidated <= keep_count {
            return None;
        }

        Some(message_count - keep_count)
    }

    /// Try to consolidate memory if needed.
    ///
    /// This method combines `should_consolidate` check with `consolidate` execution.
    /// If consolidation is not needed, returns `Ok(last_consolidated)`.
    ///
    /// # Arguments
    /// * `messages` - All messages in the session
    /// * `last_consolidated` - Index of last consolidated message
    /// * `provider` - LLM provider (already configured with model)
    /// * `archive_all` - If true, process all messages and return 0
    /// * `memory_window` - Total message window size (keep half in session)
    ///
    /// # Returns
    /// * `Ok(new_last_consolidated)` - New index (may equal last_consolidated if no consolidation)
    /// * `Err(_)` - On failure
    pub async fn try_consolidate<P: Provider>(
        &self,
        messages: &[Message],
        last_consolidated: usize,
        provider: P,
        archive_all: bool,
        memory_window: usize,
    ) -> Result<usize, MemoryError> {
        // Check if consolidation is needed
        let new_last_consolidated =
            match self.should_consolidate(messages.len(), last_consolidated, memory_window, archive_all) {
                Some(idx) => idx,
                None => return Ok(last_consolidated),
            };

        info!("Triggering memory consolidation: {} messages, last_consolidated={}", messages.len(), last_consolidated);

        // Execute consolidation
        self.consolidate_internal(messages, last_consolidated, new_last_consolidated, archive_all, provider).await
    }

    /// Internal consolidation logic (assumes checks already done).
    async fn consolidate_internal<P: Provider>(
        &self,
        messages: &[Message],
        last_consolidated: usize,
        new_last_consolidated: usize,
        archive_all: bool,
        mut provider: P,
    ) -> Result<usize, MemoryError> {
        // Determine which messages to archive
        let old_messages = if archive_all {
            info!("Memory consolidation (archive_all): {} messages", messages.len());
            messages.to_vec()
        } else {
            let keep_count = new_last_consolidated - last_consolidated;
            let old_messages: Vec<_> = messages[last_consolidated..messages.len() - keep_count].to_vec();
            if old_messages.is_empty() {
                return Ok(last_consolidated);
            }
            info!("Memory consolidation: {} to consolidate", old_messages.len());
            old_messages
        };

        // Format messages for LLM
        let lines: Vec<String> = old_messages
            .iter()
            .filter_map(|m| {
                let content = m.content();
                if content.is_empty() {
                    return None;
                }
                Some(format!(
                    "[{}] {}: {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M"),
                    m.role().to_uppercase(),
                    content
                ))
            })
            .collect();

        // Build prompt
        let current_memory = self.read_long_term()?;
        let prompt = format!(
            r#"Process this conversation and call the save_memory tool with your consolidation.

## Current Long-term Memory
{}

## Conversation to Process
{}"#,
            if current_memory.is_empty() { "(empty)" } else { &current_memory },
            lines.join("\n")
        );

        // Bind save_memory tool to provider
        provider.bind_tools(vec![create_save_memory_tool()]);

        // Send LLM request
        let options = nanobot_provider::Options::default();
        let response = provider
            .chat(&[
                Message::system("You are a memory consolidation agent. Call the save_memory tool with your consolidation of the conversation."),
                Message::user(&prompt),
            ], &options)
            .await
            .map_err(|e| MemoryError::LlmApi(e.to_string()))?;

        // Check for tool calls
        let tool_calls = response.tool_calls();
        if tool_calls.is_empty() {
            warn!("Memory consolidation: LLM did not call save_memory");
            return Err(MemoryError::NoToolCall);
        }

        // Find save_memory tool call
        let save_memory_call =
            tool_calls.iter().find(|tc| tc.name == SAVE_MEMORY_TOOL).ok_or(MemoryError::NoToolCall)?;

        // Parse arguments
        let args = save_memory_call
            .parse_arguments::<serde_json::Value>()
            .map_err(|e| MemoryError::ToolParse(e.to_string()))?;

        // Extract history_entry
        if let Some(entry) = args.get("history_entry") {
            let entry_str = if entry.is_string() {
                entry.as_str().unwrap_or_default().to_string()
            } else {
                serde_json::to_string(entry).unwrap_or_default()
            };
            self.append_history(&entry_str)?;
        }

        // Extract memory_update
        if let Some(update) = args.get("memory_update") {
            let update_str = if update.is_string() {
                update.as_str().unwrap_or_default().to_string()
            } else {
                serde_json::to_string(update).unwrap_or_default()
            };
            if update_str != current_memory {
                self.write_long_term(&update_str)?;
            }
        }

        info!("Memory consolidation completed: new last_consolidated={}", new_last_consolidated);

        Ok(new_last_consolidated)
    }

    /// Consolidate old messages into MEMORY.md + HISTORY.md via LLM tool call.
    ///
    /// This is an alias for `try_consolidate`, kept for backward compatibility.
    ///
    /// # Arguments
    /// * `messages` - All messages in the session
    /// * `last_consolidated` - Index of last consolidated message
    /// * `provider` - LLM provider (already configured with model)
    /// * `archive_all` - If true, process all messages and return 0
    /// * `memory_window` - Total message window size (keep half in session)
    ///
    /// # Returns
    /// * `Ok(new_last_consolidated)` - New index for last_consolidated
    /// * `Err(_)` - On failure
    pub async fn consolidate<P: Provider>(
        &self,
        messages: &[Message],
        last_consolidated: usize,
        provider: P,
        archive_all: bool,
        memory_window: usize,
    ) -> Result<usize, MemoryError> {
        self.try_consolidate(messages, last_consolidated, provider, archive_all, memory_window).await
    }
}
