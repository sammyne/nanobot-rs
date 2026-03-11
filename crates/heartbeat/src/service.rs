//! Heartbeat service implementation

use std::path::PathBuf;
use std::sync::Arc;

use nanobot_provider::Provider;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::callback::{OnExecuteCallback, OnNotifyCallback};
use crate::config::HeartbeatConfig;
use crate::error::HeartbeatError;

/// Heartbeat tool definition for LLM decision making
static HEARTBEAT_TOOL: std::sync::LazyLock<nanobot_tools::ToolDefinition> =
    std::sync::LazyLock::new(|| nanobot_tools::ToolDefinition {
        name: "heartbeat".to_string(),
        description: "Decide whether to execute tasks based on HEARTBEAT.md content".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["skip", "run"],
                    "description": "Action to take: 'skip' to skip execution, 'run' to execute tasks"
                },
                "tasks": {
                    "type": "string",
                    "description": "Natural language summary of active tasks to execute (required when action='run')"
                }
            },
            "required": ["action"]
        }),
    });

/// Heartbeat service for periodic task checking
pub struct HeartbeatService<P>
where
    P: Provider + Send + Sync + Clone + 'static,
{
    /// Workspace path
    workspace_path: PathBuf,
    /// LLM provider
    provider: P,
    /// Heartbeat configuration
    config: HeartbeatConfig,
    /// Execute callback
    on_execute: Arc<RwLock<Option<Arc<dyn OnExecuteCallback>>>>,
    /// Notify callback
    on_notify: Arc<RwLock<Option<Arc<dyn OnNotifyCallback>>>>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Timer task handle
    timer_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl<P> HeartbeatService<P>
where
    P: Provider + Send + Sync + Clone + 'static,
{
    /// Create a new heartbeat service
    ///
    /// # Arguments
    ///
    /// * `workspace_path` - Path to the workspace directory
    /// * `provider` - LLM provider for decision making
    /// * `config` - Heartbeat configuration
    /// * `on_execute` - Optional callback for executing tasks
    /// * `on_notify` - Optional callback for notifying task results
    ///
    /// # Returns
    ///
    /// A new `HeartbeatService` instance with heartbeat tool bound to provider
    pub fn new(
        workspace_path: PathBuf,
        provider: P,
        config: HeartbeatConfig,
        on_execute: Option<Arc<dyn OnExecuteCallback>>,
        on_notify: Option<Arc<dyn OnNotifyCallback>>,
    ) -> Self {
        // Bind heartbeat tool to provider once during initialization
        let mut provider = provider;
        provider.bind_tools(vec![HEARTBEAT_TOOL.clone()]);

        Self {
            workspace_path,
            provider,
            config,
            on_execute: Arc::new(RwLock::new(on_execute)),
            on_notify: Arc::new(RwLock::new(on_notify)),
            running: Arc::new(RwLock::new(false)),
            timer_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the heartbeat service
    pub async fn start(self: Arc<Self>) -> Result<(), HeartbeatError> {
        // Check if disabled
        if !self.config.enabled {
            info!("Heartbeat disabled");
            return Err(HeartbeatError::Disabled);
        }

        // Check if already running
        let mut running = self.running.write().await;
        if *running {
            info!("Heartbeat already running");
            return Err(HeartbeatError::AlreadyRunning);
        }

        // Mark as running
        *running = true;
        drop(running);

        // Start the timer in background (non-blocking)
        let arc_clone = Arc::clone(&self);
        let task = tokio::spawn(async move {
            if let Err(e) = arc_clone.run_loop().await {
                error!("Heartbeat loop error: {:?}", e);
            }
        });

        // Store task handle
        let mut timer_task = self.timer_task.write().await;
        *timer_task = Some(task);
        drop(timer_task);

        info!("Heartbeat started (every {}s)", self.config.interval_seconds);
        Ok(())
    }

    /// Stop the heartbeat service
    pub async fn stop(&self) {
        // Set running flag to false
        *self.running.write().await = false;

        // Abort task
        let mut timer_task = self.timer_task.write().await;
        if let Some(task) = timer_task.take() {
            task.abort();
        }

        info!("Heartbeat service stopped");
    }

    /// Check if the service is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Main heartbeat loop (equivalent to Python's _run_loop)
    async fn run_loop(&self) -> Result<(), HeartbeatError> {
        let running = Arc::clone(&self.running);
        let interval = self.config.interval_seconds;

        // Main loop
        while *running.read().await {
            // Sleep first, then check (matches Python's asyncio.sleep pattern)
            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;

            // Check if still running after sleep
            if !*running.read().await {
                break;
            }

            // Execute a single heartbeat tick
            if let Err(e) = self.tick().await {
                error!("Heartbeat error: {:?}", e);
            }
        }

        Ok(())
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
    async fn tick(&self) -> Result<Option<String>, HeartbeatError> {
        // Phase 1: Decide
        let (action, tasks) = match self.decide().await? {
            Some(decision) => decision,
            None => {
                info!("HEARTBEAT.md not found or empty, skipping");
                return Ok(None);
            }
        };

        // Check action
        match action.as_str() {
            "run" => {
                info!("Action: run");
                let tasks = tasks.as_deref().unwrap_or("");

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

                let result = callback.execute(tasks).await.map_err(HeartbeatError::ExecuteError)?;

                // Check result
                if result.trim().is_empty() {
                    info!("Execute callback returned empty result");
                    return Ok(None);
                }

                // Notify callback if configured
                let on_notify = self.on_notify.read().await;
                if let Some(notify_callback) = on_notify.as_ref() {
                    notify_callback
                        .notify(&result)
                        .await
                        .map_err(HeartbeatError::NotifyError)?;
                }

                Ok(Some(result))
            }
            "skip" => {
                info!("Action: skip");
                Ok(None)
            }
            _ => {
                error!("Unknown action: {}", action);
                Ok(None)
            }
        }
    }

    /// Phase 1: Check heartbeat - Decision phase
    ///
    /// Reads HEARTBEAT.md and asks LLM to decide if tasks need execution
    ///
    /// # Returns
    ///
    /// - `Ok(Some((action, tasks)))` - LLM decision with action and optional tasks
    /// - `Ok(None)` - HEARTBEAT.md not found or empty
    /// - `Err(HeartbeatError)` - Error occurred during check
    async fn decide(&self) -> Result<Option<(String, Option<String>)>, HeartbeatError> {
        // Read HEARTBEAT.md file
        let heartbeat_path = self.heartbeat_file();
        let content = match tokio::fs::read_to_string(&heartbeat_path).await {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("HEARTBEAT.md not found, skipping heartbeat check");
                return Ok(None);
            }
            Err(e) => {
                error!("Failed to read HEARTBEAT.md: {}", e);
                return Err(HeartbeatError::FileReadError(e));
            }
        };

        // Check if content is empty or only whitespace
        if content.trim().is_empty() {
            info!("HEARTBEAT.md is empty, skipping heartbeat check");
            return Ok(None);
        }

        // Prepare messages for LLM
        let messages = vec![
            nanobot_provider::Message::system(
                "You are a heartbeat agent. Call the heartbeat tool to report your decision.",
            ),
            nanobot_provider::Message::user(format!(
                "Review the following HEARTBEAT.md and decide whether there are active tasks.\n\n{content}"
            )),
        ];

        // Call provider (tools are already bound during initialization)
        let response = self
            .provider
            .chat(&messages)
            .await
            .map_err(HeartbeatError::ProviderError)?;

        // Parse response - Message may contain tool_calls
        let tool_calls = response.tool_calls();
        if tool_calls.is_empty() {
            info!("LLM did not return a tool call, treating as skip");
            return Ok(Some(("skip".to_string(), None)));
        }

        // Extract tool call
        let tool_call = response
            .tool_calls()
            .first()
            .ok_or_else(|| HeartbeatError::ParseError("No tool call in response".to_string()))?;

        if tool_call.name != "heartbeat" {
            error!("Unexpected tool name: {}", tool_call.name);
            return Ok(Some(("skip".to_string(), None)));
        }

        // Parse arguments
        let args: serde_json::Value = tool_call
            .parse_arguments()
            .map_err(|e| HeartbeatError::ParseError(format!("Failed to parse tool arguments: {e}")))?;

        let action = args
            .get("action")
            .and_then(|v: &serde_json::Value| v.as_str())
            .unwrap_or("skip")
            .to_string();

        let tasks = args
            .get("tasks")
            .and_then(|v: &serde_json::Value| v.as_str())
            .map(|s: &str| s.to_string());

        Ok(Some((action, tasks)))
    }

    // ========== Public API ==========

    /// Get the path to the heartbeat file (equivalent to Python's heartbeat_file property)
    pub fn heartbeat_file(&self) -> PathBuf {
        self.workspace_path.join("HEARTBEAT.md")
    }

    /// Manually trigger a heartbeat check (equivalent to Python's trigger_now)
    ///
    /// Returns the execution result if action="run" and execute callback is configured,
    /// otherwise returns None.
    pub async fn trigger_now(&self) -> Option<String> {
        info!("Manual heartbeat trigger");
        self.tick().await.ok().flatten()
    }

    /// Get service status
    pub async fn status(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.config.enabled,
            "running": *self.running.read().await,
            "interval_seconds": self.config.interval_seconds,
            "workspace_path": self.workspace_path.to_string_lossy().to_string(),
        })
    }
}
