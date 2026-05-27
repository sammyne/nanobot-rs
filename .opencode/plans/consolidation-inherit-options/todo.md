# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/memory/src/store.rs` | 修改 | `MemoryStore` 新增 `options` 字段，`new()` 接受 options，`consolidate_internal` 使用 `self.options` |
| `crates/memory/tests/memory.rs` | 修改 | 所有 `MemoryStore::new()` 调用传入 `Options::default()` |
| `crates/context/src/builder/mod.rs` | 修改 | `ContextBuilder::new()` 接受 options，透传给 `MemoryStore::new()` |
| `crates/context/tests/context.rs` | 修改 | 所有 `ContextBuilder::new()` 调用传入 `Options::default()` |
| `crates/agent/src/loop/mod.rs` | 修改 | 构造 Options 传入 `ContextBuilder::new()` |
| `crates/agent/src/loop/tests.rs` | 修改 | 适配 `ContextBuilder::new()` 新签名（如有直接调用） |
| `crates/memory/AGENTS.md` | 修改 | 更新 `MemoryStore::new()` 签名说明 |
| `crates/context/AGENTS.md` | 修改 | 更新 `ContextBuilder::new()` 签名说明 |

## 任务列表

### 1. MemoryStore 新增 options 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/memory/src/store.rs`, `crates/memory/tests/memory.rs`
- 验收标准: `MemoryStore` 持有 `options: Options`；`new(workspace, options)` 接受并存储；`consolidate_internal` 使用 `self.options` 而非 `Options::default()`
- 信心评估: 5
- 步骤:
  - [ ] `MemoryStore` struct 新增 `options: nanobot_provider::Options` 字段
  - [ ] `new()` 签名改为 `new(workspace: PathBuf, options: nanobot_provider::Options)`，存储 options
  - [ ] `consolidate_internal` 中删除 `let options = nanobot_provider::Options::default();`，改为 `let options = self.options;`
  - [ ] `crates/memory/tests/memory.rs` 中所有 `MemoryStore::new(workspace)` 改为 `MemoryStore::new(workspace, nanobot_provider::Options::default())`
  - [ ] 运行 `cargo test -p nanobot-memory` 验证通过

### 2. ContextBuilder 透传 options

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/context/src/builder/mod.rs`, `crates/context/tests/context.rs`
- 验收标准: `ContextBuilder::new(workspace, options)` 接受 options 并透传给 `MemoryStore::new()`
- 信心评估: 5
- 步骤:
  - [ ] `ContextBuilder::new()` 签名改为 `new(workspace: PathBuf, options: nanobot_provider::Options)`
  - [ ] 将 options 透传给 `MemoryStore::new(workspace, options)`
  - [ ] `crates/context/tests/context.rs` 中所有 `ContextBuilder::new(path)` 改为 `ContextBuilder::new(path, nanobot_provider::Options::default())`
  - [ ] 运行 `cargo test -p nanobot-context` 验证通过

### 3. AgentLoop 构造 Options 传入

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/agent/src/loop/tests.rs`
- 验收标准: `AgentLoop::new()` 从 config 构造 Options 传入 `ContextBuilder::new()`
- 信心评估: 5
- 步骤:
  - [ ] 在 `AgentLoop::new()` 中，构造 `let options = nanobot_provider::Options { max_tokens: config.max_tokens as u16, temperature: config.temperature as f32, reasoning_effort: config.reasoning_effort };`
  - [ ] 将 `ContextBuilder::new(config.workspace.clone())` 改为 `ContextBuilder::new(config.workspace.clone(), options)`
  - [ ] 检查 `crates/agent/src/loop/tests.rs` 是否有直接调用 `ContextBuilder::new()`，如有则适配
  - [ ] 运行 `cargo test` 全量验证通过
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 验证无警告

### 4. 更新 AGENTS.md

- 优先级: P1
- 依赖项: 3
- 涉及文件: `crates/memory/AGENTS.md`, `crates/context/AGENTS.md`
- 验收标准: 文档反映新签名
- 信心评估: 5
- 步骤:
  - [ ] `crates/memory/AGENTS.md`：`new(workspace)` 改为 `new(workspace, options)`
  - [ ] `crates/context/AGENTS.md`：`new(workspace)` 改为 `new(workspace, options)`
