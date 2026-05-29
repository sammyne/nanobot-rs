# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `docs/dreaming.md` | 新增 | Dream 记忆系统调研文档 |
| `crates/config/src/schema/agent.rs` | 修改 | 新增 DreamConfig |
| `crates/memory/src/store.rs` | 修改 | MemoryStore 解耦 LLM，history.jsonl + cursor |
| `crates/memory/src/history.rs` | 新增 | history.jsonl 读写 + compaction |
| `crates/memory/src/consolidator.rs` | 新增 | 纯文本 LLM 摘要（替代 tool_choice 方式） |
| `crates/memory/src/error.rs` | 修改 | 移除 NoToolCall/ToolParse 错误变体 |
| `crates/memory/src/gitstore.rs` | 新增 | Git 版本控制（通过 git CLI） |
| `crates/memory/src/dream.rs` | 新增 | Dream 两阶段处理器 |
| `crates/agent/src/loop/mod.rs` | 修改 | try_consolidate 适配新 API |
| `crates/agent/src/cmd/dream/mod.rs` | 新增 | /dream、/dream-log、/dream-restore 命令 |
| `crates/cron/src/service/mod.rs` | 修改 | 新增 register_system_job |
| `crates/context/src/builder/mod.rs` | 修改 | 系统提示中 HISTORY.md → history.jsonl |

## 任务列表

### 0. 调研文档

- 优先级: P0
- 依赖项: 无
- 涉及文件: `docs/dreaming.md`
- 验收标准: 文档覆盖设计理念、架构、文件布局、两阶段流程、GitStore、命令、配置
- 信心评估: 5
- 步骤:
  - [ ] 编写 `docs/dreaming.md`，内容包括：设计理念、架构图、文件布局、Stage 1 (Consolidator)、Stage 2 (Dream Phase 1/2)、GitStore、命令、配置
  - [ ] 参考上游 `docs/MEMORY.md`，适配 Rust 版实现差异

### 1. DreamConfig

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/agent.rs`
- 验收标准: `AgentDefaults` 包含 `dream: Option<DreamConfig>`，`cargo check` 通过
- 信心评估: 5
- 步骤:
  - [ ] 定义 `DreamConfig` 结构体：`cron: String`（默认 `"0 */2 * * *"`）、`model: Option<String>`、`max_batch_size: usize`（默认 20）、`max_iterations: usize`（默认 10）
  - [ ] `AgentDefaults` 新增 `#[serde(default)] pub dream: Option<DreamConfig>`
  - [ ] 更新 `Default` 实现
  - [ ] `cargo check` 验证

### 2. MemoryStore 解耦 + history.jsonl

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/memory/src/store.rs`, `crates/memory/src/history.rs`（新增）, `crates/memory/src/error.rs`
- 验收标准: MemoryStore 不再持有 Provider/Options；history 使用 JSONL + cursor；现有测试适配通过
- 信心评估: 3
- 步骤:
  - [x] 新增 `history.rs`：定义 `HistoryEntry { cursor: u64, timestamp: String, content: String }`，实现 JSONL 读写、append、compaction（>1000 条时截断）
  - [x] MemoryStore 移除 `options: Options` 字段，移除 `create_save_memory_tool()`、`SaveMemoryArgs`
  - [x] MemoryStore 新增 `cursor: u64` 字段，`append_history` 改为写入 JSONL
  - [x] MemoryStore 新增 `read_history_since(cursor) -> Vec<HistoryEntry>` 方法
  - [x] 移除 `MemoryError::NoToolCall` 和 `MemoryError::ToolParse` 变体
  - [x] 更新 `ContextBuilder::new()` 中 MemoryStore 构造（不再传 options）
  - [x] 更新 `build_core_identity()` 中 HISTORY.MD 引用为 history.jsonl
  - [x] 更新 `try_consolidate()` 中的 MemoryStore 调用
  - [x] 适配现有测试
  - [x] `cargo test -p nanobot-memory` 验证

### 3. Consolidator 简化

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/memory/src/store.rs` 或 `crates/memory/src/consolidator.rs`（新增）
- 验收标准: 整合使用纯文本 LLM 摘要（tools=None），不再依赖 tool_choice
- 信心评估: 4
- 步骤:
  - [ ] 新增 `consolidator.rs`（或在 store.rs 中重写 `consolidate_internal`）
  - [ ] 整合提示词：要求 LLM 返回纯文本摘要（不调用工具）
  - [ ] 添加 "Solutions" 分类到提示词
  - [ ] 摘要结果直接 append 到 history.jsonl
  - [ ] 更新 `try_consolidate()` 调用新 API
  - [ ] `cargo test -p nanobot-memory` 验证

