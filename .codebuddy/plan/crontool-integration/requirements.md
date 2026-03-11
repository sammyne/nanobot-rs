# 需求文档

## 引言

本功能旨在将 CronTool（定时任务工具）完整集成到 AgentLoop 和 CLI 系统中。CronTool 允许 AI 助手通过工具调用方式管理定时任务，包括创建、查询、启用/禁用和手动执行任务。

**关键技术特性：** 本项目的 Tool trait 设计支持运行时传入 ToolContext，该上下文在每次工具执行时通过 `execute(&ctx, params)` 方法动态传入，包含 `channel`（渠道名称）和 `chat_id`（聊天标识）。CronTool 在创建任务时会将当前上下文保存到任务的 payload 中，以便任务执行时能够正确投递消息到原始渠道。

## 需求

### 需求 1：AgentLoop 中的 CronTool 注册与管理

**用户故事：** 作为系统开发者，我希望 AgentLoop 能够自动注册和管理 CronTool，以便 AI 助手可以通过工具调用方式操作定时任务。

#### 验收标准

1. WHEN AgentLoop 初始化时 AND 提供了 cron_service 参数 THEN 系统 SHALL 自动注册 CronTool 到工具注册表
2. IF AgentLoop 初始化时未提供 cron_service THEN 系统 SHALL 跳过 CronTool 注册且不抛出异常
3. WHEN 工具执行时 THEN 系统 SHALL 通过 ToolContext 动态传入当前的 channel 和 chat_id 上下文
4. WHEN CronTool 被注册后 THEN 系统 SHALL 确保工具定义正确暴露给 LLM 提供商
5. IF CronTool 执行时发生异常 THEN 系统 SHALL 返回错误消息给 LLM 且不中断 AgentLoop

### 需求 2：CronService 的初始化与配置

**用户故事：** 作为系统运维人员，我希望 CronService 能够正确初始化并持久化任务数据，以便定时任务可以在系统重启后恢复。

#### 验收标准

1. WHEN 启动 gateway 模式时 THEN 系统 SHALL 创建 CronService 实例并指定持久化存储路径
2. WHEN CronService 初始化时 THEN 系统 SHALL 从存储文件加载已存在的任务
3. WHEN 创建 CronService 时 AND 存储文件不存在 THEN 系统 SHALL 创建空的存储文件
4. WHEN 设置 CronService 的 on_job 回调时 THEN 系统 SHALL 确保回调能够通过 AgentLoop 执行任务
5. WHEN gateway 启动时 THEN 系统 SHALL 调用 cron.start() 启动任务调度器
6. WHEN gateway 停止时 THEN 系统 SHALL 调用 cron.stop() 优雅关闭调度器

### 需求 3：CLI 命令支持

**用户故事：** 作为系统用户，我希望通过 CLI 命令管理定时任务，以便无需直接编辑配置文件或调用 API。

#### 验收标准

1. WHEN 执行 `nanobot cron list` 命令时 THEN 系统 SHALL 以表格形式显示所有任务的 ID、名称、调度规则、状态和下次执行时间
2. WHEN 执行 `nanobot cron list --all` 时 THEN 系统 SHALL 包含已禁用的任务
3. WHEN 执行 `nanobot cron add` 命令时 THEN 系统 SHALL 支持 --every（间隔）、--cron（表达式）、--at（指定时间）三种调度方式
4. WHEN 添加任务时指定 --tz 参数 AND 未指定 --cron THEN 系统 SHALL 返回错误提示
5. WHEN 执行 `nanobot cron remove <job_id>` 命令时 THEN 系统 SHALL 删除指定任务并显示成功消息
6. WHEN 执行 `nanobot cron enable <job_id>` 命令时 THEN 系统 SHALL 启用指定任务
7. WHEN 执行 `nanobot cron enable <job_id> --disable` 命令时 THEN 系统 SHALL 禁用指定任务
8. WHEN 执行 `nanobot cron run <job_id>` 命令时 THEN 系统 SHALL 立即执行指定任务并显示执行结果
9. WHEN 执行 `nanobot cron run <job_id> --force` 命令时 THEN 系统 SHALL 即使任务已禁用也执行

