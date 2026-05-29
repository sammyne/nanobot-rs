//! history.jsonl 读写模块
//!
//! 结构化历史条目的 append-only JSONL 存储，支持 cursor 追踪和 compaction。

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::MemoryError;

/// 最大历史条目数，超过时触发 compaction
const MAX_ENTRIES: usize = 1000;

/// 历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 自增游标
    pub cursor: u64,
    /// 时间戳（ISO 8601）
    pub timestamp: String,
    /// 摘要内容
    pub content: String,
}

/// history.jsonl 存储
pub struct History {
    path: PathBuf,
}

impl History {
    /// 创建 History 实例
    pub fn new(memory_dir: &Path) -> Self {
        Self { path: memory_dir.join("history.jsonl") }
    }

    /// 读取所有条目
    pub fn read_all(&self) -> Result<Vec<HistoryEntry>, MemoryError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.path)?;
        let entries: Vec<HistoryEntry> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(entries)
    }

    /// 读取指定 cursor 之后的条目
    pub fn read_since(&self, cursor: u64) -> Result<Vec<HistoryEntry>, MemoryError> {
        Ok(self.read_all()?.into_iter().filter(|e| e.cursor > cursor).collect())
    }

    /// 获取当前最大 cursor 值
    pub fn max_cursor(&self) -> Result<u64, MemoryError> {
        Ok(self.read_all()?.last().map(|e| e.cursor).unwrap_or(0))
    }

    /// 追加一条历史条目，返回分配的 cursor
    pub fn append(&self, content: &str) -> Result<u64, MemoryError> {
        let cursor = self.max_cursor()? + 1;
        let entry = HistoryEntry { cursor, timestamp: chrono::Utc::now().to_rfc3339(), content: content.to_string() };

        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&self.path)?;
        let json = serde_json::to_string(&entry).map_err(|e| MemoryError::Io(std::io::Error::other(e)))?;
        writeln!(file, "{json}")?;

        // compaction
        self.compact_if_needed()?;

        Ok(cursor)
    }

    /// 超过 MAX_ENTRIES 时截断旧条目
    fn compact_if_needed(&self) -> Result<(), MemoryError> {
        let entries = self.read_all()?;
        if entries.len() <= MAX_ENTRIES {
            return Ok(());
        }

        // 保留后半部分
        let keep = &entries[entries.len() - MAX_ENTRIES / 2..];
        let content: String =
            keep.iter().filter_map(|e| serde_json::to_string(e).ok()).map(|s| format!("{s}\n")).collect();

        std::fs::write(&self.path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
