//! Dream two-phase memory processor.
//!
//! Phase 1 (Analyze): reads new history entries and current memory files,
//! asks the LLM to identify atomic facts worth recording.
//!
//! Phase 2 (Edit): appends the identified facts to the appropriate memory files.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use nanobot_config::DreamConfig;
use nanobot_provider::{Message, Options, Provider};
use tracing::info;

use crate::MemoryError;
use crate::gitstore::GitStore;
use crate::history::HistoryEntry;
use crate::store::MemoryStore;

/// Result of a Dream processing run.
pub struct DreamResult {
    /// Number of history entries processed.
    pub entries_processed: usize,
    /// List of memory files that were changed.
    pub files_changed: Vec<String>,
}

/// Two-phase memory processor.
///
/// Reads unprocessed history entries, asks an LLM to extract atomic facts
/// (Phase 1), then appends those facts to the target memory files (Phase 2).
/// Each run is committed via [`GitStore`] for version control.
pub struct Dream {
    memory: Arc<MemoryStore>,
    git: GitStore,
    config: DreamConfig,
    /// Path to the workspace root (parent of `memory/`).
    workspace: PathBuf,
}

impl Dream {
    /// Create a new Dream processor.
    ///
    /// Initializes a [`GitStore`] in the memory directory (`workspace/memory/`).
    /// Returns an error if `git` is not available on the system.
    pub fn new(memory: Arc<MemoryStore>, workspace: PathBuf, config: DreamConfig) -> Result<Self, MemoryError> {
        let memory_dir = workspace.join("memory");
        let git = GitStore::init(memory_dir)?;
        Ok(Self { memory, git, config, workspace })
    }

    /// Run the two-phase Dream processor.
    ///
    /// 1. Reads unprocessed history entries (after the dream cursor).
    /// 2. Batches entries up to `config.max_batch_size`.
    /// 3. Phase 1: analyzes entries against current memory files via LLM.
    /// 4. Phase 2: appends extracted facts to the target files.
    /// 5. Advances the dream cursor and commits changes via git.
    pub async fn run<P: Provider>(&self, provider: &P) -> Result<DreamResult, MemoryError> {
        let dream_cursor = self.read_dream_cursor();

        let entries = self.memory.history().read_since(dream_cursor)?;
        if entries.is_empty() {
            return Ok(DreamResult { entries_processed: 0, files_changed: vec![] });
        }

        let entries: Vec<_> = entries.into_iter().take(self.config.max_batch_size).collect();
        info!("dream: processing {} entries (cursor={dream_cursor})", entries.len());

        // Phase 1: analyze
        let instructions = self.phase1_analyze(provider, &entries).await?;

        // Phase 2: edit
        let files_changed = self.phase2_edit(provider, &instructions).await?;

        // Advance cursor
        let new_cursor = entries.last().map(|e| e.cursor).unwrap_or(dream_cursor);
        self.write_dream_cursor(new_cursor)?;

        // Git commit
        let commit_msg = format!("dream: process {} entries", entries.len());
        self.git.commit(&commit_msg)?;

        info!("dream: done, {} files changed", files_changed.len());

        Ok(DreamResult { entries_processed: entries.len(), files_changed })
    }

