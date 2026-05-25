# 需求

## 目标与背景

Python 版 nanobot 有一个 `message` tool，允许 LLM 主动向指定 channel/chat 发送消息并附带媒体文件。这是钉钉/飞书媒体发送功能的前置依赖——没有 message tool，LLM 无法将文件路径附加到出站消息的 `media` 字段。

nanobot-rs 当前没有 message tool。`OutboundMessage.media` 字段存在但从未被填充。LLM 的响应只有纯文本，无法携带媒体附件。

## 方案比较（强制）

### 方案 1: 发送 + 媒体，不含抑制逻辑（最小可行版）

- 思路: 实现 `MessageTool`，支持 `content`、`channel`、`chat_id`、`media` 参数。通过 `mpsc::Sender<OutboundMessage>` 发送消息。不实现同目标抑制逻辑
- 优点:
  - 实现简单，`MessageTool` 无状态，`process_message()` 不改
  - 最终回复与 message tool 发送的内容不同，两条都发给用户可接受
- 缺点:
  - 当 message tool 发送到同一 chat 时，用户收到两条消息（message tool 的内容 + agent 最终回复）
- 工作量估算: M

### 方案 2: 含同目标抑制逻辑（理想架构）

- 思路: 在方案 1 基础上，当 message tool 已向当前 channel+chat_id 发送过消息时，抑制 agent 的最终回复
- 优点:
  - 与 Python 版完全对齐
  - 无重复消息
- 缺点:
  - 需要在 `process_message()` 中检查消息历史或通过其他机制传递抑制信号
  - 复杂度增加
- 工作量估算: L

### 推荐

方案 1（用户选择）。最终回复与 message tool 发送的内容不同，两条消息各有用途（内容 + 确认），不需要抑制。

## 架构设计

### outbound_tx 传递方式

`outbound_tx: mpsc::Sender<OutboundMessage>` 作为 `AgentLoop::new()` 的**必填参数**：

- `AgentLoop` 存储 `outbound_tx`，构造时注册 `MessageTool`（传入 `outbound_tx.clone()`）
- `run()` 移除 `outbound_tx` 参数，使用 `self.outbound_tx`
- `process_direct()` 不变（返回 `OutboundMessage`），message tool 通过 `self.outbound_tx` 发送

调用方适配：
- **CLI 单次模式**：创建 `(tx, rx)` channel，`tx` 传入 `new()`，`process_direct()` 后 drain `rx` 打印
- **CLI 交互模式**：已有 `(tx, rx)`，`tx` 从 `run()` 参数移到 `new()` 参数
- **Gateway**：已有 `(tx, rx)`，`tx` 从 `run()` 参数移到 `new()` 参数
- **测试**：每处创建 `let (tx, _rx) = mpsc::channel(100)` 并传入

## 功能需求列表

### 核心功能

1. 重构 `AgentLoop`：`outbound_tx` 从 `run()` 参数移到 `new()` 必填参数，存储为字段
2. 新增 `MessageTool` 实现 `Tool` trait（无状态）
   - name: `"message"`
   - 参数: `content`（必填）、`channel`（可选，默认当前 channel）、`chat_id`（可选，默认当前 chat_id）、`media`（可选，文件路径数组）
3. `MessageTool` 持有 `mpsc::Sender<OutboundMessage>` 和 workspace 路径，用于发送消息和解析媒体路径
4. 媒体路径解析：本地路径相对于 workspace 解析，URL 直接透传
5. 适配所有调用方（CLI 单次/交互、gateway、测试）

### 扩展功能

- 无（buttons 参数、同目标抑制逻辑留作后续）

## 非功能需求

- **安全性**：当 `restrict_to_workspace` 为 true 时，媒体路径必须在 workspace 内
- **健壮性**：媒体路径不存在时返回错误信息但不阻止文本消息发送

## 边界与不做事项

- 不实现同目标抑制逻辑（最终回复始终发送）
- 不实现 `buttons` 参数
- 不修改 channel 层的 `send()` 方法（媒体发送由各 channel 自行处理，本次只确保 `OutboundMessage.media` 被正确填充）

## 假设与约束

- **技术假设**：所有调用方在 `AgentLoop::new()` 前创建 `mpsc::channel`，传入 `tx` 端
- **依赖**：message tool 发送的 `OutboundMessage` 通过现有的 `ChannelManager` 路由（gateway 模式），或由 CLI 消费打印（CLI 模式）

## 待确认事项

无
