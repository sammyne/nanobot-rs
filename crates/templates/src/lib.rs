//! Workspace templates for nanobot
//!
//! This crate provides template files for workspace initialization.
//! Templates are embedded in the binary at compile time using `include_dir`.

use include_dir::{include_dir, Dir};

/// Embedded templates directory
static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Get the content of a template file
///
/// # Arguments
/// * `path` - Path relative to the templates directory (e.g., "USER.md", "memory/MEMORY.md")
///
/// # Returns
/// The template content as a string, or None if the file doesn't exist
pub fn get_template(path: &str) -> Option<&'static str> {
    TEMPLATES_DIR.get_file(path).and_then(|file| file.contents_utf8())
}

/// Get the USER.md template content
pub fn user_template() -> &'static str {
    get_template("USER.md").expect("USER.md template not found")
}

/// Get the AGENTS.md template content
pub fn agents_template() -> &'static str {
    get_template("AGENTS.md").expect("AGENTS.md template not found")
}

/// Get the SOUL.md template content
pub fn soul_template() -> &'static str {
    get_template("SOUL.md").expect("SOUL.md template not found")
}

/// Get the TOOLS.md template content
pub fn tools_template() -> &'static str {
    get_template("TOOLS.md").expect("TOOLS.md template not found")
}

/// Get the MEMORY.md template content (in memory subdirectory)
pub fn memory_template() -> &'static str {
    get_template("memory/MEMORY.md").expect("memory/MEMORY.md template not found")
}

/// Get the HEARTBEAT.md template content
pub fn heartbeat_template() -> &'static str {
    get_template("HEARTBEAT.md").expect("HEARTBEAT.md template not found")
}

#[cfg(test)]
mod tests;
