# TODO

## 任务列表

### 1. 修复 `decide` 方法中 tool_call_id 悬空引用 ✅

- 优先级: P0
- 依赖项: 无
- 风险/注意点: 在 `match tool_call.parse_arguments` 前添加 `let tool_call_id = tool_call.id.clone();`，确保在 `messages.push(response)` move `response` 后仍可使用 `tool_call_id`

### 2. 修复 retry 分支消息顺序并移除冗余 assistant 消息构造 ✅

- 优先级: P0
- 依赖项: 1
- 风险/注意点: 将第 254-260 行的两段 push 替换为连续两行：`messages.push(response);` 和 `messages.push(Message::tool(&tool_call_id, error_msg));`。原代码先 push tool 再 push assistant 是错误的（工具结果必须在调用它的 assistant 消息之后），且原 assistant 消息丢失了 tool_calls。

### 3. 修复错误日志计数 ✅

- 优先级: P1
- 依赖项: 无
- 风险/注意点: 将第 264 行的 `MAX_PARSE_RETRIES + 1` 改为 `MAX_PARSE_RETRIES`，与实际重试次数（1 次）一致

### 4. 验证修改 ✅

- 优先级: P0
- 依赖项: 1, 2, 3
- 风险/注意点: 运行 `cargo check` 和 `cargo clippy` 确保无编译错误和 lint 警告

## 实现建议

- 仅修改 `crates/heartbeat/src/service.rs` 中 `decide` 方法的 for 循环内 retry 分支
- 修改点均在 `crates/heartbeat/src/service.rs`，不涉及其他 crate
- 修改完成后执行 `cargo +nightly fmt` 和 `cargo clippy -- -D warnings -D clippy::uninlined_format_args` 验证代码规范
