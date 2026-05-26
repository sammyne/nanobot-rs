# agent crate

Agent 核心循环，接收消息、构建上下文、调用 LLM、执行工具并返回响应。

## 架构

```
┌───────────────┐
│InboundMessage │
└───────┬───────┘
        ▼
┌──────────────────┐     ┌──────────────────────────┐
│process_message() ├────►│process_system_message()  │ channel == "system"
└───────┬──────────┘     └──────────────────────────┘
        │
        ├── /help /new /stop ──► ┌──────────────────┐
        │                        │Command dispatch  │
        │                        └──────────────────┘
        │
        │  普通消息               ┌──────────────────┐
        ├── tokio::spawn ───────►│try_consolidate() │ 并行记忆整合
        │                        └──────────────────┘
        ▼
┌──────────────────────────────────┐
│          re_act() 循环           │
│                                  │
│  ┌───────────┐   ┌────────────┐ │
│  │call_llm() ├──►│tool_calls? │ │
│  └───────────┘   └──┬─────┬──┘ │
│        ▲         yes│     │no  │
│        │            ▼     │    │
│        │     ┌────────┐   │    │
│        └─────┤execute │   │    │
│              │tools   │   │    │
│              └────────┘   │    │
└───────────────────────────┼────┘
                            ▼
                    ┌─────────────┐
                    │ ReActResult │
                    └─────────────┘
```

交互式模式（`run()`）支持 `/stop` 中断：主处理任务在 `tokio::spawn` 中运行，通过 `tokio::select!` 同时监听新消息，收到 `/stop` 时 abort 主任务并取消子代理。

## 关键类型

- **`AgentLoop<P: Provider>`** -- 核心引擎，持有 provider、config、sessions、tool_registry、context、subagent_manager
  - `new(provider, config, cron_service, subagent_manager, tools_config)` -- 初始化工具注册表（含 MCP/cron/spawn）、会话管理器、上下文构建器
  - `run(self: Arc<Self>, inbound_rx, outbound_tx)` -- 启动交互式消息处理循环
  - `process_direct(content, session_key, channel, chat_id, media, on_progress)` -- 单次消息处理
  - `re_act(messages, channel, chat_id, on_progress)` -- ReAct 推理-行动循环（调用 LLM -> 执行工具 -> 重复）. Includes token budget check: trims history messages when total tokens exceed max_input_tokens
  - `config() -> &AgentDefaults`
- **`ProgressTracker`** (trait) -- `async fn track(content, is_tool_hint) -> Result<()>`
- **`ChannelProgressTracker`** -- 通过 mpsc channel 发送进度的默认实现
- **`Command`** (trait) -- `async fn run(self, msg, session_key) -> Result<String, String>`
- Re-export: `InboundMessage`, `OutboundMessage`, `AgentDefaults`, `Message`, `Provider`, `Session`, `SessionManager`

## 内部依赖

config, context, mcp, provider, tools, session, memory, channels, cron, subagent
