//! Builtin skills management.
//!
//! This module handles copying builtin skills from embedded crate resources
//! to the workspace at runtime.

use std::path::Path;
use std::{fs, io};

use anyhow::{Context, Result};
use include_dir::{Dir, include_dir};
use tracing::{debug, warn};

/// Embedded builtin skills directory (compiled into binary)
static BUILTIN_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/builtin");

/// Extract a directory from embedded resources to filesystem
fn extract_dir(dir: &Dir, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;

    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(subdir) => {
                let target_path = target.join(subdir.path().file_name().unwrap());
                extract_dir(subdir, &target_path)?;
            }
            include_dir::DirEntry::File(file) => {
                let target_path = target.join(file.path().file_name().unwrap());
                fs::write(&target_path, file.contents())?;
            }
        }
    }

    Ok(())
}

/// Initialize builtin skills in the specified directory
///
/// This function extracts builtin skills from embedded resources
/// to the target builtin-skills directory.
pub fn initialize_builtin_skills(builtin_dir: &Path) -> Result<()> {
    debug!(
        "Initializing builtin skills from embedded resources to {:?}",
        builtin_dir
    );

    // Create target directory if it doesn't exist
    fs::create_dir_all(builtin_dir).with_context(|| format!("Failed to create directory: {builtin_dir:?}"))?;

    // Extract embedded builtin directory to target
    extract_dir(&BUILTIN_DIR, builtin_dir)
        .with_context(|| format!("Failed to extract builtin skills to {builtin_dir:?}"))?;

    debug!("Successfully extracted builtin skills to {:?}", builtin_dir);

    Ok(())
}

/// Remove builtin-skills directory
pub fn remove_builtin_skills(builtin_dir: &Path) -> Result<()> {
    if builtin_dir.exists() {
        debug!("Removing existing builtin-skills directory: {:?}", builtin_dir);
        fs::remove_dir_all(builtin_dir).with_context(|| format!("Failed to remove directory: {builtin_dir:?}"))?;
        debug!("Successfully removed builtin-skills directory");
    }

    Ok(())
}

/// Check version and update builtin skills if needed
///
/// This function checks if the builtin-skills directory needs to be updated:
/// 1. If the directory doesn't exist, extract builtin skills and create VERSION file
/// 2. If VERSION file doesn't exist or version doesn't match, remove directory and re-extract
/// 3. If version matches, do nothing
pub fn ensure_builtin_skills(builtin_dir: &Path) -> Result<()> {
    let version_file = builtin_dir.join("VERSION");
    let current_version = crate::version::crate_version();

    // Check if we need to update
    let needs_update = if !builtin_dir.exists() {
        debug!("Builtin-skills directory doesn't exist, will initialize");
        true
    } else if !version_file.exists() {
        debug!("VERSION file doesn't exist, will reinitialize");
        true
    } else {
        match crate::version::read_version_file(&version_file) {
            Ok(stored_version) => {
                if !crate::version::version_matches(&stored_version) {
                    debug!(
                        "Version mismatch: stored={}, current={}, will reinitialize",
                        stored_version, current_version
                    );
                    true
                } else {
                    debug!("Version matches: {}, no update needed", current_version);
                    false
                }
            }
            Err(e) => {
                warn!("Failed to read VERSION file: {}, will reinitialize", e);
                true
            }
        }
    };

    if needs_update {
        // Remove existing directory if it exists
        if builtin_dir.exists() {
            remove_builtin_skills(builtin_dir)?;
        }

        // Extract builtin skills from embedded resources
        initialize_builtin_skills(builtin_dir)?;

        // Write VERSION file
        crate::version::write_version_file(&version_file, current_version)
            .with_context(|| format!("Failed to write VERSION file: {version_file:?}"))?;

        debug!(
            "Successfully initialized builtin skills with version {}",
            current_version
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests;