### 4. GitStore（git CLI）

- 优先级: P1
- 依赖项: 无
- 涉及文件: `crates/memory/src/gitstore.rs`（新增）
- 验收标准: 通过 `std::process::Command` 调用 git CLI 实现 init/commit/log/diff/revert
- 风险/注意点: 运行环境必须有 git 命令；git 不可用时必须报错
- 信心评估: 5
- 步骤:
  - [ ] 实现 `GitStore::init(path)` — `git init` + 写入 .gitignore（只跟踪记忆文件）
  - [ ] 实现 `GitStore::commit(message)` — `git add .` + `git commit -m`
  - [ ] 实现 `GitStore::log(limit) -> Vec<CommitInfo>` — `git log --format` 解析
  - [ ] 实现 `GitStore::diff(sha) -> String` — `git diff sha~1 sha`
  - [ ] 实现 `GitStore::revert(sha)` — `git checkout sha -- .` + commit
  - [ ] GitStore::init 时检查 git 可用性，不可用则返回错误，阻止 Dream 启动
  - [ ] 新增测试（在 tempdir 中初始化 git 仓库）
  - [ ] `cargo test -p nanobot-memory` 验证

### 5. Dream 两阶段处理器

- 优先级: P1
- 依赖项: 2, 3, 4
- 涉及文件: `crates/memory/src/dream.rs`（新增）
- 验收标准: Dream 能读取 history.jsonl 未处理条目，通过两阶段 LLM 调用更新记忆文件，推进 cursor
- 信心评估: 2
- 步骤:
  - [ ] 定义 `Dream` 结构体，持有 MemoryStore、GitStore、Provider、DreamConfig
  - [ ] Phase 1：构造提示词（history entries vs 记忆文件内容），LLM 返回 `[FILE] fact` 行
  - [ ] Phase 2：使用 re_act 循环 + read_file/edit_file 工具执行增量编辑
  - [ ] 执行后推进 cursor，调用 GitStore::commit
  - [ ] 新增测试
  - [ ] `cargo test -p nanobot-memory` 验证

### 6. CronService.register_system_job + Dream 调度

- 优先级: P1
- 依赖项: 5
- 涉及文件: `crates/cron/src/service/mod.rs`, `crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: Dream 作为系统 cron 任务注册，按 DreamConfig.cron 调度执行
- 信心评估: 3
- 步骤:
  - [ ] CronService 新增 `register_system_job(name, schedule, callback)` — 幂等注册内部任务
  - [ ] gateway 启动时注册 Dream 系统任务
  - [ ] `cargo test -p nanobot-cron` 验证

### 7. /dream 命令

- 优先级: P2
- 依赖项: 5, 4
- 涉及文件: `crates/agent/src/cmd/dream/mod.rs`（新增）, `crates/agent/src/loop/mod.rs`
- 验收标准: /dream 手动触发 Dream；/dream-log 显示变更；/dream-restore 回退
- 信心评估: 4
- 步骤:
  - [ ] 新增 `DreamCmd` 实现 `Command` trait — 触发 Dream::run()
  - [ ] 新增 `DreamLogCmd` — 调用 GitStore::log/diff
  - [ ] 新增 `DreamRestoreCmd` — 调用 GitStore::revert
  - [ ] 在 `try_handle_cmd()` 中注册 /dream、/dream-log、/dream-restore
  - [ ] `cargo test -p nanobot-agent` 验证

## 实现建议

- history.jsonl 格式：`{"cursor": 1, "timestamp": "2026-05-29T10:00:00Z", "content": "..."}\n`
- GitStore 通过 `std::process::Command` 调用 git CLI，零新依赖
- git 不可用时报错：GitStore::init 检查 git 命令存在性，不存在则返回错误
- Dream Phase 2 可复用 `re_act` 循环，传入受限的 ToolRegistry（只有 read_file + edit_file）
- Consolidator 的 LLM 调用需要传入 provider，但 MemoryStore 本身不持有 provider（由调用方传入）
- 阶段 1（任务 0+1）可以先行合入，不影响现有功能
