//! Session manager for persistence and caching.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use nanobot_provider::Message;
use parking_lot::RwLock;
use serde_json::Value;
use tracing::{info, warn};

use crate::{Session, SessionInfo, SessionMetadata};

/// Manager for conversation sessions.
///
/// Sessions are stored as JSONL files in the sessions directory.
/// The manager also maintains an in-memory cache for faster access.
pub struct SessionManager {
    /// Workspace root directory
    workspace: PathBuf,
    /// Directory for session files
    sessions_dir: PathBuf,
    /// In-memory cache for sessions
    cache: RwLock<HashMap<String, Session>>,
}

impl SessionManager {
    /// Create a new SessionManager.
    ///
    /// # Arguments
    /// * `workspace` - The workspace root directory
    ///
    /// # Returns
    /// A new SessionManager instance
    pub fn new(workspace: PathBuf) -> Self {
        let sessions_dir = workspace.join("sessions");
        if !sessions_dir.exists()
            && let Err(e) = fs::create_dir_all(&sessions_dir)
        {
            warn!("Failed to create sessions directory: {}", e);
        }
        Self { workspace, sessions_dir, cache: RwLock::new(HashMap::new()) }
    }

    /// Get the file path for a session.
    fn get_session_path(&self, key: &str) -> PathBuf {
        let safe_key = key.replace(':', "_").replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
        self.sessions_dir.join(format!("{safe_key}.jsonl"))
    }

    /// Get an existing session or create a new one.
    ///
    /// # Arguments
    /// * `key` - Session key (usually channel:chat_id)
    ///
    /// # Returns
    /// The session
    pub fn get_or_create(&self, key: &str) -> Session {
        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(session) = cache.get(key) {
                return session.clone();
            }
        }

        // Try to load from disk
        if let Some(session) = self.load(key) {
            let mut cache = self.cache.write();
            cache.insert(key.to_string(), session.clone());
            return session;
        }

        // Create new session
        let session = Session::new(key);
        let mut cache = self.cache.write();
        cache.insert(key.to_string(), session.clone());
        session
    }

    /// Load a session from disk.
    ///
    /// # Arguments
    /// * `key` - Session key
    ///
    /// # Returns
    /// The loaded session, or None if it doesn't exist or is corrupted
    fn load(&self, key: &str) -> Option<Session> {
        let path = self.get_session_path(key);
        if !path.exists() {
            return None;
        }

        let file = File::open(&path).ok()?;
        let reader = BufReader::new(file);

        let mut messages: Vec<Message> = Vec::new();
        let mut metadata = HashMap::new();
        let mut created_at = None;
        let mut last_consolidated = 0;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    warn!("Failed to read line from session file: {}", e);
                    continue;
                }
            };

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(line) {
                Ok(data) => {
                    if data.get("_type").and_then(|t| t.as_str()) == Some("metadata") {
                        // Parse metadata line
                        if let Some(m) = data.get("metadata").and_then(|m| m.as_object()) {
                            for (k, v) in m {
                                metadata.insert(k.clone(), v.clone());
                            }
                        }
                        if let Some(ca) = data.get("created_at").and_then(|c| c.as_str()) {
                            created_at = chrono::DateTime::parse_from_rfc3339(ca).map(|dt| dt.with_timezone(&Utc)).ok();
                        }
                        last_consolidated =
                            data.get("last_consolidated").and_then(|lc| lc.as_u64()).unwrap_or(0) as usize;
                    } else {
                        // Parse as Message
                        match serde_json::from_value(data) {
                            Ok(msg) => messages.push(msg),
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse JSON line: {}", e);
                    continue;
                }
            }
        }

        Some(Session {
            key: key.to_string(),
            messages,
            created_at: created_at.unwrap_or_else(Utc::now),
            updated_at: Utc::now(),
            metadata,
            last_consolidated,
        })
    }

    /// Save a session to disk.
    ///
    /// This also updates the in-memory cache.
    ///
    /// # Arguments
    /// * `session` - The session to save
    ///
    /// # Returns
    /// Ok(()) on success, Err on failure
    pub fn save(&self, session: &Session) -> Result<()> {
        let path = self.get_session_path(&session.key);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&path)?;

        // Write metadata line
        let metadata = SessionMetadata::from(session);
        let metadata_line = serde_json::to_string(&metadata)?;
        writeln!(file, "{metadata_line}")?;

        // Write messages
        for msg in &session.messages {
            let msg_line = serde_json::to_string(msg)?;
            writeln!(file, "{msg_line}")?;
        }

        // Update cache
        {
            let mut cache = self.cache.write();
            cache.insert(session.key.clone(), session.clone());
        }

        info!("Saved session {} to {}", session.key, path.display());
        Ok(())
    }

    /// Remove a session from the in-memory cache.
    ///
    /// # Arguments
    /// * `key` - Session key to invalidate
    pub fn invalidate(&self, key: &str) {
        let mut cache = self.cache.write();
        cache.remove(key);
    }

    /// List all sessions.
    ///
    /// # Returns
    /// List of session info, sorted by updated_at in descending order
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let mut sessions = Vec::new();

        let entries = match fs::read_dir(&self.sessions_dir) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read sessions directory: {}", e);
                return sessions;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false)
                && let Some(info) = self.read_session_info(&path)
            {
                sessions.push(info);
            }
        }

        // Sort by updated_at descending
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions
    }

    /// Read session info from a JSONL file.
    fn read_session_info(&self, path: &PathBuf) -> Option<SessionInfo> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);

        let first_line = {
            let mut line = String::new();
            reader.read_line(&mut line).ok()?;
            line.trim().to_string()
        };

        if first_line.is_empty() {
            return None;
        }

        let data: Value = serde_json::from_str(&first_line).ok()?;
        if data.get("_type").and_then(|t| t.as_str()) != Some("metadata") {
            return None;
        }

        let key = data
            .get("key")
            .and_then(|k| k.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| path.file_stem().and_then(|s| s.to_str()).unwrap_or("").replace('_', ":"));

        let created_at = data
            .get("created_at")
            .and_then(|c| c.as_str())
            .and_then(|c| chrono::DateTime::parse_from_rfc3339(c).ok())
            .map(|dt| dt.with_timezone(&Utc))?;

        let updated_at = data
            .get("updated_at")
            .and_then(|c| c.as_str())
            .and_then(|c| chrono::DateTime::parse_from_rfc3339(c).ok())
            .map(|dt| dt.with_timezone(&Utc))?;

        Some(SessionInfo { key, created_at, updated_at, path: path.to_string_lossy().to_string() })
    }

    /// Get the workspace directory.
    pub fn workspace(&self) -> &PathBuf {
        &self.workspace
    }

    /// Get the sessions directory.
    pub fn sessions_dir(&self) -> &PathBuf {
        &self.sessions_dir
    }
}
