# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/session/src/session.rs` | 修改 | 回退 `get_history()` 签名，移除 `max_tokens` 参数 |
| `crates/session/tests/session.rs` | 修改 | 回退测试，移除 token 预算测试 |
| `crates/session/src/lib.rs` | 修改 | 回退 doc comment |
| `crates/agent/src/loop/mod.rs` | 修改 | 回退 ReAct token 检查；重写 `try_consolidate()` 为 token-based 多轮整合 |
| `crates/agent/src/loop/tests.rs` | 修改 | 回退 `get_history` 调用签名；适配新 `try_consolidate` |
| `crates/memory/src/store.rs` | 修改 | 新增 `pick_consolidation_boundary()`；新增 `raw_archive()` 降级；重构 `consolidate_internal` 支持 chunk 整合 |
| `crates/memory/src/error.rs` | 修改 | 新增 `ConsecutiveFailure` 错误变体 |

## 任务列表

### 1. 回退 get_history() 签名

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/session/src/session.rs`, `crates/session/tests/session.rs`, `crates/session/src/lib.rs`, `crates/agent/src/loop/mod.rs`, `crates/agent/src/loop/tests.rs`
- 验收标准: `get_history()` 恢复为 `get_history(&self, max_messages: usize, buf: &mut Vec<Message>) -> usize`，所有调用方同步回退，测试通过
- 风险/注意点: 需要同步回退所有调用方；PR #117 新增的 `session_get_history_token_budget` 测试需要删除
- 信心评估: 5
- 步骤:
  - [ ] 在 `session.rs` 中将 `get_history` 签名从 `(max_messages, max_tokens, buf)` 恢复为 `(max_messages, buf)`，移除 token 预算扫描逻辑
  - [ ] 在 `session/tests/session.rs` 中将所有 `get_history(N, 0, &mut buf)` 恢复为 `get_history(N, &mut buf)`，删除 `session_get_history_token_budget` 测试
  - [ ] 在 `session/src/lib.rs` doc comment 中恢复 `session.get_history(100, &mut history)`
  - [ ] 在 `agent/src/loop/mod.rs` 中将 `session.get_history(self.config.memory_window, self.config.max_input_tokens, &mut history)` 恢复为 `session.get_history(self.config.memory_window, &mut history)`
  - [ ] 在 `agent/src/loop/tests.rs` 中回退所有受影响的 `get_history` 调用
  - [ ] 运行 `cargo test -p nanobot-session -p nanobot-agent` 验证通过

### 2. 回退 ReAct 循环内 token 检查

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `re_act()` 方法中不再有 `turn_start`、`max_input_tokens`、`messages.remove(1)` 相关逻辑
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 在 `re_act()` 方法中移除 `let max_input_tokens = ...` 和 `let mut turn_start = ...` 声明
  - [ ] 移除 `while turn_start > 1 { ... }` token 检查循环
  - [ ] 运行 `cargo test -p nanobot-agent` 验证通过

### 3. MemoryStore 新增 pick_consolidation_boundary 和 raw_archive

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/memory/src/store.rs`, `crates/memory/src/error.rs`
- 验收标准: `pick_consolidation_boundary()` 从 `last_consolidated` 向前扫描，在 user 消息边界返回 `(end_idx, removed_tokens)`；`raw_archive()` 将消息原文转储到 HISTORY.md；`MemoryError` 新增 `ConsecutiveFailure` 变体
- 风险/注意点: 切割点必须在 user 消息边界，不能拆开 assistant + tool_result 对
- 信心评估: 4
- 步骤:
  - [ ] 在 `MemoryStore` 中新增常量 `const MAX_CONSOLIDATION_ROUNDS: usize = 5` 和 `const MAX_FAILURES_BEFORE_RAW_ARCHIVE: usize = 3`
  - [ ] 新增 `pub fn pick_consolidation_boundary(messages: &[Message], last_consolidated: usize, tokens_to_remove: usize) -> Option<(usize, usize)>` 方法：从 `last_consolidated` 向前扫描，累加每条消息的 `token_len()`，在 `idx > last_consolidated && message.role() == "user"` 时记录边界 `(idx, removed_tokens)`，当 `removed_tokens >= tokens_to_remove` 时返回
  - [ ] 新增 `pub fn raw_archive(&self, messages: &[Message]) -> Result<(), MemoryError>` 方法：将消息格式化为 `[YYYY-MM-DD HH:MM] [RAW] N messages\n{formatted_lines}` 并 append 到 HISTORY.md
  - [ ] 在 `MemoryError` 中新增 `#[error("连续整合失败")]` `ConsecutiveFailure` 变体
  - [ ] 运行 `cargo test -p nanobot-memory` 验证通过

