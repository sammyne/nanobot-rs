# 需求

## 目标与背景

Anthropic 的 extended thinking 功能要求 assistant 消息中的 thinking block 在后续请求中原样回传。nanobot-rs 的 Anthropic provider 已正确提取 thinking 数据并存入 `Message::Assistant.thinking` 字段，`convert_messages()` 也能正确将其回传给 API。但 agent loop（`re_act()`）和 subagent manager（`run_subagent()`）在将 LLM 响应追加到消息历史时，总是构造新的 `Message` 对象（`thinking: None`），导致 thinking 数据被丢弃。

**后果**：
- 使用 Anthropic extended thinking 模式时，多轮工具调用场景中 thinking block 丢失，可能导致 API 返回错误或行为异常
- thinking 数据不会被持久化到 session JSONL，历史会话恢复后也无法回传
- `convert_messages()` 中的 thinking 回传逻辑实际上是死代码

## 方案比较

### 方案 1: 直接推入原始 response

- 思路: `re_act()` 和 `run_subagent()` 中不再构造新 `Message`，直接将 `call_llm()` / `provider.chat()` 返回的 `response`（已经是 `Message::Assistant`）push 到 messages 向量中
- 优点: 改动最小，不引入新 API，天然保留 response 上的所有字段（包括未来可能新增的字段）
- 缺点: 需要调整变量生命周期——`response.tool_calls()` 返回借用，push response 后借用失效，需先提取所需数据再消费 response

### 方案 2: 提取 thinking 字段传入现有构造函数

- 思路: 从 response 中取出 `thinking()` 值，调用 `Message::assistant_with_thinking()` 构造新消息
- 优点: 代码结构变化小，保持"构造新消息"的模式
- 缺点: 每个构造点都需要额外的 thinking 提取逻辑；如果未来 Message 新增其他 provider 特定字段，每个点都要再改一次

### 推荐

推荐方案 1。改动量最小（每处约 3-5 行），且对未来扩展友好——任何新增到 `Message` 的字段都会自动保留，无需逐点修改。

## 功能需求列表

### 核心功能

- `AgentLoop::re_act()` 在有工具调用时，将原始 response 直接推入 messages，而非构造 `Message::assistant_with_tools()`
- `AgentLoop::re_act()` 在无工具调用时，将原始 response 直接推入 messages，而非构造 `Message::assistant()`
- `SubagentManager::run_subagent()` 在有工具调用时，将原始 response 直接推入 messages，而非构造 `Message::assistant_with_tools()`

### 不修改的位置

- `re_act()` 达到最大迭代次数时的 warning 消息（line 272）：这是合成消息，不来自 LLM 响应，保持 `Message::assistant()` 构造
- `run_subagent()` 无工具调用时的 `final_result` 提取（line 216）：此处只提取文本内容用于返回，不推入 messages，无需修改

## 非功能需求

- **兼容性**：修改不影响 OpenAI provider 的行为（OpenAI provider 返回的 Message 的 thinking 字段始终为 None，push 原始 response 等价于当前行为）
- **可维护性**：不引入新的 API 或抽象
- **测试要求**：为 `re_act()` 添加单元测试，验证 thinking 数据在多轮工具调用中被保留到最终 messages 列表

## 边界与不做事项

- 不处理 OpenAI provider 的 `reasoning_content` 支持（属于独立功能）
- 不修改 `convert_messages()` 或 Anthropic provider 的响应解析逻辑（已正确工作）
- 不修改 session 持久化逻辑（thinking 字段已有 `#[serde(default, skip_serializing_if = "Option::is_none")]`，无需额外处理）

## 假设与约束

- **技术假设**：`Provider::chat()` 返回的 `Message` 在 tool_calls 非空时始终为 `Message::Assistant` 变体
- **环境约束**：Rust >= 1.93，使用 `let-else` 等新语法特性

## 待确认事项

无
