//! Memory store implementation for persistent agent memory.

use std::path::PathBuf;

use nanobot_provider::{Message, Options, Provider};
use tracing::{info, warn};

use crate::MemoryError;
use crate::history::History;

/// 最大整合轮次
pub const MAX_CONSOLIDATION_ROUNDS: usize = 5;

/// 连续失败多少次后降级为原文转储
pub const MAX_FAILURES_BEFORE_RAW_ARCHIVE: usize = 3;

/// Two-layer memory store: MEMORY.md (long-term facts) + history.jsonl (structured log with cursor).
pub struct MemoryStore {
    /// Path to MEMORY.md file
    memory_file: PathBuf,
    /// Structured history store (history.jsonl)
    history: History,
}

impl MemoryStore {
    /// Create a new MemoryStore instance.
    ///
    /// Creates the memory/ directory under the workspace if it doesn't exist.
    pub fn new(workspace: PathBuf) -> Result<Self, MemoryError> {
        let memory_dir = workspace.join("memory");
        std::fs::create_dir_all(&memory_dir)?;

        let memory_file = memory_dir.join("MEMORY.md");
        let history = History::new(&memory_dir);

        Ok(Self { memory_file, history })
    }

    /// Get a reference to the history store.
    pub fn history(&self) -> &History {
        &self.history
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

    /// Append an entry to history.jsonl via the History store.
    ///
    /// Returns the cursor assigned to this entry.
    pub fn append_history(&self, entry: &str) -> Result<u64, MemoryError> {
        self.history.append(entry)
    }

    /// Get formatted memory context for LLM input.
    ///
    /// Returns a string starting with "## Long-term Memory" header.
    pub fn get_memory_context(&self) -> Result<String, MemoryError> {
        let long_term = self.read_long_term()?;
        if long_term.is_empty() { Ok(String::new()) } else { Ok(format!("## Long-term Memory\n{long_term}")) }
    }

    /// 在 user 消息边界处选择整合切割点
    ///
    /// 从 `last_consolidated` 向前扫描，累加 token，在 user 消息边界处记录切割点。
    /// 当累计移除的 token 达到 `tokens_to_remove` 时返回。
    ///
    /// # Returns
    /// * `Some((end_idx, removed_tokens))` - 切割点索引和实际移除的 token 数
    /// * `None` - 无法找到合适的切割点
    pub fn pick_consolidation_boundary(
        messages: &[Message],
        last_consolidated: usize,
        tokens_to_remove: usize,
    ) -> Option<(usize, usize)> {
        if last_consolidated >= messages.len() || tokens_to_remove == 0 {
            return None;
        }

        let mut removed_tokens = 0usize;
        let mut last_boundary: Option<(usize, usize)> = None;

        for (idx, msg) in messages.iter().enumerate().skip(last_consolidated) {
            if idx > last_consolidated && msg.role() == "user" {
                last_boundary = Some((idx, removed_tokens));
                if removed_tokens >= tokens_to_remove {
                    return last_boundary;
                }
            }
            removed_tokens += msg.token_len();
        }

        last_boundary
    }

    /// 将消息原文转储到 history.jsonl（降级策略）
    ///
    /// 当 LLM 整合连续失败时，直接将消息原文写入历史日志，避免消息丢失。
    pub fn raw_archive(&self, messages: &[Message]) -> Result<(), MemoryError> {
        let lines: Vec<String> = messages
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

        let entry = format!("[RAW] {} messages\n{}", messages.len(), lines.join("\n"));
        self.append_history(&entry)?;
        Ok(())
    }
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

/// Consolidate conversation messages into a plain-text summary stored in history.jsonl.
///
/// Checks if consolidation is needed via [`should_consolidate`], then calls the LLM
/// for a plain-text summary and appends the result to history.
///
/// # Arguments
/// * `memory` - The memory store
/// * `messages` - All messages in the session
/// * `last_consolidated` - Index of last consolidated message
/// * `provider` - LLM provider for generating summaries
/// * `archive_all` - If true, process all messages
/// * `memory_window` - Total message window size (keep half in session)
/// * `options` - LLM call options (max_tokens, temperature, etc.)
///
/// # Returns
/// * `Ok(new_last_consolidated)` - New index (may equal last_consolidated if no consolidation)
/// * `Err(_)` - On failure
pub async fn consolidate_memory<P: Provider>(
    memory: &MemoryStore,
    messages: &[Message],
    last_consolidated: usize,
    provider: &P,
    archive_all: bool,
    memory_window: usize,
    options: &Options,
) -> Result<usize, MemoryError> {
    // Check if consolidation is needed
    let new_last_consolidated = match should_consolidate(messages.len(), last_consolidated, memory_window, archive_all)
    {
        Some(idx) => idx,
        None => return Ok(last_consolidated),
    };

    info!("Triggering memory consolidation: {} messages, last_consolidated={}", messages.len(), last_consolidated);

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
            Some(format!("[{}] {}: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M"), m.role().to_uppercase(), content))
        })
        .collect();

    // Build prompt for plain text summary
    let current_memory = memory.read_long_term()?;
    let prompt = format!(
        r#"Summarize this conversation excerpt. Write a paragraph (2-5 sentences) capturing key events, decisions, topics, and solutions. Start with [YYYY-MM-DD HH:MM]. Include detail useful for grep search.

## Current Long-term Memory
{}

## Conversation to Process
{}"#,
        if current_memory.is_empty() { "(empty)" } else { &current_memory },
        lines.join("\n")
    );

    // Call LLM for plain text summary (no tools bound)
    let response = provider
        .chat(
            &[
                Message::system("You are a memory consolidation agent. Produce a concise plain-text summary."),
                Message::user(&prompt),
            ],
            options,
        )
        .await
        .map_err(|e| MemoryError::LlmApi(e.to_string()))?;

    let summary = response.content();
    let summary = summary.trim();
    if summary.is_empty() {
        warn!("Memory consolidation: LLM returned empty summary");
        return Err(MemoryError::LlmApi("LLM returned empty summary".to_string()));
    }

    // Append summary to history.jsonl
    memory.append_history(summary)?;

    info!("Memory consolidation completed: new last_consolidated={new_last_consolidated}");

    Ok(new_last_consolidated)
}
// trigger ci
