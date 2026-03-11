# 需求文档：Heartbeat Component

## 引言

Heartbeat 组件是 nanobot 系统中的周期性任务检查服务。它以固定间隔唤醒代理，检查是否有活跃任务需要处理。该组件采用两阶段设计：首先通过 LLM 分析 HEARTBEAT.md 文件决定是否需要执行任务，然后仅在确认有任务时才触发完整的代理执行流程。这种设计避免了不必要的代理唤醒和资源消耗。

本需求文档描述在 Rust 实现中需要满足的功能和非功能需求。

## 需求

### 需求 1：周期性心跳检查

**用户故事：** 作为一名系统用户，我希望心跳服务能按固定间隔自动检查任务，以便及时响应需要处理的任务。

#### 验收标准

1. WHEN 心跳服务启动且已启用 THEN 系统 SHALL 按配置的间隔时间（默认 30 分钟）定期触发心跳检查
2. WHEN 心跳间隔配置为 N 秒 THEN 系统 SHALL 每隔 N 秒执行一次心跳检查
3. WHEN 心跳服务停止 THEN 系统 SHALL 立即取消所有正在进行的定时任务
4. IF 心跳服务被禁用 THEN 系统 SHALL 不启动任何定时检查
5. WHEN 心跳循环中出现异常 THEN 系统 SHALL 记录错误日志并继续下一次检查

### 需求 2：两阶段决策机制

**用户故事：** 作为一名系统架构师，我希望心跳检查分为决策和执行两个阶段，以便在确认有任务时才触发资源密集型的代理执行。

#### 验收标准

1. WHEN 心跳检查触发 THEN 系统 SHALL 先执行 Phase 1（决策阶段）
2. WHEN Phase 1 执行时 THEN 系统 SHALL 读取工作区中的 HEARTBEAT.md 文件
3. IF HEARTBEAT.md 文件不存在或为空 THEN 系统 SHALL 跳过本次检查并记录调试日志
4. WHEN HEARTBEAT.md 文件存在 THEN 系统 SHALL 调用 LLM 分析文件内容
5. WHEN LLM 分析完成且返回 action="run" THEN 系统 SHALL 进入 Phase 2（执行阶段）
6. WHEN LLM 分析完成且返回 action="skip" THEN 系统 SHALL 跳过执行阶段并记录信息日志
7. WHEN Phase 2 执行时 THEN 系统 SHALL 调用配置的执行回调函数

### 需求 3：结构化 LLM 工具调用

**用户故事：** 作为一名开发者，我希望使用结构化的工具调用而非文本解析，以便获得可靠的决策结果。

#### 验收标准

1. WHEN 系统向 LLM 发送决策请求时 THEN 系统 SHALL 包含虚拟的 heartbeat 工具定义
2. WHEN 定义 heartbeat 工具时 THEN 系统 SHALL 包含 action 参数（枚举值：skip、run）
3. WHEN action 参数值为 "run" THEN 系统 SHALL 要求包含 tasks 参数
4. WHEN tasks 参数提供时 THEN 系统 SHALL 包含活跃任务的自然语言摘要
5. WHEN LLM 响应包含工具调用时 THEN 系统 SHALL 解析工具参数获取 action 和 tasks
6. WHEN LLM 响应不包含工具调用时 THEN 系统 SHALL 视为 action="skip"

### 需求 4：可配置的执行回调

**用户故事：** 作为一名系统集成者，我希望可以配置任务执行的回调函数，以便灵活地集成不同的代理执行逻辑。

#### 验收标准

1. WHEN 心跳服务初始化时 THEN 系统 SHALL 接受可选的 on_execute 回调函数
2. WHEN Phase 2 需要执行时 THEN 系统 SHALL 调用 on_execute 回调并传入任务摘要
3. WHEN on_execute 回调执行完成 THEN 系统 SHALL 获得执行结果字符串
4. IF on_execute 回调未配置 THEN 系统 SHALL 跳过 Phase 2 执行

