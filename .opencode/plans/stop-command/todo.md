# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/subagent/src/manager.rs` | 修改 | 会话级任务追踪、`cancel_by_session()`、存储 JoinHandle |
| `crates/subagent/src/tool.rs` | 修改 | 构造 session_key 传给 `spawn()` |
| `crates/subagent/tests/tests.rs` | 修改 | 更新 `spawn()` 调用签名，新增取消测试 |
| `crates/agent/src/loop/mod.rs` | 修改 | 存储 SubagentManager、重构 `run()`、`handle_stop()`、`is_stop_cmd()` |
| `crates/agent/src/cmd/mod.rs` | 修改 | 注册 StopCmd 模块 |
| `crates/agent/src/cmd/stop/mod.rs` | 新增 | StopCmd 命令实现（用于 `process_direct` 路径） |
| `crates/agent/src/cmd/stop/tests.rs` | 新增 | StopCmd 单元测试 |
| `crates/agent/src/cmd/help/mod.rs` | 修改 | 帮助文本添加 /stop 说明 |
| `crates/agent/src/loop/tests.rs` | 修改 | 新增 /stop 相关测试 |
| `crates/nanobot/src/commands/agent/mod.rs` | 修改 | CLI 模式 AgentLoop 包装 `Arc` |
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | 适配 `run(self: Arc<Self>)` 签名变更 |

## 任务列表

### ✅ 1. SubagentManager 会话级任务追踪

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/subagent/src/manager.rs`
- 验收标准: `spawn()` 按 session_key 存储 JoinHandle；`cancel_by_session()` 能 abort 指定会话的所有任务并返回取消数量；`get_running_count()` 返回正确计数
- 风险/注意点: 已完成的任务需要从 `session_tasks` 中清理，否则 HashMap 会无限增长。在 `spawn()` 的 tokio task 结束时自行移除
- 步骤:
  - [ ] 将 `running_tasks: AtomicUsize` 替换为 `session_tasks: Arc<tokio::sync::Mutex<HashMap<String, Vec<(String, tokio::task::JoinHandle<()>)>>>>`，key 为 session_key，value 为 `(task_id, JoinHandle)` 列表
  - [ ] 修改 `spawn()` 签名：新增 `session_key: impl Into<String>` 参数，放在 `task` 参数之后
  - [ ] 在 `spawn()` 中：将 `tokio::spawn()` 返回的 JoinHandle 存入 `session_tasks`，而非直接丢弃。在 spawned task 内部完成后，从 `session_tasks` 中移除自身条目（通过 task_id 匹配）
  - [ ] 新增 `pub async fn cancel_by_session(&self, session_key: &str) -> usize` 方法：从 `session_tasks` 中取出指定 session_key 的所有条目，对每个 JoinHandle 调用 `abort()`，返回取消的任务数量
  - [ ] 修改 `get_running_count()`：遍历 `session_tasks` 所有 value 的长度之和
  - [ ] 运行 `cargo clippy -p nanobot-subagent -- -D warnings -D clippy::uninlined_format_args` 确认无警告

### ✅ 2. SpawnTool 传递 session_key

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/subagent/src/tool.rs`
- 验收标准: `execute()` 从 `ToolContext` 构造 session_key 并传给 `spawn()`
- 风险/注意点: session_key 格式需与 `InboundMessage::session_key()` 一致（`"channel:chat_id"`）
- 步骤:
  - [ ] 在 `execute()` 中构造 `let session_key = format!("{}:{}", ctx.channel, ctx.chat_id);`
  - [ ] 将 `session_key` 作为第二个参数传给 `self.manager.clone().spawn(params.task, session_key, params.label, ...)`
  - [ ] 运行 `cargo clippy -p nanobot-subagent -- -D warnings -D clippy::uninlined_format_args` 确认无警告

### ✅ 3. 更新 subagent 集成测试

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: `crates/subagent/tests/tests.rs`
- 验收标准: 现有测试适配新的 `spawn()` 签名；新增 `cancel_by_session` 测试
- 风险/注意点: 现有测试中 `spawn()` 调用需要补充 `session_key` 参数
- 步骤:
  - [ ] 更新所有现有 `spawn()` 调用，补充 `session_key` 参数（如 `"test:chat"`)
  - [ ] 新增测试 `cancel_by_session_aborts_tasks`：spawn 2 个任务到同一 session_key，调用 `cancel_by_session()`，验证返回值为 2，`get_running_count()` 为 0
  - [ ] 新增测试 `cancel_by_session_only_affects_target`：spawn 任务到两个不同 session_key，取消其中一个，验证另一个不受影响
  - [ ] 运行 `cargo test -p nanobot-subagent` 确认全部通过

### ✅ 4. AgentLoop 存储 SubagentManager 引用

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `AgentLoop` 持有 `subagent_manager` 字段，命令处理器可通过 `&self` 访问
- 风险/注意点: 无
- 步骤:
  - [ ] 在 `AgentLoop` struct 中新增字段 `subagent_manager: Option<Arc<SubagentManager<P>>>`
  - [ ] 在 `new()` 中，在注册 SpawnTool 之前 clone `subagent_manager` 参数并存入字段
  - [ ] 运行 `cargo clippy -p nanobot-agent -- -D warnings -D clippy::uninlined_format_args` 确认无警告

### ✅ 5. 新增 StopCmd 命令

