# 需求

## 目标与背景

为 Anthropic provider 实现 prompt caching，在多轮对话中大幅降低 token 消耗。对齐上游 HKUDS/nanobot PR #1109。

当前 nanobot-rs 的 Anthropic provider 每次请求都完整发送系统提示词和对话历史，没有利用 Anthropic 的 prompt caching 能力。在多轮对话中，不断增长的历史消息每次都重新处理，造成不必要的 token 消耗。

上游的做法：在请求中放置两个 `cache_control` 断点：
1. **系统消息** — 缓存静态系统提示词
2. **倒数第二条消息** — 缓存对话历史前缀，使多轮对话中只有最新一条消息需要重新处理

预期效果：多轮对话历史 cache 命中率从 0% 提升至 ~90%+，cache 读取成本仅为正常输入的 0.1x。

## 方案比较（强制）

### 方案 1: 显式 per-block cache_control 断点（最小可行版）

- 思路: 在 `ContentBlock` 变体上添加可选的 `cache_control` 字段。`AnthropicRequest.system` 从 `Option<String>` 改为 `Option<Vec<SystemBlock>>`（支持 cache_control）。在 `convert_messages()` 后对系统消息和倒数第二条消息注入 `cache_control: {"type": "ephemeral"}`。
- 优点: 精确控制断点位置，与上游行为一致；支持两个断点
- 缺点: 需要改 `system` 字段类型和 `ContentBlock` 结构
- 工作量估算: S

### 方案 2: 顶层 cache_control（理想架构）

- 思路: 在 `AnthropicRequest` 上添加顶层 `cache_control` 字段，API 自动在最后一个可缓存 block 上放置断点
- 优点: 改动最小（加一个字段）
- 缺点: 只支持一个断点，无法同时缓存系统提示词和历史前缀
- 工作量估算: S

### 推荐

方案 1。两个断点是性能收益的关键：断点 1 缓存系统提示词（每次对话固定），断点 2 缓存历史前缀（每轮只有最新消息是新的）。

## 功能需求列表

### 核心功能

1. **ContentBlock 支持 cache_control** -- `Text` 和 `ToolResult` 变体新增可选 `cache_control` 字段
2. **system 字段改为 block 数组** -- `AnthropicRequest.system` 从 `Option<String>` 改为 `Option<Vec<SystemBlock>>`，支持在系统提示词上附加 `cache_control`
3. **注入 cache 断点** -- 在 `chat()` 中构建请求后、发送前，对系统消息 block 和倒数第二条消息的最后一个 content block 注入 `cache_control: {"type": "ephemeral"}`
4. **条件保护** -- 仅当消息数 >= 3 时才添加第二个断点（避免消息太少时无效标记）

### 不纳入本次

- cache 命中率统计（`cache_creation_input_tokens` / `cache_read_input_tokens`）— 可后续在 Usage 中扩展
- 可配置的 cache TTL（当前固定 `ephemeral`，5 分钟）

## 非功能需求

- 向后兼容：不改变 `Provider` trait 签名；不影响 OpenAI provider
- 性能：cache 写入成本 1.25x，读取成本 0.1x，多轮对话净节省显著
- 测试：验证 cache_control 正确注入到系统消息和倒数第二条消息

## 边界与不做事项

- 不实现 OpenAI 的 prompt caching（OpenAI 自动缓存，无需客户端标记）
- 不实现 cache TTL 配置
- 不实现 beta header 注入（Anthropic prompt caching 已 GA，不需要 beta header）

## 待确认事项

无
