# memory crate

两层记忆：长期记忆（MEMORY.md）+ 历史日志（history.jsonl），LLM 驱动整合。

## 关键类型

- **`MemoryStore`** -- 管理 `memory_file`（MEMORY.md）和 `history`（history.jsonl）
  - `new(workspace)` -- 创建 memory/ 目录和 History 实例
  - `history() -> &History` -- 返回历史存储的引用
  - `read_long_term() -> Result<String>` -- 读取 MEMORY.md
  - `write_long_term(content)` -- 写入 MEMORY.md
  - `append_history(entry) -> Result<u64>` -- 追加到 history.jsonl，返回分配的 cursor
  - `get_memory_context() -> Result<String>` -- 格式化记忆内容供系统提示使用
  - `pick_consolidation_boundary(messages, last_consolidated, tokens_to_remove) -> Option<(usize, usize)>` -- 在 user 消息边界选择整合切割点
  - `raw_archive(messages)` -- 降级策略：原文转储到 history.jsonl
- **`History`** -- history.jsonl 的 append-only JSONL 存储
  - `new(memory_dir)` -- 创建 History 实例
  - `append(content) -> Result<u64>` -- 追加条目，返回 cursor
  - `read_all() -> Result<Vec<HistoryEntry>>` -- 读取所有条目
  - `read_since(cursor) -> Result<Vec<HistoryEntry>>` -- 读取指定 cursor 之后的条目
  - `max_cursor() -> Result<u64>` -- 获取当前最大 cursor
- **`HistoryEntry`** -- `{ cursor: u64, timestamp: String, content: String }`
- **`consolidate_memory(memory, messages, last_consolidated, provider, archive_all, memory_window, options) -> Result<usize>`** -- 独立函数，LLM 纯文本摘要整合
- **`should_consolidate(message_count, last_consolidated, memory_window, archive_all) -> Option<usize>`** -- 独立函数，纯检查是否需要整合
- **`MemoryError`** (enum) -- `Io`, `LlmApi`
- **常量** -- `MAX_CONSOLIDATION_ROUNDS = 5`, `MAX_FAILURES_BEFORE_RAW_ARCHIVE = 3`

## 内部依赖

provider
