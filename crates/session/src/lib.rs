//! Session management for nanobot.
//!
//! This crate provides session persistence and caching capabilities.
//!
//! # Overview
//!
//! - [`Session`] - A conversation session that stores messages in JSONL format
//! - [`SessionManager`] - Manages session persistence and caching
//! - [`SessionInfo`] - Basic information about a session for listing
//!
//! # Example
//!
//! ```rust,no_run
//! use nanobot_session::{Session, SessionManager};
//! use nanobot_provider::Message;
//! use std::path::PathBuf;
//!
//! // Create a session manager
//! let workspace = PathBuf::from("/path/to/workspace");
//! let manager = SessionManager::new(workspace);
//!
//! // Get or create a session
//! let mut session = manager.get_or_create("channel:chat_id");
//!
//! // Add messages
//! session.add_message(Message::user("Hello"));
//!
//! // Save session
//! manager.save(&session).expect("Failed to save session");
//!
//! // Get history for LLM
//! let mut history = Vec::new();
//! session.get_history(100, &mut history);
//! ```

mod manager;
mod session;

pub use manager::SessionManager;
// Re-export Message type for convenience
pub use nanobot_provider::Message;
pub use session::{Session, SessionInfo, SessionMetadata};