- 优先级: P0
- 依赖项: 1, 4
- 涉及文件: `crates/agent/src/cmd/stop/mod.rs`（新增）、`crates/agent/src/cmd/stop/tests.rs`（新增）、`crates/agent/src/cmd/mod.rs`（修改）
- 验收标准: `/stop` 通过 `try_handle_cmd` 可用，调用 `cancel_by_session()` 并返回确认消息
- 风险/注意点: StopCmd 用于 `process_direct()` 路径（CLI 单次调用、cron 回调）；`run()` 路径中 `/stop` 由 `handle_stop()` 直接处理，不经过 `try_handle_cmd`
- 步骤:
  - [ ] 创建 `crates/agent/src/cmd/stop/` 目录
  - [ ] 创建 `mod.rs`：定义 `StopCmd<P: Provider>` 结构体，持有 `subagent_manager: Option<Arc<SubagentManager<P>>>`；实现 `Command` trait，在 `run()` 中调用 `cancel_by_session(&session_key)` 并返回 "Stopped." 或含取消数量的消息
  - [ ] 创建 `tests.rs`：测试有/无 subagent_manager 时的行为
  - [ ] 在 `crates/agent/src/cmd/mod.rs` 中添加 `mod stop;` 和 `pub use stop::StopCmd;`
  - [ ] 在 `try_handle_cmd` 的 match 中添加 `"stop"` 分支，构造 `StopCmd` 并执行
  - [ ] 运行 `cargo test -p nanobot-agent -- stop` 确认测试通过

### ✅ 6. 更新 HelpCmd

- 优先级: P1
- 依赖项: 5
- 涉及文件: `crates/agent/src/cmd/help/mod.rs`
- 验收标准: `/help` 输出包含 `/stop` 说明
- 风险/注意点: 需要同步更新 help 相关的测试断言
- 步骤:
  - [ ] 在帮助文本中添加 `\n/stop — Stop current processing and cancel background tasks`
  - [ ] 更新 `crates/agent/src/loop/tests.rs` 中所有断言 help 输出内容的测试用例
  - [ ] 运行 `cargo test -p nanobot-agent -- help` 确认通过

### ✅ 7. 重构 `run()` 支持并发 /stop

- 优先级: P0
- 依赖项: 4, 5
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `run()` 将 `process_message` 作为 tokio task 启动；处理期间收到 `/stop` 时 abort 主任务并取消子代理；非 /stop 消息在处理期间到达时缓冲到下一轮处理
- 风险/注意点: 签名从 `&self` 改为 `self: Arc<Self>`，影响所有调用方；需要 `use std::collections::VecDeque` 做消息缓冲
- 步骤:
  - [ ] 新增辅助函数 `fn is_stop_cmd(content: &str) -> bool`：`content.trim_end().eq_ignore_ascii_case("/stop")`
  - [ ] 新增方法 `async fn handle_stop(&self, msg: &InboundMessage, outbound_tx: &mpsc::Sender<OutboundMessage>)`：调用 `self.subagent_manager.cancel_by_session()` 取消子代理，发送 "Stopped." 响应
  - [ ] 修改 `run()` 签名为 `pub async fn run(self: Arc<Self>, mut inbound_rx: mpsc::Receiver<InboundMessage>, outbound_tx: mpsc::Sender<OutboundMessage>) -> Result<()>`
  - [ ] 重写 `run()` 循环体：(a) 维护 `pending: VecDeque<InboundMessage>` 缓冲区；(b) 优先从 pending 取消息，否则从 inbound_rx recv；(c) 如果是 /stop 且无任务运行，直接 handle_stop 并 continue；(d) 否则 `tokio::spawn` 处理消息；(e) 用 `tokio::select!` 等待任务完成或新消息到达——新消息是 /stop 则 abort 任务并 handle_stop，否则 push 到 pending
  - [ ] 运行 `cargo clippy -p nanobot-agent -- -D warnings -D clippy::uninlined_format_args` 确认无警告

### ✅ 8. 适配调用方

- 优先级: P0
- 依赖项: 7
- 涉及文件: `crates/nanobot/src/commands/agent/mod.rs`、`crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: CLI 和 gateway 模式正常启动，`run()` 调用适配新签名
- 风险/注意点: gateway 模式已用 `Arc<AgentLoop>`，只需调整调用方式；CLI 模式需包装 `Arc::new()`
- 步骤:
  - [ ] CLI agent 模式（`commands/agent/mod.rs`）：将 `let agent_loop = AgentLoop::new(...)` 改为 `let agent_loop = Arc::new(AgentLoop::new(...).await?)`，`tokio::spawn` 中调用 `agent_loop.run(...)` 时 clone Arc
  - [ ] Gateway 模式（`commands/gateway/mod.rs`）：`agent_loop_clone` 已是 `Arc`，确认 `agent_loop_clone.run(ctx.inbound_rx, ctx.outbound_tx)` 调用兼容新签名（`Arc<Self>` 可直接调用 `self: Arc<Self>` 方法）
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args` 确认全项目无警告

### ✅ 9. 全量验证

- 优先级: P1
- 依赖项: 1-8
- 涉及文件: 无（仅运行命令）
- 验收标准: 所有检查通过，无回归
- 风险/注意点: 无
- 步骤:
  - [ ] 运行 `cargo +nightly fmt`
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 运行 `cargo test`
  - [ ] 运行 `cargo doc --no-deps`

## 实现建议

- 任务 1-3（subagent crate）和任务 4-6（agent crate 命令部分）可以并行开发，任务 7 依赖两者
- `session_tasks` 使用 `tokio::sync::Mutex` 而非 `std::sync::Mutex`，因为清理操作在 async 上下文中执行
- spawned task 完成后自行清理：在 `tokio::spawn` 的 async block 末尾，lock `session_tasks` 并移除自身的 `(task_id, JoinHandle)` 条目。由于 JoinHandle 已在 map 中，task 内部需要通过 task_id 匹配来移除（不能移除 JoinHandle 本身，因为它在 map 中被持有）
- `run()` 中的 `tokio::select!` 不需要 `biased`——两个分支的优先级无所谓，因为 /stop 到达时无论哪个分支先匹配都会正确处理
