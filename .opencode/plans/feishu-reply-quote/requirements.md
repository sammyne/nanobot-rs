# 需求

## 目标与背景

飞书群聊中，bot 的回复作为独立消息发送，用户难以关联"这条回复是针对哪条消息的"。飞书 API 提供 `im.v1.message.reply` 端点，可以将回复线程化到原始消息下方（显示引用气泡）。

Python 版本（PR #1963）实现了此功能，通过 `reply_to_message` 配置控制。

**Rust 版本现状**：
- `send()` 始终调用 `im.v1.message.create`（独立消息）
- `process_message()` 提取了 `message_id` 但仅用于 emoji reaction，未传递到 `InboundMessage.metadata`
- SDK 已有 `reply_typed()` 方法可用

## 方案比较（强制）

### 方案 1: metadata 传递 message_id + send() 条件分发（最小可行版 + 理想架构）

- 思路: `process_message()` 将 `message_id` 存入 `InboundMessage.metadata`；`send()` 检查 config + metadata 决定用 reply 还是 create API
- 优点: 改动集中在 feishu channel 内部，不影响其他模块；metadata 机制已存在
- 缺点: 无
- 工作量估算: S

### 方案 2: Channel trait 新增 reply 方法

- 思路: 在 `Channel` trait 上新增 `reply(msg, reply_to_id)` 方法
- 优点: 类型安全，reply 语义显式
- 缺点: 改动 trait 影响所有 channel 实现；agent loop 需要感知 reply 语义；过度设计
- 工作量估算: M

### 推荐

方案 1。reply 是飞书特有的 UX 优化，不需要抽象到 trait 层。

## 功能需求列表

### 核心功能

1. `FeishuConfig` 新增 `reply_to_message: bool`（默认 `false`）
2. `process_message()` 将 `message_id` 存入 `InboundMessage.metadata["message_id"]`
3. `send()` 中：当 `reply_to_message=true` 且 metadata 有 `message_id` 且非 progress 消息时，首个内容块用 `reply_typed()` API，后续块用 `send_typed()`
4. `reply_typed()` 失败时回退到 `send_typed()`

## 非功能需求

- **向后兼容**：`reply_to_message` 默认 `false`，不影响现有行为

## 边界与不做事项

- 不做入站回复上下文（用户回复某条消息时获取被引用消息内容）— 后续按需实现
- 不修改 Channel trait

## 待确认事项

- 无
