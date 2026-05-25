# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/agent/src/tools/mod.rs` | 新增 | `pub mod message;` |
| `crates/agent/src/tools/message/mod.rs` | 新增 | `MessageTool` 实现 |
| `crates/agent/src/tools/message/tests.rs` | 新增 | 单元测试 |
| `crates/agent/src/lib.rs` | 修改 | 添加 `mod tools;` |
| `crates/agent/src/loop/mod.rs` | 修改 | `AgentLoop` 加 `outbound_tx` 字段，`new()` 加参数，`run()` 移除参数，注册 `MessageTool` |
| `crates/agent/src/loop/tests.rs` | 修改 | 24 处 `AgentLoop::new()` 调用加 `outbound_tx` 参数 |
| `crates/nanobot/src/commands/agent/mod.rs` | 修改 | CLI 单次模式创建 channel 传入；交互模式 `tx` 从 `run()` 移到 `new()` |
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | `tx` 从 `run()` 移到 `new()` |

## 任务列表

### ✅ 1. 实现 MessageTool

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/tools/mod.rs`, `crates/agent/src/tools/message/mod.rs`, `crates/agent/src/lib.rs`
- 验收标准: `MessageTool` 实现 `Tool` trait，能通过 `outbound_tx` 发送 `OutboundMessage`
- 风险/注意点: `execute()` 是 `async fn` 但 `mpsc::Sender::send()` 也是 async，需要在 `execute()` 中 await
- 信心评估: 5（CronTool/SpawnTool 有参考模式）
- 步骤:
  - [ ] 创建 `crates/agent/src/tools/mod.rs`，声明 `pub mod message;`
  - [ ] 在 `crates/agent/src/lib.rs` 中添加 `mod tools;`
  - [ ] 创建 `crates/agent/src/tools/message/mod.rs`，实现 `MessageTool` struct：持有 `mpsc::Sender<OutboundMessage>`、`workspace: PathBuf`、`restrict_to_workspace: bool`
  - [ ] 实现 `Tool` trait：name `"message"`，description 描述主动发送能力，parameters JSON schema（content 必填，channel/chat_id/media 可选）
  - [ ] `execute()` 实现：从 params 解析 content/channel/chat_id/media，channel/chat_id 缺省时从 `ToolContext` 取默认值，解析媒体路径（本地相对 workspace 解析，URL 透传），构造 `OutboundMessage` 并通过 `outbound_tx.send()` 发送
  - [ ] 媒体路径安全检查：`restrict_to_workspace` 为 true 时验证路径在 workspace 内
  - [ ] 末尾添加 `#[cfg(test)] mod tests;`
  - [ ] 运行 `cargo check -p nanobot-agent` 验证编译

### ✅ 2. 重构 AgentLoop：outbound_tx 从 run() 移到 new()

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `AgentLoop::new()` 接受 `outbound_tx` 必填参数并存储；`run()` 不再接受 `outbound_tx` 参数，使用 `self.outbound_tx`；构造时注册 `MessageTool`
- 风险/注意点: `run()` 中 `handle_stop()` 也使用 `outbound_tx`，需改为 `self.outbound_tx`；`ChannelProgressTracker::new()` 也使用 `outbound_tx.clone()`，需改为 `self.outbound_tx.clone()`
- 信心评估: 4（改动面广，需仔细检查所有 `outbound_tx` 引用）
- 步骤:
  - [ ] `AgentLoop` struct 新增 `outbound_tx: mpsc::Sender<OutboundMessage>` 字段
  - [ ] `new()` 签名新增 `outbound_tx: mpsc::Sender<OutboundMessage>` 参数，存储到 `self.outbound_tx`
  - [ ] `new()` 中注册 `MessageTool`：`MessageTool::new(outbound_tx.clone(), config.workspace.clone(), tools_config.restrict_to_workspace)`
  - [ ] `run()` 签名移除 `outbound_tx` 参数，方法体中所有 `outbound_tx` 改为 `self.outbound_tx`
  - [ ] `handle_stop()` 签名移除 `outbound_tx` 参数，改用 `self.outbound_tx`
  - [ ] 更新 doc comment 和 `crates/agent/src/lib.rs` 中的示例代码
  - [ ] 运行 `cargo check -p nanobot-agent` 验证编译（测试暂时会失败，下一步修复）

