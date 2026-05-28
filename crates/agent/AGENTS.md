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
  - `run(self: Arc<Self>, inbound_rx)` -- 启动交互式消息处理循环，内部构造 `LoopHook`
  - `process_direct(content, session_key, channel, chat_id, media, hook)` -- 单次消息处理，接受 `Option<Arc<dyn Hook>>`
  - `re_act(messages, channel, chat_id, hook, scheduled)` -- ReAct 推理-行动循环，接受 `&dyn Hook`
  - `config() -> &AgentDefaults`
- **`Hook`** (trait) -- Agent 生命周期钩子，4 个方法均有默认空实现：
  - `before_iteration(ctx)` -- 每次 LLM 调用前
  - `before_execute_tools(ctx)` -- 工具执行前
  - `after_iteration(ctx)` -- 每轮迭代后
  - `finalize_content(ctx, content) -> Option<String>` -- 最终内容变换
- **`HookCtx<'a>`** -- 钩子上下文，携带 `content: &str`、`tool_calls: &[ToolCall]`、`usage: Option<&Usage>`
- **`CompositeHook`** -- 组合多个 hook，异步方法有错误隔离，`finalize_content` 串行管道传递
- **`LoopHook`** -- 交互式循环钩子，通过 mpsc channel 发送思考内容和工具提示
- **`NoopHook`** -- 空操作钩子，所有方法使用默认空实现
- **`Command`** (trait) -- `async fn run(self, msg, session_key) -> Result<String, String>`
- Re-export: `InboundMessage`, `OutboundMessage`, `AgentDefaults`, `Message`, `Provider`, `Session`, `SessionManager`, `strip_think`

## 内部依赖

config, context, mcp, provider, tools, session, memory, channels, cron, subagent