### 需求 5：可配置的通知回调

**用户故事：** 作为一名系统监控者，我希望在任务执行完成后可以发送通知，以便及时了解任务执行结果。

#### 验收标准

1. WHEN 心跳服务初始化时 THEN 系统 SHALL 接受可选的 on_notify 回调函数
2. WHEN on_execute 执行完成且返回非空结果时 THEN 系统 SHALL 调用 on_notify 回调并传入结果
3. IF on_notify 回调已配置但 on_execute 返回空结果 THEN 系统 SHALL 不调用 on_notify
4. IF on_notify 回调未配置 THEN 系统 SHALL 不发送通知

### 需求 6：手动触发机制

**用户故事：** 作为一名操作员，我希望可以手动触发心跳检查，以便在需要时立即检查任务而不等待下一次定时触发。

#### 验收标准

1. WHEN 调用手动触发方法时 THEN 系统 SHALL 立即执行一次心跳检查
2. WHEN 手动触发时 THEN 系统 SHALL 执行完整的 Phase 1 和 Phase 2 流程
3. WHEN 手动触发成功且 action="run" THEN 系统 SHALL 返回执行结果字符串
4. WHEN 手动触发失败或 action="skip" THEN 系统 SHALL 返回 None

### 需求 7：错误处理和日志记录

**用户故事：** 作为一名系统维护者，我希望所有操作都有详细的日志记录，以便监控和调试。

#### 验收标准

1. WHEN 心跳服务启动时 THEN 系统 SHALL 使用 tracing 记录启动信息
2. WHEN 心跳服务停止时 THEN 系统 SHALL 取消运行中的异步任务
3. WHEN HEARTBEAT.md 文件读取失败时 THEN 系统 SHALL 捕获异常并记录错误日志
4. WHEN LLM 调用失败时 THEN 系统 SHALL 捕获异常并记录错误日志
5. WHEN 执行回调抛出异常时 THEN 系统 SHALL 捕获异常并记录错误日志
6. WHEN 任何异常发生时 THEN 系统 SHALL 不中断心跳循环并继续下一次检查

### 需求 8：状态管理和并发控制

**用户故事：** 作为一名系统架构师，我希望心跳服务能够正确处理并发启动和停止，确保资源安全释放。

#### 验收标准

1. WHEN 心跳服务已在运行时 THEN 系统 SHALL 拒绝重复启动并记录警告日志
2. WHEN 心跳服务停止时 THEN 系统 SHALL 清理所有异步任务句柄
3. WHEN 心跳服务停止后 THEN 系统 SHALL 可以重新启动
4. WHEN 同时调用启动和停止方法时 THEN 系统 SHALL 保证最终状态一致性

### 需求 9：配置灵活性

**用户故事：** 作为一名配置管理员，我希望可以自定义心跳间隔和启用状态，以便适应不同的使用场景。

#### 验收标准

1. WHEN 初始化心跳服务时 THEN 系统 SHALL 接受可配置的间隔时间参数
2. WHEN 间隔时间未提供时 THEN 系统 SHALL 使用默认值 30 分钟
3. WHEN 初始化心跳服务时 THEN 系统 SHALL 接受可配置的启用状态参数
4. WHEN 启用状态未提供时 THEN 系统 SHALL 默认为启用状态
5. WHEN 启用状态为 false 时 THEN 系统 SHALL 忽略启动请求

### 需求 10：依赖注入

**用户故事：** 作为一名库开发者，我希望心跳服务通过依赖注入方式接收必要的组件，以便提高可测试性和可扩展性。

#### 验收标准

1. WHEN 初始化心跳服务时 THEN 系统 SHALL 接受工作区路径作为参数
2. WHEN 初始化心跳服务时 THEN 系统 SHALL 接受 LLM provider 作为参数
3. WHEN 初始化心跳服务时 THEN 系统 SHALL 接受模型名称作为参数
4. WHEN 心跳服务运行时 THEN 系统 SHALL 使用注入的 provider 进行 LLM 调用
5. WHEN 读取 HEARTBEAT.md 文件时 THEN 系统 SHALL 使用注入的工作区路径

