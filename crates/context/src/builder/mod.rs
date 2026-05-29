/// Bootstrap files that provide system-level context to the agent.
///
/// These files are loaded from the workspace root and integrated into the system prompt
/// to provide the agent with essential configuration, identity, tools, and user preferences.
/// The files are loaded in the order specified and each file's content is prefixed with
/// a section header (e.g., "## AGENTS.md").
///
/// Files are optional - missing files are silently skipped. Only non-empty files
/// with valid UTF-8 encoding are included in the final system prompt.
const BOOTSTRAP_FILES: &[&str] = &["AGENTS.md", "SOUL.md", "USER.md", "TOOLS.md", "IDENTITY.md"];

/// Context builder implementation.
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Local};
use nanobot_memory::MemoryStore;
use nanobot_provider::{ContentPart, Message, UserContent};
use nanobot_skills::SkillsLoader;
use tracing::{info, warn};

use crate::ContextError;

/// Context builder for assembling LLM context.
///
/// This struct holds the workspace path and memory store,
/// providing methods to build system prompts and message lists.
pub struct ContextBuilder {
    /// Canonicalized workspace path (resolved symlinks, absolute path)
    workspace: PathBuf,
    /// Memory store for accessing long-term memory
    memory: Arc<MemoryStore>,
    /// Skills loader for managing agent skills
    skills: SkillsLoader,
}

impl ContextBuilder {
    pub fn new(workspace: PathBuf) -> Result<Self, ContextError> {
        let workspace_canonical = workspace.canonicalize()?;
        let memory = Arc::new(MemoryStore::new(workspace_canonical.clone())?);
        let skills = SkillsLoader::new(workspace_canonical.clone());

        info!("ContextBuilder initialized for workspace: {}", workspace_canonical.display());

        Ok(Self { workspace: workspace_canonical, memory, skills })
    }

    /// Get a reference to the underlying memory store.
    ///
    /// This is useful for operations that need direct access to memory,
    /// such as memory consolidation.
    pub fn memory(&self) -> Arc<MemoryStore> {
        Arc::clone(&self.memory)
    }

    /// Build the core identity section of the system prompt.
    ///
    /// Includes nanobot introduction, runtime info, workspace path,
    /// memory file paths, and tool call guidelines.
    pub fn build_core_identity(&self) -> String {
        let os_name = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let workspace = self.workspace.display();

        format!(
            r#"# nanobot 🐱

You are nanobot, a helpful AI assistant.

## Runtime
{os_name} {arch}

## Workspace
Your workspace is at: {workspace}
- Long-term memory: {workspace}/memory/MEMORY.md (automatically managed by consolidation — do not edit directly)
- History log: {workspace}/memory/history.jsonl (structured JSONL log with cursor tracking, use exec with grep to search). Each entry is a JSON object with cursor, timestamp, and content fields.
- Custom skills: {workspace}/skills/{{skill-name}}/SKILL.md

Reply directly with text for conversations. Only use the 'message' tool to send to a specific chat channel.

## Tool Call Guidelines
- Before calling tools, you may briefly state your intent (e.g. "Let me check that"), but NEVER predict or describe the expected result before receiving it.
- Before modifying a file, read it first to confirm its current content.
- Do not assume a file or directory exists — use list_dir or read_file to verify.
- After writing or editing a file, re-read it if accuracy matters.
- If a tool call fails, analyze the error before retrying with a different approach."#
        )
    }

