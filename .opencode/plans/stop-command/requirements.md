# 需求

## 目标与背景

实现 `/stop` 命令，允许用户取消正在进行的主任务和所有关联的后台子代理。当前 nanobot-rs 只有 `/new` 和 `/help` 两个命令，没有任何取消机制。`run()` 循环是顺序处理的，无法在处理消息期间响应新命令；`SubagentManager` 的 `tokio::spawn()` 返回的 `JoinHandle` 被直接丢弃，无法中止已启动的子代理。

## 方案比较

### 方案 1: 顺序 /stop（仅取消子代理）

- 思路: `/stop` 作为普通命令通过 `try_handle_cmd` 处理，只取消后台子代理，不中断正在进行的主任务
- 优点: 改动最小，不需要重构 `run()` 循环
- 缺点: `/stop` 只能在两条消息之间处理，无法中断正在进行的 LLM 调用；用户体验不完整

### 方案 2: 并发 /stop（取消主任务 + 子代理）

- 思路: 重构 `run()` 将 `process_message` 作为 tokio task 启动，用 `tokio::select!` 同时监听任务完成和新消息。`/stop` 到达时 abort 主任务并取消所有子代理
- 优点: 完整的取消体验，`/stop` 可以在处理期间中断 LLM 调用
- 缺点: 需要将 `run()` 签名改为 `self: Arc<Self>`，影响调用方；需要处理非 /stop 消息在处理期间到达的情况

### 推荐

推荐方案 2。gateway 模式下 `AgentLoop` 已经用 `Arc` 包装，CLI 模式只需加一层 `Arc::new()`。`tokio::select!` 是 Rust 异步取消的标准模式，改动可控。

## 功能需求列表

### 核心功能

1. **SubagentManager 会话级任务追踪**：`spawn()` 存储 `JoinHandle`，按 session_key 分组追踪运行中的子代理任务
2. **SubagentManager::cancel_by_session()**：中止指定会话的所有子代理任务，返回取消数量
3. **SpawnTool 传递 session_key**：从 `ToolContext` 的 channel + chat_id 构造 session_key，传给 `spawn()`
4. **AgentLoop 存储 SubagentManager 引用**：新增 `subagent_manager` 字段，供命令处理器访问
5. **StopCmd 命令**：调用 `cancel_by_session()` 取消子代理，返回确认消息
6. **run() 重构**：将 `process_message` 作为 tokio task 启动，用 `tokio::select!` 监听 `/stop`，到达时 abort 主任务
7. **HelpCmd 更新**：帮助文本中添加 `/stop` 命令说明

### 扩展功能

- 无

## 非功能需求

- **兼容性**：`process_direct()` 不受影响（单次调用模式，不经过 `run()` 循环）
- **安全**：`JoinHandle::abort()` 在下一个 `.await` 点取消 future，不会造成数据损坏（session 保存在 abort 之后不会执行）
- **测试要求**：为 `cancel_by_session()` 添加单元测试；为 `/stop` 命令添加集成测试

## 边界与不做事项

- 不实现 `re_act()` 内部的协作式取消（`CancellationToken` 检查）——`JoinHandle::abort()` 已足够
- 不实现跨会话的全局 `/stop`——只取消当前会话的任务
- 不修改 `process_direct()` 路径（CLI 单次调用模式）

## 假设与约束

- **技术假设**：`AgentLoop<P>` 满足 `Send + Sync`（已在测试中通过 `Arc` 跨任务使用验证）
- **依赖**：`tokio_util::sync::CancellationToken` 已是项目依赖（gateway 的 HeartbeatService 使用）

## 待确认事项

- `/stop` 在没有正在运行的任务时，是否仍应返回成功消息（如 "Nothing to stop."）？暂定返回 "Stopped."，与 Python 版行为一致
