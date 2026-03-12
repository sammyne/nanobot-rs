# 需求文档：HeartbeatService 集成到 Gateway 命令

## 引言

本文档描述将 HeartbeatService 集成到 nanobot Rust 版本的 gateway 命令中的需求。HeartbeatService 是一个定期检查任务执行的服务，它通过读取工作区中的 `HEARTBEAT.md` 文件，使用 LLM 决定是否需要执行任务，并通过回调函数执行任务并通知结果。

该需求参考了 Python 版本的实现，确保 Rust 版本的 gateway 命令具有相同的功能和行为。

## 需求

### 需求 1：在 GatewayCmd 中初始化 HeartbeatService

**用户故事：** 作为一名系统开发者，我希望在 gateway 命令启动时创建并初始化 HeartbeatService 实例，以便系统能够定期检查并执行任务。

#### 验收标准

1. WHEN 用户执行 `nanobot gateway` 命令 THEN 系统 SHALL 创建 HeartbeatService 实例
2. WHEN 创建 HeartbeatService 时 THEN 系统 SHALL 使用配置文件中的 heartbeat 配置（enabled 和 interval_seconds）
3. WHEN 创建 HeartbeatService 时 THEN 系统 SHALL 传入工作区路径、LLM provider 和配置
4. WHEN 创建 HeartbeatService 时 THEN 系统 SHALL 设置 on_execute 回调函数（为空，后续配置）
5. WHEN 创建 HeartbeatService 时 THEN 系统 SHALL 设置 on_notify 回调函数（为空，后续配置）

### 需求 2：配置 on_execute 回调函数

**用户故事：** 作为一名系统开发者，我希望配置 HeartbeatService 的 on_execute 回调，以便心跳检测确定需要执行任务时，能够通过 AgentLoop 执行这些任务。

#### 验收标准

1. WHEN 配置 on_execute 回调 THEN 系统 SHALL 接收任务描述字符串作为参数
2. WHEN on_execute 回调被调用 THEN 系统 SHALL 调用 AgentLoop 的 process_direct 方法执行任务
3. WHEN 调用 process_direct 时 THEN 系统 SHALL 使用 "heartbeat" 作为 session_key
4. WHEN 调用 process_direct 时 THEN 系统 SHALL 选择合适的 channel 和 chat_id（使用 pick_heartbeat_target 逻辑）
5. WHEN process_direct 完成时 THEN 系统 SHALL 返回执行结果

### 需求 3：配置 on_notify 回调函数

**用户故事：** 作为一名系统开发者，我希望配置 HeartbeatService 的 on_notify 回调，以便任务执行完成后，能够将结果通知给用户。

#### 验收标准

1. WHEN 配置 on_notify 回调 THEN 系统 SHALL 接收任务执行结果作为参数
2. WHEN on_notify 回调被调用 THEN 系统 SHALL 使用 pick_heartbeat_target 选择通知目标
3. WHEN 目标是 "cli" 时 THEN 系统 SHALL 跳过通知（无外部渠道）
4. WHEN 目标不是 "cli" 时 THEN 系统 SHALL 通过消息总线发送 OutboundMessage
5. WHEN 发送消息时 THEN 系统 SHALL 包含任务执行结果内容

### 需求 4：实现 pick_heartbeat_target 辅助函数

**用户故事：** 作为一名系统开发者，我希望有一个函数能够选择心跳通知的目标渠道和聊天ID，以便任务执行结果能够发送到合适的位置。

#### 验收标准

1. WHEN 调用 pick_heartbeat_target 时 THEN 系统 SHALL 从已启用的渠道中选择目标
2. WHEN 存在非内部（cli/system）会话时 THEN 系统 SHALL 优先选择最近更新的会话
3. WHEN 不存在非内部会话时 THEN 系统 SHALL 返回 ("cli", "direct") 作为默认值
4. WHEN 返回结果时 THEN 系统 SHALL 返回 (channel, chat_id) 元组

### 需求 5：在服务启动时启动 HeartbeatService

**用户故事：** 作为一名系统开发者，我希望在 gateway 启动后自动启动 HeartbeatService，以便系统开始定期检查任务。

#### 验收标准

1. WHEN gateway 启动时 THEN 系统 SHALL 在 CronService 启动后启动 HeartbeatService
2. WHEN 启动 HeartbeatService 时 THEN 系统 SHALL 使用异步方式启动
3. WHEN HeartbeatService 被禁用时 THEN 系统 SHALL 记录日志但不启动服务
4. WHEN HeartbeatService 启动成功时 THEN 系统 SHALL 记录日志显示心跳间隔

### 需求 6：在服务关闭时停止 HeartbeatService

**用户故事：** 作为一名系统开发者，我希望在 gateway 关闭时优雅地停止 HeartbeatService，以便系统能够正确清理资源。

#### 验收标准

1. WHEN 收到关闭信号时 THEN 系统 SHALL 在关闭其他服务前停止 HeartbeatService
2. WHEN 停止 HeartbeatService 时 THEN 系统 SHALL 调用 stop 方法
3. WHEN 停止完成时 THEN 系统 SHALL 记录日志

### 需求 7：显示 HeartbeatService 启动状态

**用户故事：** 作为一名终端用户，我希望在启动 gateway 时看到 HeartbeatService 的状态信息，以便确认服务是否正常配置。

#### 验收标准

1. WHEN gateway 启动时 THEN 系统 SHALL 显示 HeartbeatService 的配置信息
2. WHEN 显示信息时 THEN 系统 SHALL 包含心跳间隔（interval_s）
3. WHEN HeartbeatService 被禁用时 THEN 系统 SHALL 显示相应的状态信息

## 技术约束

1. 必须使用现有的 `nanobot_heartbeat` crate 中的 HeartbeatService
2. 必须与现有的 AgentLoop 和消息总线集成
3. 必须使用异步 Rust (tokio)
4. 必须保持与 Python 版本的行为一致
5. 必须正确处理错误和日志记录

## 成功标准

1. HeartbeatService 能够正确初始化并启动
2. 心跳检测能够正常工作，定期检查 HEARTBEAT.md
3. on_execute 回调能够正确执行任务
4. on_notify 回调能够正确发送通知
5. 服务启动和关闭时能够正确处理 HeartbeatService
6. 日志记录清晰，便于调试