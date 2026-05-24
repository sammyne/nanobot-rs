# memory crate

两层记忆：长期记忆（MEMORY.md）+ 历史日志（HISTORY.md），LLM 驱动整合。

## 关键类型

- **`MemoryStore`** -- 管理 `memory_file`（MEMORY.md）和 `history_file`（HISTORY.md）
  - `new(workspace)` -- 创建 memory/ 目录
  - `read_long_term() -> Result<String>` -- 读取 MEMORY.md
  - `write_long_term(content)` -- 写入 MEMORY.md
  - `append_history(entry)` -- 追加到 HISTORY.md
  - `get_memory_context() -> Result<String>` -- 格式化记忆内容供系统提示使用
  - `try_consolidate(messages, last_consolidated, provider, archive_all, memory_window)` -- 检查阈值并执行 LLM 驱动的记忆整合
  - `should_consolidate(message_count, last_consolidated, memory_window, archive_all) -> Option<usize>` -- 纯检查，不执行
- **`MemoryError`** (enum) -- `Io`, `LlmApi`, `ToolParse`, `NoToolCall`

## 内部依赖

provider, tools
