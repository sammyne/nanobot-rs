# subagent crate

子代理任务管理器，创建和管理后台轻量级代理实例。

## 架构

```
┌────────────────────────────────┐
│ SpawnTool (Tool, 注册为 "spawn")│
└───────────────┬────────────────┘
                ▼
┌────────────────────────────────────────────┐
│          SubagentManager                   │
│                                            │
│  session_tasks: Mutex<HashMap<             │
│    session_key, Vec<(task_id, JoinHandle)>>>│
│                                            │
│  spawn(task)                               │
│    ├── tokio::spawn 后台循环               │
│    │   (最多 15 次 LLM 迭代)               │
│    │   独立 ToolRegistry                   │
│    │                                       │
│    └── 完成后 ──► bus(mpsc) ──► 主 agent   │
│                                            │
│  cancel_by_session(key)                    │
│    └── abort 所有 JoinHandle               │
└────────────────────────────────────────────┘
```

## 关键类型

- **`SubagentManager<P: Provider>`** -- 创建和管理后台 agent 任务，按 session 跟踪
  - `new(provider, workspace, bus, temperature, max_tokens) -> Arc<Self>`
  - `spawn(task, session_key, label, channel, chat_id)` -- 创建后台任务
  - `cancel_by_session(session_key) -> usize` -- 取消指定会话的所有任务
  - `get_running_count() -> usize`
- **`Task`** -- `id`, `description`, `label`, `channel`, `chat_id`
- **`SpawnTool<P>`** -- 实现 `Tool` trait，注册为 "spawn"，允许 agent 通过 function calling 创建子代理
- **`SubagentError`** (enum) -- `Provider`, `Tool`, `Timeout`, `Config`, `InvalidParam`, `Internal`
- **`SubagentResult<T>`** -- `Result<T, SubagentError>` 类型别名

## 内部依赖

provider, tools, channels, config
