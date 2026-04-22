//! Heartbeat service implementation

use std::path::PathBuf;
use std::sync::Arc;

use nanobot_config::HeartbeatConfig;
use nanobot_provider::{Message, Provider};
use schemars::JsonSchema;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::{HeartbeatError, OnExecuteCallback, OnNotifyCallback};

/// Action enum for heartbeat decision
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
enum Action {
    /// Skip execution
    Skip,
    /// Execute tasks
    Run {
        /// Natural language summary of active tasks to execute
        tasks: String,
    },
}

/// Heartbeat tool definition for LLM decision making
static HEARTBEAT_TOOL: std::sync::LazyLock<nanobot_tools::ToolDefinition> =
    std::sync::LazyLock::new(|| nanobot_tools::ToolDefinition {
        name: "heartbeat".to_string(),
        description: "Decide whether to execute tasks based on HEARTBEAT.md content".to_string(),
        parameters: schemars::schema_for!(Action).to_value(),
    });

/// Maximum retries when tool argument parsing fails
const MAX_PARSE_RETRIES: u8 = 1;

/// Heartbeat service for periodic task checking
pub struct HeartbeatService<P>
where
    P: Provider,
{
    /// Path to HEARTBEAT.md file
    filepath: PathBuf,
    /// LLM provider
    provider: P,
    /// Heartbeat configuration
    config: HeartbeatConfig,
    /// Execute callback
    on_execute: Arc<RwLock<Option<OnExecuteCallback>>>,
    /// Notify callback
    on_notify: Arc<RwLock<Option<OnNotifyCallback>>>,
}