### 需求 11：网关层心跳配置

**用户故事：** 作为一名配置管理员，我希望在网关配置中可以配置心跳服务的参数，以便通过配置文件统一管理心跳服务。

#### 验收标准

1. WHEN 定义网关配置结构时 THEN 系统 SHALL 包含 heartbeat 字段
2. WHEN heartbeat 配置包含 enabled 字段 THEN 系统 SHALL 使用 serde 进行序列化和反序列化
3. WHEN heartbeat.enabled 未配置时 THEN 系统 SHALL 使用默认值 true
4. WHEN heartbeat 配置包含 interval_seconds 字段 THEN 系统 SHALL 表示心跳间隔（秒）
5. WHEN heartbeat.interval_seconds 未配置时 THEN 系统 SHALL 使用默认值 1800（30 分钟）
6. WHEN 加载配置文件时 THEN 系统 SHALL 正确解析 gateway.heartbeat 配置节
7. WHEN 保存配置文件时 THEN 系统 SHALL 正确序列化 gateway.heartbeat 配置节
8. WHEN 验证配置时 THEN 系统 SHALL 确保 heartbeat.interval_seconds 大于 0
9. WHEN heartbeat.interval_seconds 验证失败时 THEN 系统 SHALL 返回 ConfigError::Validation 错误

### 需求 12：工作空间模板中的 HEARTBEAT.md

**用户故事：** 作为一名系统管理员，我希望在创建工作空间时自动包含默认的 HEARTBEAT.md 文件，以便开箱即用地使用心跳功能。

#### 验收标准

1. WHEN 定义工作空间模板时 THEN 系统 SHALL 包含默认的 HEARTBEAT.md 文件内容
2. WHEN 默认 HEARTBEAT.md 文件 THEN 系统 SHALL 包含示例内容，说明如何触发心跳执行
3. WHEN 执行 onboard 命令时 THEN 系统 SHALL 将默认的 HEARTBEAT.md 文件写入到新创建的工作空间
4. WHEN 写入 HEARTBEAT.md 文件时 THEN 系统 SHALL 使用 UTF-8 编码
5. WHEN 工作空间目录已存在时 THEN 系统 SHALL 覆盖现有的 HEARTBEAT.md 文件
6. WHEN 写入文件失败时 THEN 系统 SHALL 返回 OnboardError::FileWrite 错误
7. IF 工作空间中已存在 HEARTBEAT.md 文件 THEN 系统 SHALL 保留该文件（不覆盖）
8. WHEN HEARTBEAT.md 文件写入成功时 THEN 系统 SHALL 记录信息日志

## 非功能需求

### API 命名规范

1. 核心心跳检查方法应命名为 `check`
2. 执行阶段方法应命名为 `execute`
3. 手动触发方法应命名为 `trigger`
4. 服务启动和停止方法应命名为 `start` 和 `stop`
5. 所有公共方法应使用蛇形命名法（snake_case）

### 性能需求

1. 心跳间隔的最小支持粒度应达到秒级
2. 心跳检查不应阻塞主线程或事件循环
3. LLM 调用应支持超时控制

### 可靠性需求

1. 任何单个操作的失败不应导致心跳服务停止
2. 服务应能够从临时错误中自动恢复
3. 资源泄漏应在服务停止时被正确清理

### 可测试性需求

1. 所有回调函数应可模拟（mock）
2. 文件系统操作应可抽象以便测试
3. LLM 调用应可拦截以便单元测试

### 可维护性需求

1. 代码应遵循项目现有的代码风格和命名约定
2. 应使用 tracing 进行日志记录，而非 println!
3. 错误处理应使用 thiserror 库定义具体错误类型
4. 测试代码应与源代码分离
5. 测试应采用表驱动测试和 Arrange-Act-Assert 模式
