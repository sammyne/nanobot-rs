# 需求文档：HeartbeatService 重构

## 引言

本文档描述了对 `HeartbeatService` 的重构需求。当前实现使用 `Arc<Self>` 和原子布尔值来管理服务状态，并通过显式的 `stop()` 方法停止服务。重构的目标是简化状态管理，利用 Rust 的 async/await 机制和 tokio 的任务取消机制来实现更简洁、更符合惯用模式的实现。

重构将改变启动服务的 API，允许外部通过取消 future 来停止服务，从而移除对内部状态管理的依赖。

## 需求

### 需求 1

**用户故事：** 作为开发者，我希望 `HeartbeatService` 的启动方法能够消费自身（`self`）并直接包含心跳循环逻辑，以便外部可以通过取消 future 来停止服务，而不需要内部管理运行状态或额外的私有方法。

#### 验收标准

1. WHEN 调用 `start(self)` 方法 THEN 系统 SHALL 使用 `tokio::time::interval` 创建定时器并在循环中执行心跳检查
2. WHEN 外部 abort 启动任务的 future THEN 系统 SHALL 立即停止执行心跳循环
3. WHEN `start(self)` 方法返回 `HeartbeatError::Disabled` THEN 系统 SHALL 在服务被禁用时返回错误
4. IF 服务配置为启用状态 THEN 系统 SHALL 在 `start` 方法内部进入无限循环执行心跳检查直到被取消
5. WHEN 重构完成 THEN 系统 SHALL 移除 `run_loop` 私有方法，所有循环逻辑直接包含在 `start` 方法中

### 需求 2

**用户故事：** 作为开发者，我希望移除 `HeartbeatService` 中的运行状态相关字段，以便简化结构体定义并减少并发状态管理的复杂性。

#### 验收标准

1. WHEN 重构完成 THEN 系统 SHALL 从 `HeartbeatService` 结构体中移除 `running: Arc<AtomicBool>` 字段
2. WHEN 重构完成 THEN 系统 SHALL 从 `HeartbeatService` 结构体中移除 `timer_task: Arc<RwLock<Option<JoinHandle<()>>>>` 字段
3. WHEN 重构完成 THEN 系统 SHALL 保持其他字段（`filepath`、`provider`、`config`、`on_execute`、`on_notify`）不变

### 需求 3

**用户故事：** 作为开发者，我希望移除显式控制服务生命周期的方法，以便外部代码通过 tokio 的任务管理机制来控制服务的启动和停止。

#### 验收标准

1. WHEN 重构完成 THEN 系统 SHALL 移除 `stop()` 方法
2. WHEN 重构完成 THEN 系统 SHALL 移除 `is_running()` 方法
3. WHEN 重构完成 THEN 系统 SHALL 移除 `start(self: Arc<Self>)` 方法签名并替换为 `start(self)`

### 需求 4

**用户故事：** 作为开发者，我希望 `start` 方法内部直接实现心跳循环，使用 `tokio::time::interval` 替代手动睡眠和原子布尔值检查，以便代码更简洁且符合 Rust async 生态的惯用模式。

#### 验收标准

1. WHEN 重构完成 THEN 系统 SHALL 在 `start` 方法内部使用 `tokio::time::interval` 创建定时器
2. WHEN 重构完成 THEN 系统 SHALL 不再传递 `running: Arc<AtomicBool>` 参数
3. WHEN 重构完成 THEN 系统 SHALL 在 `start` 方法中使用 `ticker.tick().await` 和 `self.tick().await` 构建循环
4. WHEN 心跳执行出错 THEN 系统 SHALL 记录错误日志并继续下一次循环

### 需求 5

**用户故事：** 作为开发者，我希望移除不再使用的错误变体，以便错误类型更准确地反映实际的错误情况。

#### 验收标准

1. WHEN 重构完成 THEN 系统 SHALL 从 `HeartbeatError` 枚举中移除 `AlreadyRunning` 变体
2. WHEN 重构完成 THEN 系统 SHALL 从 `HeartbeatError` 枚举中移除 `NotRunning` 变体
3. WHEN 重构完成 THEN 系统 SHALL 保留其他错误变体（`Disabled`、`InvalidConfig`、`FileRead`、`Provider`、`Parse`、`Execute`、`Notify`）

### 需求 6

**用户故事：** 作为 gateway 命令的实现者，我希望使用 `CancellationToken` 来控制 `HeartbeatService` 的生命周期，以便在 shutdown 时通过释放 `DropGuard` 来优雅地停止服务。

#### 验收标准

1. WHEN 启动 HeartbeatService THEN 系统 SHALL 创建 `CancellationToken` 实例
2. WHEN 启动 HeartbeatService THEN 系统 SHALL 使用 `CancellationToken::run_until_cancelled_owned()` 包装 `heartbeat_service.start()` 创建可取消的 future
3. WHEN 创建可取消的 future THEN 系统 SHALL 将 `CancellationToken` 的 `DropGuard` 存储在 `ServicesContext` 中
4. WHEN 执行优雅关闭 THEN 系统 SHALL 释放 `DropGuard` 来触发 `CancellationToken` 的取消
5. WHEN `CancellationToken` 被取消 THEN 系统 SHALL 自动 abort 被包装的 `heartbeat_service.start()` future
6. WHEN 重构完成 THEN 系统 SHALL 移除 `heartbeat_service.stop()` 调用
7. WHEN 重构完成 THEN 系统 SHALL 不再将 `HeartbeatService` 包装在 `Arc` 中

### 需求 7

**用户故事：** 作为开发者，我希望更新所有测试用例以适应新的 API，以便测试代码能够正确验证重构后的功能。

#### 验收标准

1. WHEN 重构完成 THEN 系统 SHALL 更新生命周期管理测试以使用 `tokio::spawn` 和 `abort`
2. WHEN 重构完成 THEN 系统 SHALL 移除测试中对 `is_running()` 方法的调用
3. WHEN 重构完成 THEN 系统 SHALL 移除测试中对 `AlreadyRunning` 错误的验证
4. WHEN 重构完成 THEN 系统 SHALL 移除测试中对 `NotRunning` 错误的验证
5. WHEN 重构完成 THEN 系统 SHALL 确保其他测试（如配置验证、序列化、双阶段决策、异常场景）继续正常工作

### 需求 8

**用户故事：** 作为开发者，我希望确保 `tick()` 方法不受重构影响，以便心跳检查的核心逻辑保持不变。

#### 验收标准

1. WHEN 重构完成 THEN 系统 SHALL 保持 `tick()` 方法的签名和行为不变
2. WHEN 重构完成 THEN 系统 SHALL 保持 `decide()` 方法的实现不变
3. WHEN 重构完成 THEN 系统 SHALL 保持回调机制（`on_execute` 和 `on_notify`）的工作方式不变

## 技术约束

1. 必须使用 `tokio::time::interval` 而不是 `tokio::time::sleep`
2. 不需要 `HeartbeatService` 内部监听取消信号，外部直接 abort future 即可
3. 必须遵循 Rust async/await 的惯用模式
4. 必须保持 `HeartbeatService` 的泛型参数 `P: Provider` 不变
5. 必须保持回调机制（`on_execute` 和 `on_notify`）的工作方式不变
