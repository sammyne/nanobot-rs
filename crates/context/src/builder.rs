/// Context builder implementation.
use std::path::PathBuf;

use chrono::{DateTime, Local};
use nanobot_memory::MemoryStore;
use nanobot_provider::{Message, ToolCall};
use tracing::info;

use crate::ContextError;

/// Context builder for assembling LLM context.
///
/// This struct holds the workspace path and memory store,
/// providing methods to build system prompts and message lists.
pub struct ContextBuilder {
    /// Workspace root path
    workspace: PathBuf,
    /// Memory store for accessing long-term memory
    memory: MemoryStore,
}

impl ContextBuilder {
    /// Create a new ContextBuilder instance.
    ///
    /// # Arguments
    /// * `workspace` - Path to the workspace directory
    ///
    /// # Errors
    /// Returns `ContextError::InvalidPath` if the workspace path doesn't exist.
    pub fn new(workspace: PathBuf) -> Result<Self, ContextError> {
        if !workspace.exists() {
            return Err(ContextError::InvalidPath(format!(
                "Workspace does not exist: {}",
                workspace.display()
            )));
        }

        let memory = MemoryStore::new(workspace.clone())?;

        info!("ContextBuilder initialized for workspace: {}", workspace.display());

        Ok(Self { workspace, memory })
    }

    /// Get a reference to the underlying memory store.
    ///
    /// This is useful for operations that need direct access to memory,
    /// such as memory consolidation.
    pub fn memory(&self) -> &MemoryStore {
        &self.memory
    }

    /// Build the core identity section of the system prompt.
    ///
    /// Includes nanobot introduction, runtime info, workspace path,
    /// memory file paths, and tool call guidelines.
    pub fn build_core_identity(&self) -> String {
        let workspace_path = self.workspace.canonicalize().unwrap_or_else(|_| self.workspace.clone());

        let os_name = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let runtime = format!("{} {}", os_name, arch);

        format!(
            r#"# nanobot 🐈

You are nanobot, a helpful AI assistant.

## Runtime
{}

## Workspace
Your workspace is at: {}
- Long-term memory: {}/memory/MEMORY.md
- History log: {}/memory/HISTORY.md (grep-searchable)
- Custom skills: {}/skills/{{skill-name}}/SKILL.md

Reply directly with text for conversations. Only use the 'message' tool to send to a specific chat channel.

## Tool Call Guidelines
- Before calling tools, you may briefly state your intent (e.g. "Let me check that"), but NEVER predict or describe the expected result before receiving it.
- Before modifying a file, read it first to confirm its current content.
- Do not assume a file or directory exists — use list_dir or read_file to verify.
- After writing or editing a file, re-read it if accuracy matters.
- If a tool call fails, analyze the error before retrying with a different approach.

## Memory
- Remember important facts: write to {}/memory/MEMORY.md
- Recall past events: grep {}/memory/HISTORY.md"#,
            runtime,
            workspace_path.display(),
            workspace_path.display(),
            workspace_path.display(),
            workspace_path.display(),
            workspace_path.display(),
            workspace_path.display()
        )
    }

    /// Build the complete system prompt.
    ///
    /// Assembles core identity, memory context, and skills.
    /// Parts are joined with `---` separator.
    pub fn build_system_prompt(&self) -> Result<String, ContextError> {
        let mut parts = Vec::new();

        // Core identity
        parts.push(self.build_core_identity());

        // Memory context
        let memory_context = self.memory.get_memory_context()?;
        if !memory_context.is_empty() {
            parts.push(format!("# Memory\n\n{}", memory_context));
        }

        Ok(parts.join("\n\n---\n\n"))
    }

    /// Inject runtime context into user message content.
    ///
    /// Appends current time and optional channel/chat_id information.
    ///
    /// # Arguments
    /// * `content` - Original user message content
    /// * `channel` - Optional channel name (telegram, feishu, etc.)
    /// * `chat_id` - Optional chat/user ID
    pub fn inject_runtime_context(content: &str, channel: Option<&str>, chat_id: Option<&str>) -> String {
        let now: DateTime<Local> = Local::now();
        let weekday = now.format("%A");
        let time_str = now.format("%Y-%m-%d %H:%M");

        let tz = Local::now().offset().to_string();

        let mut lines = vec![format!("Current Time: {} ({}) ({})", time_str, weekday, tz)];

        if let Some(ch) = channel {
            lines.push(format!("Channel: {}", ch));
        }
        if let Some(id) = chat_id {
            lines.push(format!("Chat ID: {}", id));
        }

        let block = format!("[Runtime Context]\n{}", lines.join("\n"));
        format!("{}\n\n{}", content, block)
    }

    /// Encode an image file to base64 data URL format.
    ///
    /// Returns `data:{mime};base64,{data}` format string.
    /// Returns `None` if file doesn't exist or is not an image type.
    pub fn encode_image_to_base64(path: &PathBuf) -> Result<Option<String>, ContextError> {
        if !path.is_file() {
            return Ok(None);
        }

        // Guess MIME type from file extension
        let mime = mime_guess::from_path(path).first().map(|m| m.to_string());

        let mime = match mime {
            Some(m) if m.starts_with("image/") => m,
            _ => return Ok(None),
        };

        // Read and encode file
        let bytes = std::fs::read(path)?;
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

        Ok(Some(format!("data:{};base64,{}", mime, encoded)))
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
        let user_content = self.build_user_content(current_message, media)?;
        let user_content = Self::inject_runtime_context(&user_content, channel, chat_id);
        messages.push(Message::user(&user_content));

        Ok(messages)
    }

    /// Build user message content with optional base64-encoded images.
    ///
    /// Currently returns text content only. Media support can be extended
    /// when multimodal Message type is available.
    fn build_user_content(&self, text: &str, media: Option<&[PathBuf]>) -> Result<String, ContextError> {
        let media = match media {
            Some(m) if !m.is_empty() => m,
            _ => return Ok(text.to_string()),
        };

        // Process images - for now, just add info about attached media
        // Full multimodal support would require extending Message type
        let mut image_info = Vec::new();
        for path in media {
            if let Some(_data_url) = Self::encode_image_to_base64(path)? {
                image_info.push(format!("[Image attached: {}]", path.display()));
            }
        }

        if image_info.is_empty() {
            return Ok(text.to_string());
        }

        Ok(format!("{}\n\n{}", image_info.join("\n"), text))
    }

    /// Append a tool result to the message list.
    ///
    /// # Arguments
    /// * `messages` - Current message list
    /// * `tool_call_id` - ID of the tool call
    /// * `result` - Tool execution result
    pub fn append_tool_result(messages: &mut Vec<Message>, tool_call_id: impl Into<String>, result: impl Into<String>) {
        messages.push(Message::tool(tool_call_id, result));
    }

    /// Append an assistant message to the message list.
    ///
    /// # Arguments
    /// * `messages` - Current message list
    /// * `content` - Message content
    /// * `tool_calls` - Optional tool calls
    pub fn append_assistant_message(
        messages: &mut Vec<Message>,
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) {
        if tool_calls.is_empty() {
            messages.push(Message::assistant(content));
        } else {
            messages.push(Message::assistant_with_tools(content, tool_calls));
        }
    }
}
