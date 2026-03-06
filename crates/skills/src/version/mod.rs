//! Version management for builtin skills.
//!
//! This module handles version tracking to ensure builtin skills
//! stay synchronized with the crate version.

use std::path::Path;
use std::{fs, io};

/// Get the current crate version from Cargo.toml (compile-time constant)
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Read version from a VERSION file
pub fn read_version_file(path: &Path) -> io::Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}

/// Write version to a VERSION file
pub fn write_version_file(path: &Path, version: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, version)?;
    Ok(())
}

/// Check if the stored version matches the current crate version
pub fn version_matches(stored_version: &str) -> bool {
    stored_version == crate_version()
}

#[cfg(test)]
mod tests;
