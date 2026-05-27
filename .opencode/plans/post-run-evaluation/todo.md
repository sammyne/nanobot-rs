# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/nanobot/src/evaluator/mod.rs` | 新增 | `evaluate_response` 函数、`evaluate_notification` 工具定义、系统提示 |
| `crates/nanobot/src/evaluator/tests.rs` | 新增 | 评估器单元测试 |
| `crates/nanobot/src/lib.rs` | 修改 | 导出 `evaluator` 模块 |
| `crates/heartbeat/src/callback.rs` | 修改 | 新增 `OnEvaluateCallback` 类型 |
| `crates/heartbeat/src/lib.rs` | 修改 | 重导出 `OnEvaluateCallback` |
| `crates/heartbeat/src/service/mod.rs` | 修改 | 新增 `on_evaluate` 字段，`tick()` 中插入评估阶段 |
| `crates/heartbeat/AGENTS.md` | 修改 | 更新关键类型文档 |
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | 装配 `on_evaluate` 回调，cron 回调中插入评估 |

## 任务列表

### 1. 创建评估器模块

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/nanobot/src/evaluator/mod.rs`, `crates/nanobot/src/lib.rs`
- 验收标准: `evaluate_response` 函数可编译，接受 `&impl Provider`、response、task_context，返回 `bool`
- 风险/注意点: `bind_tools` 会替换所有已绑定工具，必须 clone provider 再绑定评估工具
- 信心评估: 5（heartbeat 的 `decide()` 已有完全相同的 tool-call + 解析模式可参考）
- 步骤:
  - [ ] 在 `crates/nanobot/src/` 下创建 `evaluator/mod.rs`
  - [ ] 定义 `EvaluateNotificationArgs` 结构体（`should_notify: bool`, `reason: Option<String>`），derive `Deserialize` + `JsonSchema`
  - [ ] 定义 `EVALUATE_TOOL` 静态工具定义（`LazyLock<ToolDefinition>`），使用 `schemars::schema_for!(EvaluateNotificationArgs)` 生成参数 schema
  - [ ] 定义 `SYSTEM_PROMPT` 常量，内容：通知门控角色 + 通知/抑制判断标准
  - [ ] 实现 `pub async fn evaluate_response<P: Provider>(provider: &P, response: &str, task_context: &str) -> bool`：clone provider → bind EVALUATE_TOOL → 构造 system+user 消息 → `chat(messages, &Options { max_tokens: Some(256), temperature: Some(0.0), .. })` → 解析 tool_call 中的 `should_notify` → 任何错误返回 `true`（fail-open）
  - [ ] 在 `crates/nanobot/src/lib.rs` 中添加 `pub mod evaluator;`
  - [ ] `cargo check -p nanobot` 确认编译通过

### 2. 扩展 heartbeat 回调支持评估

- 优先级: P0
- 依赖项: 无（与任务 1 并行）
- 涉及文件: `crates/heartbeat/src/callback.rs`, `crates/heartbeat/src/lib.rs`, `crates/heartbeat/src/service/mod.rs`
- 验收标准: `HeartbeatService::new()` 接受可选的 `on_evaluate` 回调；`tick()` 在 `on_execute` 返回后、`on_notify` 调用前执行评估；评估返回 `false` 时跳过 `on_notify`
- 风险/注意点: `on_evaluate` 返回 `bool` 而非 `Result<bool>`，因为评估器内部 fail-open，不会产生需要上层处理的错误
- 信心评估: 5（完全对齐现有 `on_execute`/`on_notify` 回调模式）
- 步骤:
  - [ ] 在 `crates/heartbeat/src/callback.rs` 中新增 `OnEvaluateCallback` 类型：`Arc<dyn Fn(&str, &str) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>`（参数：response, task_context）
  - [ ] 在 `crates/heartbeat/src/lib.rs` 中添加 `OnEvaluateCallback` 到 re-export
  - [ ] 在 `HeartbeatService` 结构体中新增字段 `on_evaluate: Arc<RwLock<Option<OnEvaluateCallback>>>`
  - [ ] 修改 `HeartbeatService::new()` 签名，新增 `on_evaluate: Option<OnEvaluateCallback>` 参数，初始化字段
  - [ ] 修改 `tick()` 中 `Action::Run` 分支：在 `on_execute` 返回非空 result 后、`on_notify` 调用前，插入评估逻辑：读取 `self.on_evaluate`，若有则调用 `evaluate(&result, &tasks).await`，返回 `false` 时 `info!("Heartbeat notification suppressed by post-run evaluation")` 并跳过 `on_notify`；若无 `on_evaluate` 则保持原行为（直接调用 `on_notify`）
  - [ ] `cargo check -p nanobot-heartbeat` 确认编译通过

### 3. 装配评估回调到 gateway

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: `crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: heartbeat 和 cron 任务执行后，通过 LLM 评估决定是否发送通知；评估失败时默认发送
- 风险/注意点: `setup_cron_callback` 需要新增 `provider` 参数以供评估器使用；cron 回调闭包需要 clone provider 进入 `move` 闭包
- 信心评估: 4（回调装配模式已有参考，但 cron 闭包需要额外捕获 provider）
- 步骤:
  - [ ] 在 `setup_heartbeat_service()` 中构造 `on_evaluate` 回调：捕获 `provider.clone()`，闭包内调用 `crate::evaluator::evaluate_response(&provider, response, task_context).await`
  - [ ] 修改 `HeartbeatService::new()` 调用，传入 `Some(on_evaluate)`
  - [ ] 修改 `setup_cron_callback()` 签名，新增 `provider: AnyProvider` 参数
  - [ ] 修改 cron 回调闭包：在 `payload.deliver && payload.to.is_some()` 分支中，`outbound_tx.send()` 前调用 `evaluate_response(&provider, &response, &payload.message).await`，仅 `should_notify == true` 时发送
  - [ ] 更新 `run_services()` 中 `setup_cron_callback` 的调用，传入 `ctx.provider.clone()`
  - [ ] `cargo check -p nanobot` 确认编译通过

