# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/agent/src/loop/mod.rs` | 修改 | AgentLoop struct 字段、`new()` 签名、`handle_stop()`、`try_handle_cmd()` 移除 Option |
| `crates/agent/src/cmd/stop/mod.rs` | 修改 | StopCmd 字段移除 Option |
| `crates/agent/src/loop/tests.rs` | 修改 | 19 处 `None` 改为 `mock_subagent_manager(provider.clone())` |
| `crates/nanobot/src/commands/agent/mod.rs` | 修改 | `run_once()` 创建 SubagentManager，`run_interactive()` 移除 `Some()` |
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | 移除 `Some()` 包裹 |

## 任务列表

### 1. AgentLoop 移除 Option 包裹

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `subagent_manager` 字段和参数类型为 `Arc<SubagentManager<P>>`；`handle_stop()` 直接调用 `self.subagent_manager.cancel_by_session()` 无 `if let`
- 步骤:
  - [ ] struct 字段：`subagent_manager: Option<Arc<SubagentManager<P>>>` → `subagent_manager: Arc<SubagentManager<P>>`
  - [ ] `new()` 参数：`subagent_manager: Option<Arc<SubagentManager<P>>>` → `subagent_manager: Arc<SubagentManager<P>>`
  - [ ] `new()` 内部：SpawnTool 注册从 `if let Some(ref manager)` 改为直接使用 `&subagent_manager`
  - [ ] `new()` 内部：字段赋值从 `subagent_manager` (Option) 改为直接赋值
  - [ ] `handle_stop()`：移除 `if let Some(ref manager)` 分支，改为 `let cancelled = self.subagent_manager.cancel_by_session(&session_key).await;`
  - [ ] `try_handle_cmd()`：`StopCmd::new(self.subagent_manager.clone())` 移除外层 Option

### 2. StopCmd 移除 Option 包裹

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/cmd/stop/mod.rs`、`crates/agent/src/cmd/stop/tests.rs`
- 验收标准: `subagent_manager` 字段类型为 `Arc<SubagentManager<P>>`；`run()` 直接调用无 `if let`
- 步骤:
  - [ ] 字段：`subagent_manager: Option<Arc<SubagentManager<P>>>` → `subagent_manager: Arc<SubagentManager<P>>`
  - [ ] `new()` 参数同步修改
  - [ ] `run()` 中移除 `if let Some(ref manager)` 分支，直接调用 `self.subagent_manager.cancel_by_session(&session_key).await`
  - [ ] 更新 `tests.rs`：测试需要创建 SubagentManager 而非传 None

### 3. 更新测试调用

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/loop/tests.rs`
- 验收标准: 所有 `AgentLoop::new()` 调用传入 `mock_subagent_manager(provider.clone())`，无 `None` 和 `Some()`
- 步骤:
  - [ ] 将所有 `None, None, nanobot_config::ToolsConfig::default()` 模式（第 3、4 参数为 None, None）中的第 4 个 `None` 替换为 `mock_subagent_manager(provider.clone())`
  - [ ] 将所有 `Some(subagent_manager)` 替换为 `subagent_manager`（移除 Some 包裹）
  - [ ] 运行 `cargo test -p nanobot-agent` 确认全部通过

### 4. 更新生产代码调用方

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/nanobot/src/commands/agent/mod.rs`、`crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: `run_once()` 创建 SubagentManager；所有调用移除 `Some()` 包裹
- 步骤:
  - [ ] `run_once()`：创建 mpsc channel 和 SubagentManager，替换原来的 `None`
  - [ ] `run_interactive()`：`Some(subagent_manager)` → `subagent_manager`
  - [ ] `gateway/mod.rs`：`Some(subagent_manager)` → `subagent_manager`（如有）
  - [ ] 更新 `lib.rs` 中的文档注释示例（如有 `None` 示例）

### 5. 全量验证

- 优先级: P1
- 依赖项: 1-4
- 涉及文件: 无
- 验收标准: 所有检查通过
- 步骤:
  - [ ] `cargo +nightly fmt`
  - [ ] `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] `cargo test`
  - [ ] `cargo doc --no-deps`
