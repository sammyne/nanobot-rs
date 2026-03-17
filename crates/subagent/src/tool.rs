//! Spawn tool for creating background subagents.

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use nanobot_provider::Provider;
use nanobot_tools::{Tool, ToolContext, ToolError, ToolResult};
use schemars::schema::SchemaObject;

use crate::manager::SubagentManager;

/// Tool to spawn a subagent for background task execution.
///
/// This tool allows the main agent to spawn a background subagent to handle
/// complex or time-consuming tasks. The subagent will complete the task and
/// report back when done.
pub struct SpawnTool<P>
where
    P: Provider + Clone + Send + Sync + 'static,
{
    /// Subagent manager
    manager: Arc<SubagentManager<P>>,
}

impl<P> SpawnTool<P>
where
    P: Provider + Clone + Send + Sync + 'static,
{
    /// Create a new SpawnTool instance.
    ///
    /// # Arguments
    /// * `manager` - The subagent manager to use for spawning subagents
    pub fn new(manager: Arc<SubagentManager<P>>) -> Self {
        Self { manager }
    }
}

/// Parameters for the spawn tool
#[derive(Debug, Clone, serde::Deserialize)]
struct SpawnParams {
    /// The task for the subagent to complete
    task: String,
    /// Optional short label for the task (for display)
    #[serde(default)]
    label: Option<String>,
}

/// Parameters schema for spawn tool
static SPAWN_PARAMS_SCHEMA: LazyLock<SchemaObject> = LazyLock::new(|| {
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "task": {
                "type": "string",
                "description": "The task for the subagent to complete"
            },
            "label": {
                "type": "string",
                "description": "Optional short label for the task (for display)"
            }
        },
        "required": ["task"]
    }))
    .unwrap_or_default()
});

#[async_trait]
impl<P> Tool for SpawnTool<P>
where
    P: Provider + Clone + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        "spawn"
    }

    fn description(&self) -> &str {
        "Spawn a subagent to handle a task in the background. \
         Use this for complex or time-consuming tasks that can run independently. \
         The subagent will complete the task and report back when done."
    }

    fn parameters(&self) -> SchemaObject {
        SPAWN_PARAMS_SCHEMA.clone()
    }

    async fn execute(&self, ctx: &ToolContext, params: serde_json::Value) -> ToolResult {
        // Parse parameters
        let params: SpawnParams =
            serde_json::from_value(params).map_err(|e| ToolError::validation("params", e.to_string()))?;

        // Spawn the subagent
        self.manager
            .clone()
            .spawn(params.task, params.label, ctx.channel.clone(), ctx.chat_id.clone())
            .await
            .map_err(|e| ToolError::execution(e.to_string()))
    }
}