### 4. 添加测试

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: `crates/nanobot/src/evaluator/tests.rs`, `crates/heartbeat/src/tests.rs`
- 验收标准: 覆盖评估器的 4 种场景 + heartbeat tick 的评估集成
- 风险/注意点: 需要 mock Provider；heartbeat 已有测试基础设施（`crates/heartbeat/src/tests.rs`）可参考
- 信心评估: 4（需要构造 mock provider 返回预设 tool_call 响应）
- 步骤:
  - [ ] 在 `crates/nanobot/src/evaluator/mod.rs` 末尾添加 `#[cfg(test)] mod tests;`
  - [ ] 创建 `crates/nanobot/src/evaluator/tests.rs`，实现 `MockProvider`（clone + bind_tools + chat 返回预设响应）
  - [ ] 测试 `should_notify_true`：mock 返回 `should_notify=true` 的 tool_call → 断言 `evaluate_response` 返回 `true`
  - [ ] 测试 `should_notify_false`：mock 返回 `should_notify=false` 的 tool_call → 断言返回 `false`
  - [ ] 测试 `fallback_on_provider_error`：mock chat 返回 `Err(ProviderError::Api(...))` → 断言返回 `true`（fail-open）
  - [ ] 测试 `fallback_on_no_tool_call`：mock 返回纯文本 Assistant 消息（无 tool_calls）→ 断言返回 `true`
  - [ ] 在 `crates/heartbeat/src/tests.rs` 中新增 `tick_suppresses_when_evaluator_says_no`：构造 `on_evaluate` 返回 `false` 的 HeartbeatService → 断言 `on_notify` 未被调用
  - [ ] 在 `crates/heartbeat/src/tests.rs` 中新增 `tick_notifies_when_evaluator_says_yes`：构造 `on_evaluate` 返回 `true` → 断言 `on_notify` 被调用
  - [ ] `cargo test -p nanobot -p nanobot-heartbeat` 确认全部通过

### 5. 更新文档

- 优先级: P1
- 依赖项: 2
- 涉及文件: `crates/heartbeat/AGENTS.md`
- 验收标准: AGENTS.md 反映新增的 `OnEvaluateCallback` 类型和 `HeartbeatService::new()` 签名变更
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 在 `crates/heartbeat/AGENTS.md` 的关键类型部分，`HeartbeatService` 的 `new()` 签名中添加 `on_evaluate` 参数
  - [ ] 新增 `OnEvaluateCallback` 类型说明

## 实现建议

- `evaluate_response` 的实现可直接参考 `crates/heartbeat/src/service/mod.rs` 中 `decide()` 的 tool-call + 解析模式（clone provider → bind tool → chat → parse tool_call arguments）
- `EvaluateNotificationArgs` 使用 `schemars::JsonSchema` derive 生成参数 schema，与 heartbeat 的 `Action` 结构体一致
- mock provider 可参考 `crates/heartbeat/src/tests.rs` 中已有的 `MockProvider` 实现