### ✅ 3. 适配测试代码

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/agent/src/loop/tests.rs`
- 验收标准: 所有 24 处 `AgentLoop::new()` 调用传入 `outbound_tx`，所有测试通过
- 风险/注意点: 机械性改动，每处加 `let (outbound_tx, _outbound_rx) = tokio::sync::mpsc::channel(100);` 和传参
- 信心评估: 5
- 步骤:
  - [ ] 在测试辅助函数或每个测试中创建 `(outbound_tx, _outbound_rx)` channel
  - [ ] 所有 `AgentLoop::new()` 调用加 `outbound_tx` 参数
  - [ ] 运行 `cargo test -p nanobot-agent` 验证全部通过

### ✅ 4. 适配 CLI 调用方

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/nanobot/src/commands/agent/mod.rs`
- 验收标准: CLI 单次模式和交互模式都能正常运行
- 风险/注意点: 单次模式需新建 channel；交互模式 `outbound_tx` 从 `run()` 参数移到 `new()` 参数
- 信心评估: 5
- 步骤:
  - [ ] 单次模式 `run_single_message()`：创建 `(outbound_tx, mut outbound_rx) = mpsc::channel(100)`，`outbound_tx` 传入 `AgentLoop::new()`，`process_direct()` 后 drain `outbound_rx`（`while let Ok(msg) = outbound_rx.try_recv()` 打印 msg.content）
  - [ ] 交互模式 `run_interactive()`：`outbound_tx` 从 `agent_loop.run(inbound_rx, outbound_tx)` 移到 `AgentLoop::new(..., outbound_tx)`，`run()` 调用改为 `agent_loop.run(inbound_rx)`
  - [ ] 运行 `cargo check -p nanobot` 验证编译

### ✅ 5. 适配 Gateway 调用方

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: Gateway 正常启动运行
- 风险/注意点: `outbound_tx` 从 `run()` 参数移到 `new()` 参数；`ServicesContext` 中的 `outbound_tx` 字段可能需要调整
- 信心评估: 4（需检查 `ServicesContext` 和 `run_services` 中的 `outbound_tx` 使用）
- 步骤:
  - [ ] `AgentLoop::new()` 调用加 `outbound_tx.clone()` 参数
  - [ ] `run_services()` 中 `agent_loop.run(inbound_rx, outbound_tx)` 改为 `agent_loop.run(inbound_rx)`
  - [ ] 检查 `ServicesContext` 中 `outbound_tx` 是否还有其他用途（cron callback、heartbeat 等），如有则保留
  - [ ] 运行 `cargo check -p nanobot` 验证编译

### ✅ 6. MessageTool 单元测试

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/agent/src/tools/message/tests.rs`
- 验收标准: 覆盖核心场景
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 测试基本发送：content 参数，验证 `outbound_rx` 收到正确的 `OutboundMessage`
  - [ ] 测试 channel/chat_id 默认值：不传时使用 `ToolContext` 的值
  - [ ] 测试 channel/chat_id 自定义值：跨 channel 发送
  - [ ] 测试 media 参数：本地路径解析、URL 透传
  - [ ] 测试 media 路径不存在：返回错误信息
  - [ ] 测试 restrict_to_workspace：workspace 外路径被拒绝
  - [ ] 运行 `cargo test -p nanobot-agent` 验证通过

### ✅ 7. 全量验证

- 优先级: P0
- 依赖项: 3, 4, 5, 6
- 涉及文件: 无
- 验收标准: fmt + clippy + 全量测试通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `cargo +nightly fmt`
  - [ ] `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] `cargo test`

## 实现建议

- `MessageTool::execute()` 中 `outbound_tx.send().await` 需要 `.map_err()` 转为 `ToolError`
- 媒体路径解析参考 Python 版：URL（`http://`/`https://`）直接透传，本地路径用 `workspace.join(path)` 解析后 canonicalize
- `restrict_to_workspace` 检查参考 `ExecTool` 的 `validate_paths_in_workspace()` 逻辑
- 测试中创建 channel 可提取为辅助函数避免重复代码
