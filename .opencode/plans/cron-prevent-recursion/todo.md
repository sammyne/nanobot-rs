# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/tools/src/core.rs` | 修改 | ToolContext 新增 scheduled 字段 + scheduled() 构造方法 |
| `crates/cron/src/tool/mod.rs` | 修改 | execute 中 Add 分支检查 ctx.scheduled |
| `crates/cron/src/tool/tests.rs` | 修改 | 新增 scheduled context 阻断测试 |
| `crates/agent/src/loop/mod.rs` | 修改 | re_act 新增 scheduled 参数；ToolContext 移到循环外；process_message/process_system_message 传值 |
| `crates/agent/src/loop/tests.rs` | 修改 | re_act 调用补充 scheduled 参数（2 处，均传 false） |

## 任务列表

### 1. ✅ ToolContext 新增 scheduled 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/tools/src/core.rs`
- 验收标准: `ToolContext::new()` 默认 scheduled=false；`ToolContext::scheduled()` 设置 scheduled=true
- 风险/注意点: ToolContext::new() 签名不变，现有调用方无需修改
- 信心评估: 5
- 步骤:
  - [ ] `ToolContext` 新增 `pub scheduled: bool` 字段
  - [ ] `ToolContext::new()` 中 `scheduled: false`
  - [ ] 新增 `pub fn scheduled(channel, chat_id) -> Self`，`scheduled: true`

### 2. ✅ CronTool 检查 scheduled 阻断 add

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/cron/src/tool/mod.rs`, `crates/cron/src/tool/tests.rs`
- 验收标准: ctx.scheduled 为 true 时 Add 返回错误，List/Remove 不受影响
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `execute()` 中 `CronArgs::Add` 分支开头检查 `if ctx.scheduled`，返回 `Err(ToolError::execution("cannot schedule new jobs from within a cron job execution"))`
  - [ ] 新增测试 `add_blocked_in_scheduled_context`：用 `ToolContext::scheduled()` 调用 add → 验证返回错误
  - [ ] 新增测试 `list_allowed_in_scheduled_context`：用 `ToolContext::scheduled()` 调用 list → 验证正常返回

### 3. ✅ AgentLoop 中 cron session 使用 scheduled context

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/agent/src/loop/tests.rs`
- 验收标准: session_key 以 "cron:" 开头时工具调用使用 ToolContext::scheduled()
- 风险/注意点: re_act() 签名新增 scheduled 参数；ToolContext 在 re_act 入口创建一次，移出工具调用循环
- 信心评估: 5
- 步骤:
  - [ ] `re_act()` 签名新增 `scheduled: bool` 参数
  - [ ] `re_act()` 入口处创建 `let ctx = if scheduled { ToolContext::scheduled(channel, chat_id) } else { ToolContext::new(channel, chat_id) };`
  - [ ] 移除工具调用循环内的 `let ctx = ToolContext::new(channel, chat_id);`（第 259 行），改用外部 `&ctx`
  - [ ] `process_message()` 中 `let scheduled = session_key.starts_with("cron:");`，传给 `re_act()`
  - [ ] `process_system_message()` 中 `re_act()` 调用传入 `scheduled: false`（系统消息不是 cron 上下文）
  - [ ] `crates/agent/src/loop/tests.rs` 中 2 处 `re_act()` 调用补充 `scheduled: false` 参数
  - [ ] 运行 `cargo clippy` + `cargo test` 确认通过

## 实现建议

- `ToolContext::scheduled()` 与 `ToolContext::new()` 参数相同，仅 `scheduled` 字段不同
- re_act() 的 scheduled 参数由 process_message() 根据 `session_key.starts_with("cron:")` 判断
- subagent 不调用 re_act()（有独立的工具执行循环），无需修改
