# 需求文档：Rust 版 Cron 工具

## 引言

本功能旨在为 Rust 版本的 nanobot 项目实现一个与 Python 版本功能对等的 cron 任务调度工具。该工具允许 AI Agent 调度提醒和周期性任务，支持一次性定时任务、周期性间隔任务以及标准 cron 表达式调度。

## 需求

### 需求 1：核心类型定义

**用户故事：** 作为开发者，我希望定义清晰的 cron 任务相关类型，以便系统能够正确表示和处理任务调度信息。

#### 验收标准

1. WHEN 定义 CronSchedule 类型 THEN 系统 SHALL 包含三种调度类型：at（一次性）、every（周期性）、cron（cron表达式）
2. WHEN 定义 CronSchedule 类型 THEN 系统 SHALL 支持 at_ms（毫秒时间戳）、every_ms（毫秒间隔）、expr（cron表达式）、tz（时区）字段
3. WHEN 定义 CronPayload 类型 THEN 系统 SHALL 包含 kind、message、deliver、channel、to 字段
4. WHEN 定义 CronJobState 类型 THEN 系统 SHALL 包含 next_run_at_ms、last_run_at_ms、last_status、last_error 字段
5. WHEN 定义 CronJob 类型 THEN 系统 SHALL 包含 id、name、enabled、schedule、payload、state、created_at_ms、updated_at_ms、delete_after_run 字段
6. WHEN 定义 CronStore 类型 THEN 系统 SHALL 包含 version 和 jobs 列表
7. WHEN 所有类型定义完成 THEN 系统 SHALL 实现 Serialize/Deserialize trait 以支持 JSON 序列化

### 需求 2：CronService 服务实现

**用户故事：** 作为系统，我希望有一个完整的 cron 服务来管理和执行定时任务，以便任务能够按时触发并正确执行。

#### 验收标准

1. WHEN 创建 CronService 实例 THEN 系统 SHALL 接受 store_path（存储路径）和可选的 on_job 回调函数
2. WHEN 调用 start() 方法 THEN 系统 SHALL 加载持久化数据、重新计算下次运行时间、启动定时器
3. WHEN 调用 stop() 方法 THEN 系统 SHALL 停止运行并取消定时器
4. WHEN 任务到期 THEN 系统 SHALL 执行 on_job 回调（如果存在）并更新任务状态
5. WHEN 执行任务成功 THEN 系统 SHALL 将 last_status 设置为 "ok"
6. WHEN 执行任务失败 THEN 系统 SHALL 将 last_status 设置为 "error" 并记录错误信息
7. WHEN 一次性任务（at 类型）执行完成 THEN 系统 SHALL 根据配置禁用任务或删除任务
8. WHEN 周期性或 cron 任务执行完成 THEN 系统 SHALL 计算并设置下次运行时间

### 需求 3：下次运行时间计算

**用户故事：** 作为系统，我希望能够准确计算任务的下次运行时间，以便在正确的时间触发任务。

#### 验收标准

1. WHEN 计算一次性任务（at 类型）的下次运行时间 THEN 系统 SHALL 返回 at_ms（如果大于当前时间）或 None
2. WHEN 计算周期性任务（every 类型）的下次运行时间 THEN 系统 SHALL 返回当前时间加上 every_ms
3. WHEN 计算 cron 表达式任务的下次运行时间 THEN 系统 SHALL 解析 cron 表达式并返回下次执行时间
4. IF cron 表达式包含时区设置 THEN 系统 SHALL 使用指定时区计算时间
5. IF cron 表达式解析失败 THEN 系统 SHALL 返回 None

### 需求 4：持久化存储

**用户故事：** 作为用户，我希望定时任务能够持久化保存，以便系统重启后任务不会丢失。

#### 验收标准

1. WHEN 添加或修改任务 THEN 系统 SHALL 将数据保存到 JSON 文件
2. WHEN 服务启动 THEN 系统 SHALL 从 JSON 文件加载已保存的任务
3. IF 存储文件不存在 THEN 系统 SHALL 创建空的 CronStore
4. IF 存储文件损坏 THEN 系统 SHALL 记录警告并创建空的 CronStore

### 需求 5：CronTool 工具实现

**用户故事：** 作为 AI Agent，我希望通过工具接口来管理定时任务，以便在对话过程中能够调度和查询任务。

#### 验收标准

1. WHEN 实现 Tool trait THEN 系统 SHALL 提供工具名称 "cron"
2. WHEN 实现 Tool trait THEN 系统 SHALL 提供描述 "Schedule reminders and recurring tasks. Actions: add, list, remove."
3. WHEN 定义工具参数 THEN 系统 SHALL 包含 action（必需）、message、every_seconds、cron_expr、tz、at、job_id 参数
4. WHEN 执行 add 操作 THEN 系统 SHALL 创建新任务并返回任务 ID 和名称
5. WHEN 执行 list 操作 THEN 系统 SHALL 返回所有已启用任务的列表
6. WHEN 执行 remove 操作 THEN 系统 SHALL 删除指定 ID 的任务并返回结果
7. IF add 操作未提供 message THEN 系统 SHALL 返回错误信息
8. IF add 操作同时使用 tz 和非 cron_expr 参数 THEN 系统 SHALL 返回错误信息
9. IF add 操作使用无效时区 THEN 系统 SHALL 返回错误信息

### 需求 6：会话上下文支持

**用户故事：** 作为系统，我希望 cron 工具能够关联当前会话上下文，以便任务触发时能够正确投递消息。

#### 验收标准

1. WHEN 调用 set_context 方法 THEN 系统 SHALL 设置 channel 和 chat_id
2. WHEN 添加任务且设置了上下文 THEN 系统 SHALL 将 channel 和 to（chat_id）保存到任务 payload
3. IF 添加任务时未设置上下文 THEN 系统 SHALL 返回错误信息

### 需求 7：依赖管理

**用户故事：** 作为开发者，我希望项目正确管理所需依赖，以便 cron 功能能够正常编译和运行。

#### 验收标准

1. WHEN 添加依赖 THEN 系统 SHALL 包含 cron 表达式解析库（如 cron 或 similar）
2. WHEN 添加依赖 THEN 系统 SHALL 包含时区处理库（如 chrono-tz）
3. WHEN 添加依赖 THEN 系统 SHALL 包含 UUID 生成库（如 uuid）
4. WHEN 添加依赖 THEN 系统 SHALL 复用 workspace 中已有的依赖（如 tokio、serde、chrono 等）

### 需求 8：模块导出和注册

**用户故事：** 作为开发者，我希望 cron 工具能够正确集成到工具注册表中，以便 Agent 能够发现和使用该工具。

#### 验收标准

1. WHEN 在 lib.rs 中添加模块声明 THEN 系统 SHALL 导出 cron 和 cron_types 模块
2. WHEN 在 registry 中注册工具 THEN 系统 SHALL 调用 ToolRegistry 的 register 函数来注册 CronTool
3. WHEN 注册 CronTool THEN 系统 SHALL 需要传入 CronService 实例或相关依赖
