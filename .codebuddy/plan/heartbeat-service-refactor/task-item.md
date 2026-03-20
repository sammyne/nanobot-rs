# 实施计划：HeartbeatService 重构

## 任务清单

- [ ] 1. 修改 HeartbeatService 结构体定义
  - 从 `HeartbeatService<P>` 中移除 `running: Arc<AtomicBool>` 字段
  - 从 `HeartbeatService<P>` 中移除 `timer_task: Arc<RwLock<Option<JoinHandle<()>>>>` 字段
  - 移除相关的导入语句（`AtomicBool`、`Ordering`、`JoinHandle`）
  - _需求：2.1、2.2、2.3_

- [ ] 2. 修改 HeartbeatError 枚举定义
  - 从 `HeartbeatError` 枚举中移除 `AlreadyRunning` 变体
  - 从 `HeartbeatError` 枚举中移除 `NotRunning` 变体
  - _需求：5.1、5.2、5.3_

- [ ] 3. 重写 start 方法并集成心跳循环逻辑
  - 将方法签名从 `pub async fn start(self: Arc<Self>)` 改为 `pub async fn start(self)`
  - 移除 `AlreadyRunning` 错误检查逻辑
  - 在方法内部使用 `tokio::time::interval` 创建定时器
  - 实现无限循环，使用 `ticker.tick().await` 等待间隔，调用 `self.tick().await` 执行心跳检查
  - 在循环中捕获错误并记录日志，继续下一次循环
  - _需求：1.1、1.2、1.3、1.4、1.5、3.3、4.1、4.2、4.3、4.4_

- [ ] 4. 删除不再需要的方法
  - 删除 `run_loop` 私有方法
  - 删除 `stop` 公共方法
  - 删除 `is_running` 公共方法
  - _需求：1.5、3.1、3.2、3.3_

- [ ] 5. 修改 ServicesContext 结构体定义
  - 在 `ServicesContext` 中添加 `heartbeat_token: Option<DropGuard>` 字段
  - 移除原有的 `heartbeat_service: Option<Arc<HeartbeatService<...>>>` 字段
  - 添加 `tokio_util::sync::CancellationToken` 的导入
  - _需求：6.2、6.3、6.6、6.7_

- [ ] 6. 修改 setup_heartbeat_service 函数
  - 创建 `HeartbeatService` 实例（不使用 `Arc::new` 包装）
  - 创建 `CancellationToken` 实例
  - 使用 `token.run_until_cancelled_owned()` 包装 `heartbeat_service.start()` 创建可取消的 future
  - 使用 `tokio::spawn` 启动任务（不需要存储 `JoinHandle`）
  - 将 token 的 `DropGuard` 存储在 `ServicesContext` 中
  - _需求：6.1、6.2、6.3、6.6、6.7_

- [ ] 7. 修改 shutdown 函数
  - 从 `ServicesContext` 中取出并释放 `heartbeat_token` 的 `DropGuard`（触发取消）
  - 移除对 `heartbeat_service.stop()` 的调用
  - _需求：6.4、6.5、6.6_

- [ ] 8. 更新生命周期管理测试
  - 使用 `tokio::spawn` 启动服务
  - 使用 `task.abort()` 停止服务
  - 移除对 `is_running()` 方法的调用
  - 移除对 `AlreadyRunning` 错误的验证
  - 移除对 `NotRunning` 错误的验证
  - 验证服务在启动和停止时的行为正确
  - _需求：7.1、7.2、7.3、7.4、7.5_

- [ ] 9. 验证核心功能未受影响
  - 确保 `tick()` 方法签名和行为不变
  - 确保 `decide()` 方法的实现不变
  - 确保回调机制（`on_execute` 和 `on_notify`）的工作方式不变
  - 运行配置验证、序列化、双阶段决策、异常场景等相关测试
  - _需求：8.1、8.2、8.3_

- [ ] 10. 添加 tokio-util 依赖
  - 在 `Cargo.toml` 中添加 `tokio-util` 依赖（版本 0.7 或更高）
  - 在使用 `CancellationToken` 的模块中导入 `tokio_util::sync::CancellationToken`
  - _需求：6.1、6.2_

## 实施顺序说明

1. 首先修改核心数据结构（结构体和枚举定义）
2. 然后重写核心方法（start 方法，集成 run_loop 逻辑）
3. 接着删除不再需要的方法
4. 修改调用方代码（gateway 命令）
5. 最后更新测试用例并验证功能

这个顺序确保每一步都可以独立验证，减少重构风险。
