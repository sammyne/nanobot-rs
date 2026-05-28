# 需求

## 目标与背景

为飞书通道实现基于 CardKit 的流式输出，在 agent 处理过程中实时更新卡片内容（打字机效果），而非每条进度消息都创建新卡片。对齐上游 HKUDS/nanobot PR #2382。

当前飞书通道的每条消息（包括工具提示等进度消息）都创建一个独立的交互卡片，导致 agent 执行多步工具调用时产生大量消息刷屏。

上游的完整方案包含 Provider 流式 API + Agent 流式回调 + Channel 流式显示三层。nanobot-rs 当前 Provider 不支持流式响应，完整方案改动过大。

## 方案比较（强制）

### 方案 1: Feishu 内部流式卡片（最小可行版）

- 思路: 仅在 Feishu 通道的 `send()` 方法内部实现流式卡片逻辑，不改 Channel trait、Agent loop 或 Provider。利用现有的 `is_progress()` 元数据区分进度消息和最终响应：进度消息累积到流式卡片，最终响应关闭流式模式并替换为完整内容。
- 优点: 改动集中在 Feishu 通道内部（1 个文件）；不影响其他通道和上层架构；立即可用
- 缺点: 不是真正的 token-by-token 流式（需要 Provider 流式 API 支持）；仅对工具执行阶段的进度消息有效
- 工作量估算: M

### 方案 2: 全栈流式（理想架构）

- 思路: Provider 新增 `chat_stream()` 方法，Agent loop 新增流式回调，Channel trait 新增 `send_delta()`，ChannelManager 新增 delta 路由和合并。
- 优点: 真正的 token-by-token 流式输出，与上游完全对齐
- 缺点: 跨 5+ 个 crate 的大规模改动；Provider 流式 API 需要重新设计请求/响应管道
- 工作量估算: L

### 推荐

方案 1。在不改动上层架构的前提下，显著改善飞书用户体验（进度消息不再刷屏）。Provider 流式 API 可作为独立需求后续迭代。

## 功能需求列表

### 核心功能

1. **FeishuConfig 新增 streaming 配置** -- `streaming: bool`（默认 true）
2. **StreamBuf 状态管理** -- 每个 chat_id 维护一个流式缓冲区（text、card_id、sequence、last_edit），存储在 `Feishu` 结构体中
3. **创建流式卡片** -- 调用 CardKit API 创建 `streaming_mode: true` 的卡片，包含一个空的 markdown 元素
4. **更新卡片内容** -- 调用 CardKit API 更新 markdown 元素内容，递增 sequence，节流 ~0.5s
5. **关闭流式模式** -- 调用 CardKit API 设置 `streaming_mode: false`
6. **send() 路由逻辑** -- 进度消息走流式路径（创建/更新卡片），最终响应关闭流式卡片并发送完整内容

### 不纳入本次

- Provider 流式 API（`chat_stream()`）
- Channel trait 新增 `send_delta()` 方法
- Agent loop 流式回调
- 其他通道的流式支持

## 非功能需求

- 向后兼容：`streaming: false` 时行为与当前完全一致
- 容错：CardKit API 调用失败时回退为普通交互卡片
- 性能：更新节流 ~0.5s，避免 API 限流

## 边界与不做事项

- 不实现 token-by-token 流式（需要 Provider 流式 API）
- 不改 Channel trait 签名
- 不改 Agent loop 或 ProgressTracker

## 假设与约束

- 飞书应用需要 `cardkit:card:write` 权限
- CardKit API 端点：`POST /open-apis/cardkit/v1/cards`（创建）、`PUT /open-apis/cardkit/v1/cards/{card_id}/elements/{element_id}/content`（更新）、`PATCH /open-apis/cardkit/v1/cards/{card_id}/settings`（关闭流式）

## 待确认事项

无