    /// Read the dream cursor from `.dream_cursor` file. Returns 0 if absent.
    fn read_dream_cursor(&self) -> u64 {
        std::fs::read_to_string(self.cursor_path()).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0)
    }

    /// Write the dream cursor to `.dream_cursor` file.
    fn write_dream_cursor(&self, cursor: u64) -> Result<(), MemoryError> {
        std::fs::write(self.cursor_path(), cursor.to_string())?;
        Ok(())
    }

    /// Path to the `.dream_cursor` file.
    fn cursor_path(&self) -> PathBuf {
        self.workspace.join("memory").join(".dream_cursor")
    }

    /// Phase 1: Analyze new history entries against current memory files.
    ///
    /// Builds a prompt containing the new entries and current file contents,
    /// then asks the LLM to output `[FILE] fact` lines.
    async fn phase1_analyze<P: Provider>(
        &self,
        provider: &P,
        entries: &[HistoryEntry],
    ) -> Result<Vec<String>, MemoryError> {
        let memory_content = self.memory.read_long_term()?;
        let soul_content = read_file_or_empty(&self.workspace.join("SOUL.md"));
        let user_content = read_file_or_empty(&self.workspace.join("USER.md"));

        let prompt = build_phase1_prompt(entries, &memory_content, &soul_content, &user_content);

        let response = provider
            .chat(&[Message::system(PHASE1_SYSTEM_PROMPT), Message::user(&prompt)], &Options::default())
            .await
            .map_err(|e| MemoryError::LlmApi(e.to_string()))?;

        let text = response.content();
        Ok(parse_phase1_response(&text))
    }

    /// Phase 2: Apply `[FILE] fact` instructions to memory files.
    ///
    /// This is a simplified implementation that appends each fact to the target
    /// file. A future version may use an LLM-driven re_act loop for surgical edits.
    async fn phase2_edit<P: Provider>(
        &self,
        _provider: &P,
        instructions: &[String],
    ) -> Result<Vec<String>, MemoryError> {
        let mut files_changed = Vec::new();

        for instruction in instructions {
            let Some((file_name, fact)) = parse_instruction(instruction) else {
                continue;
            };

            let Some(path) = self.resolve_memory_file(file_name) else {
                info!("dream phase2: skipping unknown file {file_name:?}");
                continue;
            };

            append_fact(&path, fact)?;

            if !files_changed.contains(&file_name.to_string()) {
                files_changed.push(file_name.to_string());
            }
        }

        Ok(files_changed)
    }

    /// Map a memory file name to its absolute path.
    fn resolve_memory_file(&self, name: &str) -> Option<PathBuf> {
        match name {
            "MEMORY.md" => Some(self.workspace.join("memory").join("MEMORY.md")),
            "SOUL.md" => Some(self.workspace.join("SOUL.md")),
            "USER.md" => Some(self.workspace.join("USER.md")),
            _ => None,
        }
    }
}

/// System prompt for Phase 1 analysis.
const PHASE1_SYSTEM_PROMPT: &str = "\
You are a memory analyst. Compare the new history entries against the existing memory files \
and output atomic facts that should be added or updated.\n\
\n\
Rules:\n\
- Output one fact per line in [FILE] fact format.\n\
- Valid files: MEMORY.md, SOUL.md, USER.md\n\
- MEMORY.md: project facts, technical decisions, environment info.\n\
- SOUL.md: bot communication style and tone preferences.\n\
- USER.md: user preferences and habits.\n\
- Only output facts that are NEW or DIFFERENT from what the files already contain.\n\
- If there is nothing new, output nothing.\n\
\n\
Example output:\n\
[MEMORY.md] Project migrated from SQLite to PostgreSQL\n\
[USER.md] User prefers vim keybindings";

/// Read a file, returning an empty string if it does not exist or cannot be read.
fn read_file_or_empty(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// Build the user prompt for Phase 1.
fn build_phase1_prompt(entries: &[HistoryEntry], memory: &str, soul: &str, user: &str) -> String {
    let mut prompt = String::from("## New History Entries\n\n");
    for entry in entries {
        prompt.push_str(&format!("[cursor={}, {}] {}\n", entry.cursor, entry.timestamp, entry.content));
    }

    prompt.push_str("\n## Current MEMORY.md\n");
    prompt.push_str(if memory.is_empty() { "(empty)\n" } else { memory });
    if !prompt.ends_with('\n') {
        prompt.push('\n');
    }

    prompt.push_str("\n## Current SOUL.md\n");
    prompt.push_str(if soul.is_empty() { "(empty)\n" } else { soul });
    if !prompt.ends_with('\n') {
        prompt.push('\n');
    }

    prompt.push_str("\n## Current USER.md\n");
    prompt.push_str(if user.is_empty() { "(empty)\n" } else { user });
    if !prompt.ends_with('\n') {
        prompt.push('\n');
    }

    prompt
}

/// Extract `[FILE] fact` lines from LLM Phase 1 response.
fn parse_phase1_response(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| {
            l.starts_with('[')
                && l.find(']').is_some_and(|pos| {
                    let after = l[pos + 1..].trim_start();
                    !after.is_empty()
                })
        })
        .map(|l| l.to_string())
        .collect()
}

/// Parse a single `[FILE] fact` instruction into (file_name, fact).
fn parse_instruction(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let rest = line.strip_prefix('[')?;
    let close = rest.find(']')?;
    let file_name = &rest[..close];
    let fact = rest[close + 1..].trim_start();
    if fact.is_empty() {
        return None;
    }
    Some((file_name, fact))
}

/// Append a fact as a bullet point to a file, creating it if necessary.
fn append_fact(path: &Path, fact: &str) -> Result<(), MemoryError> {
    let mut content = std::fs::read_to_string(path).unwrap_or_default();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("- ");
    content.push_str(fact);
    content.push('\n');
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests;