    /// Load bootstrap files from the workspace directory.
    ///
    /// This method reads bootstrap files (AGENTS.md, SOUL.md, USER.md, TOOLS.md, IDENTITY.md)
    /// from the workspace and combines them into a single string. Each file's content is
    /// prefixed with a section header using the filename.
    ///
    /// # Returns
    /// A string containing all valid bootstrap file contents, joined by newlines.
    /// Returns an empty string if no valid files are found.
    ///
    /// # Behavior
    /// - Files that don't exist are silently skipped
    /// - Files with IO errors are logged as warnings and skipped
    /// - Files with non-UTF-8 encoding are logged as warnings and skipped
    /// - Empty files or files with only whitespace are not included in the output
    ///
    /// # Example Output
    /// ```text
    /// ## AGENTS.md
    /// This is the content of AGENTS.md...
    ///
    /// ## SOUL.md
    /// This is the content of SOUL.md...
    /// ```
    pub fn load_bootstrap_files(&self) -> String {
        let mut sections = Vec::new();

        for filename in BOOTSTRAP_FILES {
            let file_path = self.workspace.join(filename);

            match std::fs::read_to_string(&file_path) {
                Ok(content) => {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        sections.push(format!("## {filename}\n\n{content}"));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // Silently skip missing files
                }
                Err(e) => {
                    warn!("Failed to read bootstrap file {}: {}", file_path.display(), e);
                }
            }
        }

        sections.join("\n\n")
    }

    /// Build the complete system prompt.
    ///
    /// Assembles core identity, bootstrap files, memory context, active skills, and skills summary.
    /// Parts are joined with `---` separator.
    ///
    /// # Assembly Order
    /// 1. Core identity (nanobot introduction, runtime info, workspace path, tool guidelines)
    /// 2. Bootstrap files (AGENTS.md, SOUL.md, USER.md, TOOLS.md, IDENTITY.md)
    /// 3. Memory context (long-term memory contents)
    /// 4. Active Skills (always-loaded skills with full content)
    /// 5. Skills Summary (available skills for on-demand loading)
    pub fn build_system_prompt(&self) -> Result<String, ContextError> {
        let mut parts = Vec::new();

        // Core identity
        parts.push(self.build_core_identity());

        // Bootstrap files
        let bootstrap_content = self.load_bootstrap_files();
        if !bootstrap_content.is_empty() {
            parts.push(bootstrap_content);
        }

        // Memory context
        let memory_context = self.memory.get_memory_context()?;
        if !memory_context.is_empty() {
            parts.push(format!("# Memory\n\n{memory_context}"));
        }

        // Active Skills - always-loaded skills with full content
        match self.skills.get_always_skills() {
            Ok(always_skills) => {
                if !always_skills.is_empty() {
                    let always_content = self.skills.load_skills_for_context(&always_skills);
                    if !always_content.is_empty() {
                        parts.push(format!("# Active Skills\n\n{always_content}"));
                        info!("Loaded {} active skills into context", always_skills.len());
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get always skills: {}", e);
            }
        }

        // Skills Summary - available skills for on-demand loading
        match self.skills.build_skills_summary() {
            Ok(skills_summary) => {
                if !skills_summary.is_empty() {
                    let skills_section = format!(
                        r#"# Skills

The following skills extend your capabilities. To use a skill, read its SKILL.md file using the read_file tool.
Skills with available="false" need dependencies installed first - you can try installing them with apt/brew.

{skills_summary}"#,
                    );
                    parts.push(skills_section);
                }
            }
            Err(e) => {
                warn!("Failed to build skills summary: {}", e);
            }
        }

        Ok(parts.join("\n\n---\n\n"))
    }

    /// Build the complete message list for an LLM call.
    ///
    /// # Arguments
    /// * `history` - Previous conversation messages
    /// * `current_message` - The new user message
    /// * `media` - Optional list of local file paths for images
    /// * `channel` - Optional channel name
    /// * `chat_id` - Optional chat/user ID
    ///
    /// # Returns
    /// A vector of messages including system prompt, history, and current user message.
    pub fn build_messages(
        &self,
        history: &[Message],
        current_message: &str,
        media: Option<&[PathBuf]>,
        channel: Option<&str>,
        chat_id: Option<&str>,
    ) -> Result<Vec<Message>, ContextError> {
        let mut messages = Vec::new();

        // System prompt
        let system_prompt = self.build_system_prompt()?;
        messages.push(Message::system(&system_prompt));

        // History
        messages.extend(history.iter().cloned());

        // Build user content with optional media
        let user_content = build_user_content(current_message, media)?;
        let user_content = inject_runtime_context(user_content, channel, chat_id);
        messages.push(Message::User { content: user_content });

        Ok(messages)
    }
}

/// Encode an image file to base64.
///
/// Returns `(media_type, base64_data)` tuple.
/// Returns `None` if file doesn't exist or is not an image type.
///
/// MIME 类型优先通过文件头 magic bytes 检测，回退到文件扩展名猜测。
/// 这避免了文件扩展名与实际内容不匹配的问题（如钉钉下载的 JPEG 图片保存为 .png）。
fn encode_image_to_base64(path: &PathBuf) -> Result<Option<(String, String)>, ContextError> {
    if !path.is_file() {
        return Ok(None);
    }

    // Read file bytes first
    let bytes = std::fs::read(path)?;

    // Detect MIME type from magic bytes, fallback to extension
    let mime = detect_image_mime(&bytes)
        .or_else(|| mime_guess::from_path(path).first().map(|m| m.to_string()))
        .filter(|m| m.starts_with("image/"));

    let Some(mime) = mime else {
        return Ok(None);
    };

    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

    Ok(Some((mime, encoded)))
}

/// Detect image MIME type from file header magic bytes.
fn detect_image_mime(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 4 {
        return None;
    }

    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        Some("image/png".to_string())
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg".to_string())
    } else if bytes.starts_with(b"GIF8") {
        Some("image/gif".to_string())
    } else if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
        Some("image/webp".to_string())
    } else {
        None
    }
}

