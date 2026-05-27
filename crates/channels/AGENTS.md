# channels crate

消息通道抽象及实现（钉钉、飞书、Email）。

## 架构

```
┌──────────────────────────────────────────┐
│            ChannelManager                │
│  channels: HashMap<name, Arc<dyn Channel>>│
│                                          │
│  ┌──────────┐  ┌────────┐  ┌─────────┐  │
│  │ DingTalk │  │ Feishu │  │  Email  │  │
│  │Stream SDK│  │WebSocket│  │IMAP+SMTP│  │
│  └────┬─────┘  └───┬────┘  └────┬────┘  │
│       └────────────┼────────────┘        │
│                    │                     │
│           Channel trait                  │
│       start / stop / send               │
└────────────────────┼─────────────────────┘
                     │
     ┌───────────────┼───────────────┐
     │ 入站                          │ 出站
     ▼                               ▼
┌─────────┐                   ┌───────────┐
│inbound  │──► AgentLoop      │outbound   │◄── AgentLoop
│  _tx    │                   │  _rx      │
└─────────┘                   └─────┬─────┘
                                    │
                          按 msg.channel 路由
                                    │
                                    ▼
                            Channel.send()
```

## 关键类型

- **`Channel`** (trait) -- `start()`, `stop()`, `send(msg)`, `is_running()`, `name()`
- **`InboundMessage`** -- `channel`, `sender_id`, `chat_id`, `content`, `media`, `metadata`；`session_key()` 方法
- **`OutboundMessage`** -- `channel`, `chat_id`, `content`, `media`, `metadata`；`progress()`, `is_progress()`, `is_tool_hint()` 方法
- **`ChannelManager`** -- 管理通道实例，路由出站消息
  - `new(config, outbound_rx, inbound_tx)` -- 创建并配置通道
  - `start_all()` / `stop_all()` -- 启动/停止所有通道
  - `route_message(msg)` -- 路由出站消息到目标通道
- **`ChannelError`** (enum) -- `StartFailed`, `StopFailed`, `SendFailed`, `Config`, `Api`, `Auth`, `Network` 等
- **`DingTalk`** -- 钉钉 Stream SDK 通道实现
- **`Feishu`** -- 飞书 WebSocket 通道实现
- **`Email`** -- IMAP 轮询收件 + SMTP 发件通道实现；`fetch_messages_between_dates()` 支持按日期范围查询历史邮件

## 内部依赖

config
