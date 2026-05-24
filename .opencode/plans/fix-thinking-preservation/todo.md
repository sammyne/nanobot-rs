# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/agent/src/loop/mod.rs` | 修改 | `re_act()` 中将原始 response 直接推入 messages |
| `crates/subagent/src/manager.rs` | 修改 | `run_subagent()` 中将原始 response 直接推入 messages |
| `crates/agent/src/loop/tests.rs` | 修改 | 新增 thinking 保留的单元测试 |

## 任务列表

### ✅ 1. 修复 `re_act()` 中 thinking 数据丢失

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `re_act()` 返回的 `ReActResult.messages` 中，来自 LLM 的 assistant 消息保留原始 `thinking` 字段值
- 风险/注意点: `response.tool_calls()` 返回 `&[ToolCall]` 借用 response，push response 后借用失效，需先提取所需数据
- 步骤:
  - [ ] 将 `content` 和 `tool_calls` 的提取移到 `if` 分支之前：在 `let response = self.call_llm(&messages).await?;` 之后，添加 `let content = response.content().to_string();` 和 `let tool_calls = response.tool_calls().to_vec();`，删除原来 if 分支内的 `let tool_calls = response.tool_calls();` 和 `let content = response.content().to_string();`
  - [ ] 修改 if 条件：`if !tool_calls.is_empty() {`（使用提取后的 owned Vec）
  - [ ] 替换 tool-call 分支的消息构造：将 `messages.push(Message::assistant_with_tools(&content, tool_calls.to_vec()));` 改为 `messages.push(response);`
  - [ ] 修改工具调用迭代：将 `for tool_call in tool_calls {` 改为 `for tool_call in &tool_calls {`
  - [ ] 替换 else 分支的消息构造：删除 `let final_content = response.content().to_string();`，将 `messages.push(Message::assistant(&final_content));` 改为 `messages.push(response);`，将 else 分支中后续使用 `final_content` 的地方改为 `content`（已在 if 之前提取）
  - [ ] 运行 `cargo clippy -p nanobot-agent -- -D warnings -D clippy::uninlined_format_args` 确认无警告

### ✅ 2. 修复 `run_subagent()` 中 thinking 数据丢失

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/subagent/src/manager.rs`
- 验收标准: `run_subagent()` 中多轮工具调用时，messages 向量中的 assistant 消息保留原始 `thinking` 字段值
- 风险/注意点: else 分支（无工具调用）只提取 `final_result` 文本，不推入 messages，无需修改
- 步骤:
  - [ ] 修改 tool_calls 提取：将 `let tool_calls = response.tool_calls();` 改为 `let tool_calls = response.tool_calls().to_vec();`
  - [ ] 替换消息构造：将 `messages.push(Message::assistant_with_tools(response.content().to_string(), tool_calls.to_vec()));` 改为 `messages.push(response);`
  - [ ] 修改工具调用迭代：将 `for tool_call in tool_calls {` 改为 `for tool_call in &tool_calls {`
  - [ ] 运行 `cargo clippy -p nanobot-subagent -- -D warnings -D clippy::uninlined_format_args` 确认无警告

### ✅ 3. 添加 thinking 保留的单元测试

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/loop/tests.rs`
- 验收标准: 测试验证 `re_act()` 在多轮工具调用场景中保留 thinking 数据到最终 messages 列表
- 风险/注意点: 需要一个能返回带 thinking 数据的 MockProvider；现有 MockProvider 不支持 thinking，需新建一个
- 步骤:
  - [ ] 在 tests.rs 中新增 `ThinkingMockProvider` 结构体：包含 `call_count: Arc<AtomicUsize>` 字段，实现 `Provider` trait。第一次调用返回 `Message::assistant_with_thinking("thinking response", vec![ToolCall::new("call_1", "read_file", json!({"path": "/tmp/test.txt"}))], json!({"type": "thinking", "thinking": "Let me think...", "signature": "sig123"}))"`；第二次调用返回 `Message::assistant_with_thinking("final answer", vec![], json!({"type": "thinking", "thinking": "Done thinking.", "signature": "sig456"}))`
  - [ ] 新增测试函数 `re_act_preserves_thinking_in_messages`：使用 `ThinkingMockProvider` 创建 `AgentLoop`，调用 `re_act()` 传入包含一条 system 消息和一条 user 消息的 messages 向量
  - [ ] 在测试中验证：遍历 `result.messages`，找到所有 `Message::Assistant` 变体，断言每个的 `thinking()` 返回 `Some(_)` 而非 `None`；验证第一条 assistant 消息的 thinking 包含 `"Let me think..."`，最后一条包含 `"Done thinking."`
  - [ ] 运行 `cargo test -p nanobot-agent -- re_act_preserves_thinking` 确认测试通过

### ✅ 4. 全量验证

- 优先级: P1
- 依赖项: 1, 2, 3
- 涉及文件: 无（仅运行命令）
- 验收标准: 所有检查通过，无回归
- 风险/注意点: 无
- 步骤:
  - [ ] 运行 `cargo +nightly fmt`
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 运行 `cargo test`
  - [ ] 运行 `cargo doc --no-deps`

## 实现建议

- 任务 1 和 2 的核心思路相同：将 `messages.push(Message::assistant_with_tools(...))` 替换为 `messages.push(response)`，关键是在 push 之前提取后续需要的 `content` 和 `tool_calls` 数据（因为 push 会 move response）
- `ToolCall` 已 derive `Clone`，`.to_vec()` 的开销与当前代码中 `tool_calls.to_vec()` 传入构造函数的开销相同，无额外性能影响
- OpenAI provider 返回的 Message 的 thinking 始终为 None，push 原始 response 等价于当前 `Message::assistant()` / `Message::assistant_with_tools()` 的行为，无兼容性风险