### 4. 重写 try_consolidate 为 token-based 多轮整合

- 优先级: P0
- 依赖项: 1, 2, 3
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `try_consolidate()` 估算完整 prompt token，超过 `max_input_tokens` 时触发多轮整合（最多 5 轮），每轮在 user 边界切割一批消息，调用 LLM 做摘要，推进 `last_consolidated`，直到 prompt token 降到 `max_input_tokens / 2` 以下；连续 3 次 LLM 失败后降级为原文转储
- 风险/注意点: 需要在 `try_consolidate` 中估算完整 prompt token，需要访问 `ContextBuilder::build_messages` 或等效方法；多轮整合中每轮需要重新估算 token
- 信心评估: 3
- 步骤:
  - [ ] 修改 `try_consolidate()` 签名，新增 `max_input_tokens: usize` 参数，在 `process_message()` 调用处传入 `self.config.max_input_tokens`
  - [ ] 替换触发条件：从 `messages.len() - last_consolidated >= memory_window` 改为估算未整合消息总 token（`session.messages[last_consolidated..].iter().map(|m| m.token_len()).sum::<usize>()`），当超过 `max_input_tokens` 时触发
  - [ ] 设定目标 `target = max_input_tokens / 2`
  - [ ] 实现多轮循环（最多 `MAX_CONSOLIDATION_ROUNDS = 5` 轮）：
    - 重新估算未整合消息 token
    - 如果 `estimated <= target`，退出循环
    - 调用 `memory.pick_consolidation_boundary(messages, last_consolidated, estimated - target)` 获取切割点
    - 如果无切割点，退出循环
    - 提取 chunk `messages[last_consolidated..end_idx]`
    - 调用 `memory.consolidate_internal(chunk, ...)` 做 LLM 摘要
    - 成功：推进 `session.last_consolidated = end_idx`
    - 失败：递增失败计数，如果达到 3 次，调用 `memory.raw_archive(chunk)` 降级，推进指针，重置计数
  - [ ] 更新 `process_message()` 中 `try_consolidate` 的调用，传入 `self.config.max_input_tokens`
  - [ ] 运行 `cargo test -p nanobot-agent` 验证通过

### 5. 更新 AGENTS.md 文档

- 优先级: P1
- 依赖项: 4
- 涉及文件: `crates/memory/AGENTS.md`, `crates/agent/AGENTS.md`, `crates/session/AGENTS.md`
- 验收标准: 文档反映 token-based 多轮整合、pick_consolidation_boundary、raw_archive 降级
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `crates/memory/AGENTS.md`：新增 `pick_consolidation_boundary`、`raw_archive`、`MAX_CONSOLIDATION_ROUNDS`、`MAX_FAILURES_BEFORE_RAW_ARCHIVE` 描述
  - [ ] `crates/agent/AGENTS.md`：更新 `try_consolidate` 描述为 token-based 多轮整合
  - [ ] `crates/session/AGENTS.md`：回退 `get_history` 签名描述（移除 max_tokens）

## 实现建议

- 任务 1 和 2 是回退操作，可并行执行
- 任务 3 是新增功能，不依赖回退，也可并行
- 任务 4 依赖 1、2、3 全部完成
- `pick_consolidation_boundary` 作为 `MemoryStore` 的关联函数（不需要 `&self`），方便测试
- 多轮整合中的失败计数是局部变量，不需要持久化（与 Python 的 `_consecutive_failures` 实例变量不同，因为 Rust 的 `try_consolidate` 是独立函数）
- 估算 prompt token 时，简化为只估算未整合消息的 token 总量（不构建完整 prompt），因为系统提示和工具定义的大小相对稳定