/// Build user message content with optional base64-encoded images.
///
/// Returns `UserContent::Text` for text-only messages,
/// `UserContent::Parts` for messages with images.
fn build_user_content(text: &str, media: Option<&[PathBuf]>) -> Result<UserContent, ContextError> {
    let media = match media {
        Some(m) if !m.is_empty() => m,
        _ => return Ok(UserContent::Text(text.to_string())),
    };

    // Encode images
    let mut parts: Vec<ContentPart> = Vec::new();

    // Text content first
    if !text.is_empty() {
        parts.push(ContentPart::Text { text: text.to_string() });
    }

    // Then images
    for path in media {
        if let Some((media_type, data)) = encode_image_to_base64(path)? {
            parts.push(ContentPart::Image { media_type, data });
        }
    }

    if parts.is_empty() {
        return Ok(UserContent::Text(text.to_string()));
    }

    // If no images were added, return as plain text
    let has_images = parts.iter().any(|p| matches!(p, ContentPart::Image { .. }));
    if !has_images {
        return Ok(UserContent::Text(text.to_string()));
    }

    Ok(UserContent::Parts(parts))
}

/// Inject runtime context into user message content.
///
/// For `UserContent::Text`, appends runtime context to the text string.
/// For `UserContent::Parts`, appends a new text part with runtime context.
fn inject_runtime_context(content: UserContent, channel: Option<&str>, chat_id: Option<&str>) -> UserContent {
    let now: DateTime<Local> = Local::now();
    let weekday = now.format("%A");
    let time_str = now.format("%Y-%m-%d %H:%M");

    let tz = Local::now().offset().to_string();

    let mut lines = vec![format!("Current Time: {} ({}) ({})", time_str, weekday, tz)];

    if let Some(ch) = channel {
        lines.push(format!("Channel: {ch}"));
    }
    if let Some(id) = chat_id {
        lines.push(format!("Chat ID: {id}"));
    }

    let block = format!("[Runtime Context]\n{}", lines.join("\n"));

    match content {
        UserContent::Text(text) => UserContent::Text(format!("{text}\n\n{block}")),
        UserContent::Parts(mut parts) => {
            parts.push(ContentPart::Text { text: format!("\n\n{block}") });
            UserContent::Parts(parts)
        }
    }
}

#[cfg(test)]
mod tests;
