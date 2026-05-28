# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/tools/src/registry.rs` | 修改 | 新增 prepare_call 方法 |
| `crates/tools/src/core.rs` | 修改 | Tool trait 新增 read_only 方法 |
| `crates/tools/src/filesystem.rs` | 修改 | 标记 read_only |
| `crates/agent/src/loop/mod.rs` | 修改 | re_act 使用 prepare_call、大结果持久化、工具批处理、上下文治理、检查点 |
| `crates/agent/src/utils/mod.rs` | 修改 | 新增 tool_result 持久化工具函数 |
| `crates/context/src/lib.rs` | 修改 | 消息角色合并逻辑 |
| `crates/config/src/schema/agent.rs` | 修改 | 新增 max_tool_result_chars、context_window_tokens 配置 |

## 任务列表

### A. prepare_call 验证分离

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/tools/src/registry.rs`
- 验收标准: `ToolRegistry::prepare_call()` 返回 `Result<(&dyn Tool, Value), ToolError>`，`execute()` 内部调用 `prepare_call()`
- 信心评估: 5
- 步骤:
  - [ ] 从 `execute()` 中提取工具查找和参数解析到 `prepare_call(name, params) -> Result<(&dyn Tool, Value), ToolError>`
  - [ ] `execute()` 改为调用 `prepare_call()` 后执行
  - [ ] `cargo test -p nanobot-tools` 验证

### B. 大工具结果持久化

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/config/src/schema/agent.rs`
- 验收标准: 超过 16000 字符的工具结果被写入磁盘，替换为引用字符串
- 信心评估: 4
- 步骤:
  - [ ] `AgentDefaults` 新增 `max_tool_result_chars: usize`（默认 16000）
  - [ ] 实现 `maybe_persist_tool_result(session_key, tool_call_id, result, max_chars, workspace) -> String`
  - [ ] 在 `re_act` 中工具执行后调用，替换过大结果
  - [ ] 原子写入（先写 .tmp 再 rename）
  - [ ] 新增测试
  - [ ] `cargo test -p nanobot-agent` 验证

### C. 跨模型消息兼容

- 优先级: P1
- 依赖项: 无
- 涉及文件: `crates/context/src/lib.rs`
- 验收标准: `build_messages()` 不产生连续相同角色消息
- 信心评估: 5
- 步骤:
  - [ ] 在 `ContextBuilder::build_messages()` 中，当最后一条历史消息与当前消息角色相同时，合并内容
  - [ ] 新增测试覆盖连续 assistant 消息场景
  - [ ] `cargo test -p nanobot-context` 验证

### D. 工具批处理

- 优先级: P1
- 依赖项: A
- 涉及文件: `crates/tools/src/core.rs`, `crates/tools/src/filesystem.rs`, `crates/agent/src/loop/mod.rs`
- 验收标准: 只读工具并行执行，其余串行
- 信心评估: 4
- 步骤:
  - [ ] `Tool` trait 新增 `fn read_only(&self) -> bool { false }`
  - [ ] `ReadFileTool`、`ListDirTool` 覆盖 `read_only() -> true`
  - [ ] 实现 `partition_tool_batches(calls, registry) -> Vec<Vec<ToolCall>>`：连续 read_only 工具分为一批并行，其余每个单独串行
  - [ ] `re_act` 中用 `futures::future::join_all` 执行并行批次
  - [ ] 新增测试
  - [ ] `cargo test -p nanobot-agent` 验证

### E. 上下文窗口治理

- 优先级: P1
- 依赖项: B
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/config/src/schema/agent.rs`
- 验收标准: 每次 LLM 调用前检查 token 预算，超出时裁剪历史
- 信心评估: 3
- 步骤:
  - [ ] `AgentDefaults` 新增 `context_window_tokens: usize`（默认 128000）
  - [ ] 实现 `snip_history(messages, budget) -> Vec<Message>`：从前往后删除消息直到 token 总量 <= budget，保持工具调用/结果边界完整
  - [ ] 实现 `find_legal_message_start(messages) -> usize`：找到第一个合法的消息起始位置（不能从工具结果消息开始）
  - [ ] 在 `re_act` 循环中 `call_llm` 前调用
  - [ ] fail-open：裁剪失败时使用原始消息
  - [ ] 新增测试
  - [ ] `cargo test -p nanobot-agent` 验证

### F. 近期轮次检查点

- 优先级: P2
- 依赖项: 无
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/session/src/session.rs`
- 验收标准: 工具执行前保存检查点，中断恢复时还原已完成结果
- 信心评估: 3
- 步骤:
  - [ ] 定义检查点数据结构：`RuntimeCheckpoint { assistant_message, pending_tool_calls, completed_results }`
  - [ ] 在 `re_act` 中工具执行前保存检查点到 `session.metadata`
  - [ ] 工具执行后更新检查点（标记已完成）
  - [ ] 正常完成时清除检查点
  - [ ] 在 `process_message` 开始时检查并恢复检查点
  - [ ] 实现重叠去重逻辑
  - [ ] 新增测试
  - [ ] `cargo test -p nanobot-agent` 验证

### G. Provider 流式看门狗（延后）

- 优先级: P2（延后）
- 依赖项: 流式支持（未实现）
- 涉及文件: provider crate
- 验收标准: 延后
- 信心评估: 2
- 步骤:
  - [ ] 等流式支持实现后再规划

## 实现建议

- 每个子任务独立提交，可单独创建 PR
- 所有加固逻辑使用 fail-open 模式：`if let Err(e) = ... { warn!(...); /* 继续使用原始数据 */ }`
- `Message::token_len()` 已存在，可直接用于 token 估算
- 工具批处理用 `futures::future::join_all`
- 检查点序列化用 `serde_json::Value` 存入 `session.metadata`
