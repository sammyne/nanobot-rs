//! Post-run evaluation for background tasks (heartbeat & cron).
//!
//! After the agent executes a background task, this module makes a lightweight
//! LLM call to decide whether the result warrants notifying the user.

use nanobot_provider::{Message, Options, Provider};
use nanobot_tools::ToolDefinition;
use schemars::JsonSchema;
use serde::Deserialize;
use tracing::{info, warn};

/// Arguments returned by the `evaluate_notification` tool call.
#[derive(Debug, Deserialize, JsonSchema)]
struct EvaluateNotificationArgs {
    /// `true` = result contains actionable/important info the user should see;
    /// `false` = routine or empty, safe to suppress.
    should_notify: bool,

    /// One-sentence reason for the decision.
    #[serde(default)]
    reason: Option<String>,
}

/// Tool definition for the evaluation LLM call.
static EVALUATE_TOOL: std::sync::LazyLock<ToolDefinition> = std::sync::LazyLock::new(|| ToolDefinition {
    name: "evaluate_notification".to_string(),
    description: "Decide whether the user should be notified about this background task result.".to_string(),
    parameters: schemars::schema_for!(EvaluateNotificationArgs).to_value(),
});

const SYSTEM_PROMPT: &str = "\
You are a notification gate for a background agent. \
You will be given the original task and the agent's response. \
Call the evaluate_notification tool to decide whether the user \
should be notified.\n\n\
Notify when the response contains actionable information, errors, \
completed deliverables, or anything the user explicitly asked to \
be reminded about.\n\n\
Suppress when the response is a routine status check with nothing \
new, a confirmation that everything is normal, or essentially empty.";

/// Decide whether a background-task result should be delivered to the user.
///
/// Uses a lightweight tool-call LLM request. Falls back to `true` (notify)
/// on any failure so that important messages are never silently dropped.
pub async fn evaluate_response<P: Provider>(provider: &P, response: &str, task_context: &str) -> bool {
    let mut evaluator = provider.clone();
    evaluator.bind_tools(vec![EVALUATE_TOOL.clone()]);

    let messages = vec![
        Message::system(SYSTEM_PROMPT),
        Message::user(format!("## Original task\n{task_context}\n\n## Agent response\n{response}")),
    ];

    let options = Options { max_tokens: 256, temperature: 0.0, ..Options::default() };

    let result = evaluator.chat(&messages, &options).await;

    match result {
        Ok(msg) => {
            let Some(tool_call) = msg.tool_calls().first() else {
                warn!("evaluate_response: no tool call returned, defaulting to notify");
                return true;
            };

            if tool_call.name != "evaluate_notification" {
                warn!("evaluate_response: unexpected tool name '{}', defaulting to notify", tool_call.name);
                return true;
            }

            match tool_call.parse_arguments::<EvaluateNotificationArgs>() {
                Ok(args) => {
                    info!(
                        "evaluate_response: should_notify={}, reason={}",
                        args.should_notify,
                        args.reason.as_deref().unwrap_or("")
                    );
                    args.should_notify
                }
                Err(e) => {
                    warn!("evaluate_response: failed to parse args: {e}, defaulting to notify");
                    true
                }
            }
        }
        Err(e) => {
            warn!("evaluate_response: provider error: {e}, defaulting to notify");
            true
        }
    }
}

#[cfg(test)]
mod tests;
