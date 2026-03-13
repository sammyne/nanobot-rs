//! Dependency checking utilities.

use std::env;
use std::process::Command;

use crate::models::Requires;

/// Checks if a CLI tool is available in PATH.
pub fn is_bin_available(bin: &str) -> bool {
    Command::new("which").arg(bin).output().map(|o| o.status.success()).unwrap_or(false)
}

/// Checks if an environment variable is set.
pub fn is_env_set(var: &str) -> bool {
    env::var(var).is_ok()
}

/// Checks if all requirements are satisfied.
pub fn check_requirements(requires: &Requires) -> bool {
    for bin in &requires.bins {
        if !is_bin_available(bin) {
            return false;
        }
    }
    for var in &requires.env {
        if !is_env_set(var) {
            return false;
        }
    }
    true
}

/// Returns a description of missing requirements.
pub fn get_missing_requirements(requires: &Requires) -> Vec<String> {
    let mut missing = Vec::new();

    for bin in &requires.bins {
        if !is_bin_available(bin) {
            missing.push(format!("CLI: {bin}"));
        }
    }

    for var in &requires.env {
        if !is_env_set(var) {
            missing.push(format!("ENV: {var}"));
        }
    }

    missing
}

#[cfg(test)]
mod tests;