impl<P> HeartbeatService<P>
where
    P: Provider,
{
    /// Create a new heartbeat service
    ///
    /// # Arguments
    ///
    /// * `workspace` - Path to the workspace directory
    /// * `provider` - LLM provider for decision making
    /// * `config` - Heartbeat configuration
    /// * `on_execute` - Optional callback for executing tasks
    /// * `on_notify` - Optional callback for notifying task results
    ///
    /// # Returns
    ///
    /// A new `HeartbeatService` instance with heartbeat tool bound to provider
    pub fn new(
        workspace: PathBuf,
        mut provider: P,
        config: HeartbeatConfig,
        on_execute: Option<OnExecuteCallback>,
        on_notify: Option<OnNotifyCallback>,
    ) -> Self {
        // Bind heartbeat tool to provider once during initialization
        provider.bind_tools(vec![HEARTBEAT_TOOL.clone()]);

        Self {
            filepath: workspace.join("HEARTBEAT.md"),
            provider,
            config,
            on_execute: Arc::new(RwLock::new(on_execute)),
            on_notify: Arc::new(RwLock::new(on_notify)),
        }
    }

    /// Start the heartbeat service
    pub async fn start(self) -> Result<(), HeartbeatError> {
        // Check if disabled
        if !self.config.enabled {
            info!("Heartbeat disabled");
            return Err(HeartbeatError::Disabled);
        }

        // Create interval timer
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(self.config.interval_s));
        interval.tick().await; // Skip first immediate tick

        info!("Heartbeat started (every {}s)", self.config.interval_s);

        // Main heartbeat loop
        loop {
            interval.tick().await;

            // Execute a single heartbeat tick
            if let Err(e) = self.tick().await {
                error!("Heartbeat error: {:?}", e);
            }
        }
    }

    /// Execute a complete heartbeat check (Phase 1 + Phase 2)
    ///
    /// This method combines decide and execute with proper error handling
    /// It's used by both manual trigger and periodic timer
    ///
    /// # Errors
    ///
    /// Returns `HeartbeatError` if check or execution fails
    ///
    /// # Returns
    ///
    /// - `Ok(Some(result))` - Execution result if action="run"
    /// - `Ok(None)` - If action="skip" or HEARTBEAT.md not found
    pub(crate) async fn tick(&self) -> Result<Option<String>, HeartbeatError> {
        // Phase 1: Decide
        let action = match self.decide().await? {
            Some(action) => action,
            None => {
                info!("HEARTBEAT.md not found or empty, skipping");
                return Ok(None);
            }
        };

        // Check action
        match action {
            Action::Run { tasks } => {
                info!("Action: run");

                // Phase 2: Execute - call execute callback
                let on_execute = self.on_execute.read().await;
                let callback = match on_execute.as_ref() {
                    Some(cb) => cb.clone(),
                    None => {
                        info!("No execute callback configured, skipping execution");
                        return Ok(None);
                    }
                };
                drop(on_execute);

                let result = callback(&tasks).await.map_err(HeartbeatError::Execute)?;

                // Check result
                if result.trim().is_empty() {
                    info!("Execute callback returned empty result");
                    return Ok(None);
                }

                // Notify callback if configured
                let on_notify = self.on_notify.read().await;
                if let Some(notify_callback) = on_notify.as_ref() {
                    notify_callback(&result).await.map_err(HeartbeatError::Notify)?;
                }

                Ok(Some(result))
            }
            Action::Skip => {
                info!("Action: skip");
                Ok(None)
            }
        }
    }

    /// Phase 1: Check heartbeat - Decision phase
    ///
    /// Reads HEARTBEAT.md and asks LLM to decide if tasks need execution.
    /// Includes retry mechanism when tool argument parsing fails.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Action))` - LLM decision
    /// - `Ok(None)` - HEARTBEAT.md not found or empty
    /// - `Err(HeartbeatError)` - Error occurred during check
    async fn decide(&self) -> Result<Option<Action>, HeartbeatError> {
        // Read HEARTBEAT.md file
        let content = match tokio::fs::read_to_string(&self.filepath).await {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("HEARTBEAT.md not found, skipping heartbeat check");
                return Ok(None);
            }
            Err(e) => {
                error!("Failed to read HEARTBEAT.md: {}", e);
                return Err(HeartbeatError::FileRead(e));
            }
        };

        // Check if content is empty or only whitespace
        if content.trim().is_empty() {
            info!("HEARTBEAT.md is empty, skipping heartbeat check");
            return Ok(None);
        }

        // Prepare initial messages for LLM
        let mut messages = vec![
            Message::system("You are a heartbeat agent. Call the heartbeat tool to report your decision."),
            Message::user(format!(
                "Review the following HEARTBEAT.md and decide whether there are active tasks.\n\n{content}"
            )),
        ];

        let options = nanobot_provider::Options::default();

        // Retry loop: if parsing fails, feed the error back to LLM for correction
        for attempt in 0..=MAX_PARSE_RETRIES {
            let response = self.provider.chat(&messages, &options).await.map_err(HeartbeatError::Provider)?;

            // Handle case: no tool call returned
            let Some(tool_call) = response.tool_calls().first() else {
                info!("LLM did not return a tool call, treating as skip");
                return Ok(Some(Action::Skip));
            };

            // Handle case: wrong tool name
            if tool_call.name != "heartbeat" {
                warn!("Unexpected tool name: {}, treating as skip", tool_call.name);
                return Ok(Some(Action::Skip));
            }

            // Try to parse the arguments
            match tool_call.parse_arguments::<Action>() {
                Ok(action) => return Ok(Some(action)),
                Err(e) if attempt < MAX_PARSE_RETRIES => {
                    // Build error feedback for LLM
                    let error_msg = format!(
                        "Invalid arguments format. Error: {e}.\n\
                         Expected JSON format:\n\
                         - To skip: \"skip\"\n\
                         - To run tasks: {{\"run\": {{\"tasks\": \"<task description>\"}}}}\n\
                         Please retry with a valid format."
                    );

                    warn!(
                        "Failed to parse heartbeat tool arguments (attempt {}): {}. \
                         Feeding error back to LLM for correction.",
                        attempt + 1,
                        e
                    );

                    // Clone id before moving response in the retry branch
                    let tool_call_id = tool_call.id.clone();

                    // Add assistant response (carries tool_calls) then tool result
                    messages.push(response);
                    messages.push(Message::tool(&tool_call_id, error_msg));
                }
                Err(e) => {
                    // Final attempt failed - log and degrade to skip
                    error!("Failed to parse heartbeat tool arguments after {} retries: {}", MAX_PARSE_RETRIES, e);
                    return Ok(Some(Action::Skip));
                }
            }
        }

        // Should not reach here, but safety net
        Ok(Some(Action::Skip))
    }

    // ========== Public API ==========
}
