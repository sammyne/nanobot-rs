# 需求

## 目标与背景

新增 `/status` 斜杠命令，显示运行时诊断信息。对齐上游 HKUDS/nanobot PR #1985。

当前 nanobot-rs 支持 `/help`、`/new`、`/stop`、`/restart` 四个斜杠命令，缺少运行时状态查看能力。`/status` 可以帮助用户和管理员快速了解 bot 的运行状况。

## 方案比较（强制）

### 方案 1: 仅展示已有信息（最小可行版）

- 思路: `/status` 展示无需额外基础设施即可获取的信息：版本号、模型名称、会话消息数、运行时长。不捕获 token 用量（需改 provider trait）。
- 优点: 改动极小，不涉及 provider 层
- 缺点: 缺少 token 用量信息，与上游不完全对齐
- 工作量估算: S

### 方案 2: MeteredMessage + Deref 完整实现（理想架构）

- 思路: 新增 `Usage` 结构体和 `MeteredMessage`（包含 `Message` + `Option<Usage>`），为 `MeteredMessage` 实现 `Deref<Target=Message>`。`Provider::chat()` 返回类型从 `Result<Message>` 改为 `Result<MeteredMessage>`。调用方通过 Deref 透明访问 Message 方法，只有需要 usage 的地方才显式访问 `.usage`。
- 优点: 与上游完全对齐；token 用量对成本监控有价值；Deref 使大部分调用方零改动
- 缺点: 新增两个类型（`Usage`、`MeteredMessage`）
- 工作量估算: M

### 推荐

方案 2。`MeteredMessage` + `Deref` 使改动最小化，大部分调用方无需修改。

## 功能需求列表

### 核心功能

1. **Usage + MeteredMessage** -- 在 provider/base 中新增 `Usage { input_tokens, output_tokens }` 和 `MeteredMessage { message: Message, usage: Option<Usage> }`，实现 `Deref<Target=Message>`。`Provider::chat()` 返回类型从 `Result<Message>` 改为 `Result<MeteredMessage>`
2. **Provider 捕获 usage** -- Anthropic 和 OpenAI provider 实现中解析 API 响应的 usage 字段，填入 `MeteredMessage.usage`
3. **AgentLoop 运行时状态** -- 新增 `start_time: Instant` 和 `last_usage: Option<Usage>` 字段，在 `call_llm()` 后从 `MeteredMessage.usage` 更新 `last_usage`
4. **StatusCmd** -- 新增 `/status` 命令，展示：版本号、模型、最近 token 用量（in/out）、会话消息数、运行时长
5. **命令注册** -- 在 `try_handle_cmd()` 中注册 `/status`，更新 `/help` 文本

## 非功能需求

- 向后兼容：`MeteredMessage` 通过 `Deref<Target=Message>` 透明兼容，subagent 等调用方无需修改；`usage` 为 `Option`，不支持的场景返回 None
- 测试：StatusCmd 单元测试；provider usage 解析测试

## 边界与不做事项

- 不实现累计 token 用量统计（仅记录最近一次调用）
- 不实现 context window 占比计算（需要 tokenizer，复杂度高）
- 不实现 Telegram 等通道的命令菜单注册（nanobot-rs 无 Telegram 通道）

## 待确认事项

无
