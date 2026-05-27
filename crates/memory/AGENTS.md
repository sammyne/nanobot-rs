# memory crate

两层记忆：长期记忆（MEMORY.md）+ 历史日志（HISTORY.md），LLM 驱动整合。

## 关键类型

- **`MemoryStore`** -- 管理 `memory_file`（MEMORY.md）和 `history_file`（HISTORY.md）
  - `new(workspace, options)` -- 创建 memory/ 目录，存储 LLM 调用选项
  - `read_long_term() -> Result<String>` -- 读取 MEMORY.md
  - `write_long_term(content)` -- 写入 MEMORY.md
  - `append_history(entry)` -- 追加到 HISTORY.md
  - `get_memory_context() -> Result<String>` -- 格式化记忆内容供系统提示使用
  - `try_consolidate(messages, last_consolidated, provider, archive_all, memory_window)` -- 检查阈值并执行 LLM 驱动的记忆整合
  - `should_consolidate(message_count, last_consolidated, memory_window, archive_all) -> Option<usize>` -- 纯检查，不执行
  - `pick_consolidation_boundary(messages, last_consolidated, tokens_to_remove) -> Option<(usize, usize)>` -- 在 user 消息边界选择整合切割点
  - `raw_archive(messages)` -- 降级策略：原文转储到 HISTORY.md
- **`MemoryError`** (enum) -- `Io`, `LlmApi`, `ToolParse`, `NoToolCall`, `ConsecutiveFailure`
- **常量** -- `MAX_CONSOLIDATION_ROUNDS = 5`, `MAX_FAILURES_BEFORE_RAW_ARCHIVE = 3`

## 内部依赖

provider, tools
