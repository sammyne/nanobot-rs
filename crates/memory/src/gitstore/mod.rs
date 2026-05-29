//! Git-based version control for memory files.
//!
//! Uses `std::process::Command` to call the git CLI directly.
//! No git library dependencies.

use std::path::PathBuf;
use std::process::Command;

use crate::MemoryError;

/// .gitignore content: track only *.md files and .gitignore itself.
const GITIGNORE_CONTENT: &str = "\
# Track only markdown memory files
*
!*.md
!.gitignore
";

/// Git commit metadata.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit SHA hash.
    pub sha: String,
    /// Commit message.
    pub message: String,
    /// Commit timestamp.
    pub timestamp: String,
}

/// Git-based version control store for memory files.
///
/// Manages a git repository in the memory directory to track changes
/// to memory files (MEMORY.md, SOUL.md, USER.md).
pub struct GitStore {
    path: PathBuf,
}

impl GitStore {
    /// Initialize git repo in the memory directory.
    ///
    /// Checks that `git` command is available -- returns error if not.
    /// Creates .gitignore to only track memory files (`*.md`).
    /// Idempotent: if `.git` already exists, skip init.
    pub fn init(path: impl Into<PathBuf>) -> Result<Self, MemoryError> {
        let path = path.into();
        std::fs::create_dir_all(&path)?;

        let store = Self { path };
        store.check_git_available()?;

        if !store.path.join(".git").exists() {
            store.git(&["init"])?;
        }

        // Always ensure local config for commits (idempotent).
        store.git(&["config", "user.name", "nanobot"])?;
        store.git(&["config", "user.email", "nanobot@local"])?;

        // Write .gitignore (always, to ensure it stays current).
        std::fs::write(store.path.join(".gitignore"), GITIGNORE_CONTENT)?;

        // Commit .gitignore if it's new or changed; no-op otherwise.
        store.commit("Initialize memory repository")?;

        Ok(store)
    }

    /// Stage all tracked files and commit with the given message.
    ///
    /// No-op if there are no changes to commit.
    pub fn commit(&self, message: &str) -> Result<(), MemoryError> {
        self.git(&["add", "-A"])?;

        let status = self.git(&["status", "--porcelain"])?;
        if status.trim().is_empty() {
            return Ok(());
        }

        self.git(&["commit", "-m", message])?;
        Ok(())
    }

    /// Get recent commit log entries.
    pub fn log(&self, limit: usize) -> Result<Vec<CommitInfo>, MemoryError> {
        let limit_arg = format!("-{limit}");
        let output = self.git(&["log", "--format=%H%x1f%s%x1f%ai", &limit_arg])?;

        let entries = output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '\x1f').collect();
                if parts.len() == 3 {
                    Some(CommitInfo {
                        sha: parts[0].to_string(),
                        message: parts[1].to_string(),
                        timestamp: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(entries)
    }

    /// Get the diff for a specific commit.
    pub fn diff(&self, sha: &str) -> Result<String, MemoryError> {
        self.git(&["diff-tree", "--root", "--no-commit-id", "-p", sha])
    }

    /// Revert memory files to a specific commit's state, then commit the revert.
    pub fn revert(&self, sha: &str) -> Result<(), MemoryError> {
        // Remove all currently tracked files so that files added after `sha`
        // are properly deleted.
        let tracked = self.git(&["ls-files"])?;
        for file in tracked.lines().filter(|l| !l.trim().is_empty()) {
            let file_path = self.path.join(file);
            if file_path.exists() {
                std::fs::remove_file(&file_path)?;
            }
        }

        // Restore files from the target commit (stages them as well).
        self.git(&["checkout", sha, "--", "."])?;

        // Stage deletions of files that existed after `sha` but not at `sha`.
        self.git(&["add", "-A"])?;

        let message = format!("Revert to {sha}");
        self.commit(&message)
    }

    /// Check that git is available on the system.
    fn check_git_available(&self) -> Result<(), MemoryError> {
        Command::new("git").arg("--version").output().map_err(|_| {
            MemoryError::Io(std::io::Error::other(
                "git command not found: install git to enable memory version control",
            ))
        })?;
        Ok(())
    }

    /// Run a git command in the memory directory.
    fn git(&self, args: &[&str]) -> Result<String, MemoryError> {
        let output = Command::new("git").current_dir(&self.path).args(args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MemoryError::Io(std::io::Error::other(format!("git {} failed: {stderr}", args.join(" ")))));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests;
