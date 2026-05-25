//! Message tool：主动向指定 channel/chat 发送消息，支持媒体附件。

use std::path::PathBuf;

use async_trait::async_trait;
use nanobot_channels::messages::OutboundMessage;
use nanobot_tools::{Tool, ToolContext, ToolError, ToolResult};
use schemars::Schema;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// 消息发送工具，允许 LLM 主动向指定 channel/chat 发送消息。
pub struct MessageTool {
    /// 出站消息发送端
    outbound_tx: mpsc::Sender<OutboundMessage>,
    /// 工作空间路径（用于解析相对媒体路径）
    workspace: PathBuf,
    /// 是否限制媒体路径在工作空间内
    restrict_to_workspace: bool,
}

/// execute() 的参数
#[derive(Deserialize)]
struct MessageParams {
    /// 消息内容（必填）
    content: String,
    /// 目标通道（可选，默认当前 channel）
    channel: Option<String>,
    /// 目标聊天（可选，默认当前 chat_id）
    chat_id: Option<String>,
    /// 媒体文件路径列表（可选）
    #[serde(default)]
    media: Vec<String>,
}

impl MessageTool {
    /// 创建新的 MessageTool。
    pub fn new(outbound_tx: mpsc::Sender<OutboundMessage>, workspace: PathBuf, restrict_to_workspace: bool) -> Self {
        Self { outbound_tx, workspace, restrict_to_workspace }
    }

    /// 解析媒体路径：URL 透传，本地路径相对于 workspace 解析。
    fn resolve_media(&self, paths: &[String]) -> Result<Vec<String>, ToolError> {
        let mut resolved = Vec::with_capacity(paths.len());

        for path in paths {
            if path.starts_with("http://") || path.starts_with("https://") {
                resolved.push(path.clone());
                continue;
            }

            let full_path =
                if PathBuf::from(path).is_absolute() { PathBuf::from(path) } else { self.workspace.join(path) };

            if !full_path.exists() {
                return Err(ToolError::validation("media", format!("file not found: {}", full_path.display())));
            }

            if self.restrict_to_workspace {
                let canonical = full_path
                    .canonicalize()
                    .map_err(|e| ToolError::path(format!("failed to resolve {}: {e}", full_path.display())))?;
                let workspace_canonical = self
                    .workspace
                    .canonicalize()
                    .map_err(|e| ToolError::path(format!("failed to resolve workspace: {e}")))?;
                if !canonical.starts_with(&workspace_canonical) {
                    return Err(ToolError::PermissionDenied {
                        path: full_path.display().to_string(),
                        allowed: Some(workspace_canonical.display().to_string()),
                    });
                }
            }

            resolved.push(full_path.display().to_string());
        }

        Ok(resolved)
    }
}

#[async_trait]
impl Tool for MessageTool {
    fn name(&self) -> &str {
        "message"
    }

    fn description(&self) -> &str {
        "Send a message to a user or channel, optionally with file attachments. \
         Use this for proactive sends, cross-channel delivery, or sending files/images. \
         For normal replies in the current conversation, just respond with text directly."
    }

    fn parameters(&self) -> Schema {
        schemars::schema_for_value!(json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Message content to send."
                },
                "channel": {
                    "type": "string",
                    "description": "Target channel (e.g. 'feishu', 'dingtalk'). Defaults to the current channel."
                },
                "chat_id": {
                    "type": "string",
                    "description": "Target chat/user ID. Defaults to the current chat."
                },
                "media": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional list of file paths or URLs to attach as media."
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, ctx: &ToolContext, params: serde_json::Value) -> ToolResult {
        let params: MessageParams = serde_json::from_value(params)
            .map_err(|e| ToolError::validation("params", format!("invalid parameters: {e}")))?;

        let channel = params.channel.unwrap_or_else(|| ctx.channel.clone());
        let chat_id = params.chat_id.unwrap_or_else(|| ctx.chat_id.clone());

        // 解析媒体路径
        let media = if params.media.is_empty() {
            Vec::new()
        } else {
            match self.resolve_media(&params.media) {
                Ok(resolved) => resolved,
                Err(e) => {
                    warn!("media resolve failed: {e}");
                    return Err(e);
                }
            }
        };

        // 构造出站消息
        let mut msg = OutboundMessage::new(&channel, &chat_id, &params.content);
        for path in &media {
            msg = msg.add_media(path);
        }

        // 发送
        self.outbound_tx.send(msg).await.map_err(|e| ToolError::execution(format!("failed to send message: {e}")))?;

        info!(
            "message tool: sent to {channel}:{chat_id}, content={} chars, media={} files",
            params.content.len(),
            media.len()
        );

        Ok(format!(
            "Message sent to {channel}:{chat_id} ({} chars{})",
            params.content.len(),
            if media.is_empty() { String::new() } else { format!(", {} media files", media.len()) }
        ))
    }
}

#[cfg(test)]
mod tests;