### 需求 4：Agent 模式中的 CronTool 支持

**用户故事：** 作为 CLI 用户，我希望在交互式 agent 模式下也能使用 CronTool，以便通过对话方式管理定时任务。

#### 验收标准

1. WHEN 启动 `nanobot agent` 命令时 THEN 系统 SHALL 创建 CronService 实例
2. WHEN AgentLoop 在 agent 模式下初始化时 THEN 系统 SHALL 传递 cron_service 参数
3. WHEN 用户在交互模式中请求创建定时任务 THEN AI 助手 SHALL 能够调用 CronTool 执行操作
4. IF 在 agent 模式下 CronService 未提供持久化路径 THEN 系统 SHALL 使用默认数据目录路径

### 需求 5：任务执行与消息投递

**用户故事：** 作为定时任务用户，我希望任务执行结果能够正确投递到指定渠道，以便及时收到通知。

#### 验收标准

1. WHEN 定时任务触发时 THEN 系统 SHALL 通过 AgentLoop.process_direct 方法执行任务消息
2. WHEN 任务配置了 --deliver 参数 AND 指定了 --to 和 --channel THEN 系统 SHALL 将执行结果投递到指定渠道
3. WHEN 任务执行完成且未配置投递 THEN 系统 SHALL 仅记录执行结果而不发送消息
4. WHEN 任务执行发生异常 THEN 系统 SHALL 记录错误日志且不中断调度器运行
5. WHEN 手动执行任务（cron run）时 THEN 系统 SHALL 在 CLI 中显示执行结果

### 需求 6：上下文传递与持久化

**用户故事：** 作为系统开发者，我希望工具执行上下文能够正确保存到任务中，以便任务执行时能够投递到原始渠道。

#### 验收标准

1. WHEN CronTool.handle_add 执行时 THEN 系统 SHALL 从 ToolContext 中获取 channel 和 chat_id
2. IF ToolContext 中 channel 或 chat_id 为空 THEN 系统 SHALL 返回 "no session context (channel/chat_id)" 错误
3. WHEN 创建任务成功时 THEN 系统 SHALL 将 channel 和 chat_id 保存到 CronPayload 中
4. WHEN 任务执行时 THEN 系统 SHALL 使用 payload 中的 channel 和 chat_id 构建执行上下文
5. WHEN 任务配置了自定义投递目标（to/channel）时 THEN 系统 SHALL 使用自定义配置覆盖 payload 中的上下文
6. WHEN 多个任务并发执行时 THEN 系统 SHALL 确保每个任务使用独立的上下文，互不干扰

### 需求 7：错误处理与边界情况

**用户故事：** 作为系统用户，我希望系统能够优雅地处理各种错误情况，以便不会因单个任务失败而影响整体服务。

#### 验收标准

1. WHEN 添加任务时参数不完整 THEN 系统 SHALL 返回明确的错误提示
2. WHEN 删除不存在的任务时 THEN 系统 SHALL 返回"任务未找到"提示
3. WHEN 任务执行超时 THEN 系统 SHALL 遵循 AgentLoop 的超时配置
4. IF CronService 存储文件损坏 THEN 系统 SHALL 记录错误并创建新的存储文件
5. WHEN CronTool 调用失败 THEN 系统 SHALL 返回友好的错误消息给 AI 助手以便用户理解

## 技术约束

1. 必须与现有的 Tool trait 和 ToolRegistry 架构保持一致
2. CronTool 必须实现 Tool trait，支持运行时传入 ToolContext
3. 所有 I/O 操作应采用异步方式（async/await）
4. 任务存储应使用 JSON 格式以便于调试和迁移
5. CLI 命令应遵循项目现有的命令结构和风格

## 成功标准

1. 所有单元测试和集成测试通过
2. 能够通过 AI 助手对话创建和管理定时任务
3. 能够通过 CLI 命令管理定时任务
4. 任务能够在系统重启后正确恢复
5. 任务执行结果能够正确投递到指定渠道
