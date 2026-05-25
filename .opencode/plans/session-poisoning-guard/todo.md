# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/agent/src/loop/mod.rs` | 修改 | 在 `re_act()` 无 tool_calls 分支中，空 content 替换为 `"(empty)"` |
| `crates/agent/src/loop/tests.rs` | 修改 | 新增空内容防御测试 |

## 任务列表

### ✅ 1. 在 re_act() 中防御空 assistant 内容

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: 当 LLM 返回空 content 且无 tool_calls 时，`ReActResult.content` 和 messages 中的 assistant 消息 content 均为 `"(empty)"`
- 风险/注意点: 需保留 response 中的 `thinking` 字段；仅在无 tool_calls 时替换（有 tool_calls 时空 content 是正常的）
- 信心评估: 5
- 步骤:
  - [ ] 在 `re_act()` 的 `else`（无 tool_calls）分支中，检查 `content.is_empty()`
  - [ ] 若为空，将 `content` 替换为 `"(empty)".to_string()`，并重建 response 保留 thinking 字段
  - [ ] 添加 `warn!` 日志记录此异常情况
  - [ ] 运行 `cargo check -p nanobot-agent` 验证编译

### ✅ 2. 新增空内容防御测试

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/agent/src/loop/tests.rs`
- 验收标准: 测试验证空 content 被替换为 `"(empty)"`
- 风险/注意点: 需要构造返回空 content 的 MockProvider
- 信心评估: 4（需确认现有测试 mock 模式）
- 步骤:
  - [ ] 查看现有 `re_act` 测试的 mock 模式
  - [ ] 新增测试：MockProvider 返回空 content 无 tool_calls → `ReActResult.content == "(empty)"`
  - [ ] 运行 `cargo test -p nanobot-agent` 验证通过

## 实现建议

- `response` 是 `Message::Assistant { content, tool_calls, thinking }`，在无 tool_calls 分支中 `tool_calls` 为空。重建时用 `match response` 解构取出 `thinking`，再用 `Message::assistant_with_thinking` 或 `Message::assistant` 重建
