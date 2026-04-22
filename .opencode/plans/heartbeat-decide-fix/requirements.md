# 需求

## 目标与背景

修复 `crates/heartbeat/src/service.rs` 中 `decide` 方法的重试逻辑。当前实现在 LLM 工具参数解析失败后进行重试时，向 `messages` 历史记录中追加消息的顺序错误，导致 LLM 无法正确理解对话上下文，重试机制实际上无效。

## 功能需求列表

### 核心功能

- **修复 `decide` 方法的重试消息顺序**：在解析工具参数失败后追加重试消息时，`Message::tool`（工具结果）必须出现在 `Message::assistant`（助手响应，包含 tool_calls）之后。当前实现先 push 了 tool 再 push assistant，顺序颠倒。
- **修复 tool_call_id 悬空引用**：在 push `response` 之前，需要先 clone `tool_call.id`，避免在 `messages.push(response)` move `response` 后继续使用已被 move 的引用。
- **移除冗余的 assistant 消息构造**：直接 push 完整的 `response`（类型为 `Message::Assistant { content, tool_calls }`），而非仅提取 `content` 构造新的 assistant 消息。新构造的 assistant 消息丢失了 `tool_calls`，导致 `tool_call.id` 失去引用目标。
- **修复错误日志计数**：当前日志输出 `MAX_PARSE_RETRIES + 1`（值为 2），但实际只重试了 1 次，应改为 `MAX_PARSE_RETRIES`（值为 1）。

## 非功能需求

- **测试覆盖**：修改后应验证 `decide` 方法在参数解析失败时能正确重试并累积正确的消息历史。
- **代码质量**：遵循项目已有的代码风格（使用 `tracing` 日志、`thiserror` 定义错误），不引入新的 lint 警告。

## 边界与不做事项

- 仅修改 `crates/heartbeat/src/service.rs` 中 `decide` 方法的重试分支。
- 不修改 `tick` 方法或其他业务逻辑。
- 不修改 `HeartbeatError` 错误类型定义。
- 不添加新测试（当前代码库无 `heartbeat` crate 的单元测试文件，后续可考虑添加）。

## 假设与约束

- **技术假设**：基于项目现有 `Message` 枚举结构（`Message::Assistant { content, tool_calls }`），`response` 可直接作为 `Message` push 到历史记录中，无需构造新实例。
- **资源约束**：仅涉及单文件修改，影响范围可控。
- **环境约束**：Rust >= 1.93，使用 `tokio` 异步运行时。

## 修改点详情

### 文件：`crates/heartbeat/src/service.rs`

| 位置 | 问题 | 修复方式 |
|------|------|---------|
| 第 234 行前 | `tool_call` 的 `id` 字段在后续使用前未 clone | 在 `match tool_call.parse_arguments` 前添加 `let tool_call_id = tool_call.id.clone();` |
| 第 254-260 行 | 消息顺序颠倒：先 push tool 再 push assistant；且 assistant 消息丢失 tool_calls | 替换为 `messages.push(response); messages.push(Message::tool(&tool_call_id, error_msg));` |
| 第 264 行 | 日志计数 `MAX_PARSE_RETRIES + 1` 语义错误（显示 2，实际重试 1 次） | 改为 `MAX_PARSE_RETRIES` |
