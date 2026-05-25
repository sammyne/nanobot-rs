# 需求

## 目标与背景

HKUDS/nanobot PR #1314 修复了两个导致 session 历史被"毒化"的问题：

1. **`content: null` 的 assistant 消息**：LLM 返回 `content: null` 且无 `tool_calls` 时，某些 provider 在后续请求中拒绝历史中的 nil content，导致永久 400 错误循环
2. **错误响应入历史**：`finish_reason == "error"` 的响应被保存到 session 历史，导致错误字符串在每次后续请求中被重放

nanobot-rs 当前状态：
- **场景 2 已正确处理**：`provider.chat()` 返回 `Err` 时，`re_act()` 通过 `?` 传播，`process_message()` early return 不调用 `save_turn()`
- **场景 1 未处理**：OpenAI provider 用 `unwrap_or_default()` 将 `null` 转为空字符串 `""`，空 assistant 消息会被持久化。虽然不会导致 400 错误循环，但空消息浪费 token 且可能困惑模型

## 方案比较（强制）

### 方案 1: 在 re_act 层替换空内容（最小可行版，推荐）

- 思路: 在 `re_act()` 中，当 assistant 响应的 content 为空且无 tool_calls 时，将 content 替换为 `"(empty)"`
- 优点:
  - 改动集中在一处，provider 无关
  - 与 Python 版行为一致
- 缺点:
  - 修改了 LLM 原始响应
- 工作量估算: S

### 方案 2: 在 provider 层替换空内容（理想架构）

- 思路: 在 OpenAI 和 Anthropic provider 的 `chat()` 方法中，检测空 content 并替换
- 优点:
  - 在数据源头处理，下游无需关心
- 缺点:
  - 需要修改两个 provider
  - provider 层不应修改 LLM 原始语义
- 工作量估算: S

### 推荐

方案 1。在 `re_act()` 层处理更合理——这是业务逻辑层面的防御，不应由 provider 层承担。Python 版也是在 agent loop 层处理的。

## 功能需求列表

### 核心功能

1. 在 `re_act()` 中，当无 tool_calls 的 assistant 响应 content 为空时，将 content 替换为 `"(empty)"` 再 push 到 messages

### 扩展功能

- 无

## 非功能需求

- **兼容性**：不影响正常的非空响应和带 tool_calls 的空内容响应（tool_calls 场景中空 content 是正常的）

## 边界与不做事项

- 不处理 `finish_reason == "error"` 场景（nanobot-rs 已通过 `Err` 传播正确处理）
- 不修改 provider 层代码

## 假设与约束

- **技术假设**：空 content + 无 tool_calls 的 assistant 消息是异常情况，用 `"(empty)"` 替代不会影响后续对话质量

## 待确认事项

无
