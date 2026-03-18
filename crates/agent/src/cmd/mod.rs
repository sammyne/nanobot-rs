//! Command module for nanobot agent
//!
//! This module contains all command implementations for the agent system.
//! Each command is represented by a struct that implements the `Command` trait.

use crate::InboundMessage;

/// Base trait for all commands
///
/// All command structs must implement this trait to provide a unified interface
/// for command execution.
pub trait Command: Send + Sync {
    /// Execute the command
    ///
    /// # Arguments
    /// * `msg` - The inbound message that triggered this command
    /// * `session_key` - The session identifier
    ///
    /// # Returns
    /// * `Ok(String)` - Command executed successfully, returns response message
    /// * `Err(String)` - Command execution failed, returns error message
    async fn run(self, msg: InboundMessage, session_key: String) -> Result<String, String>;
}

// Command sub-modules
mod help;
mod new;

// Re-export common command types
pub use help::HelpCmd;
pub use new::NewCmd;

#[cfg(test)]
mod tests;
