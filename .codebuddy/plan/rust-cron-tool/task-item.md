# 实施计划

- [ ] 1. 添加项目依赖
   - 在 crates/tools/Cargo.toml 中添加 cron、chrono-tz、uuid 依赖
   - 复用 workspace 中已有的 tokio、serde、chrono 等依赖
   - _需求：7.1、7.2、7.3、7.4_

- [ ] 2. 创建核心类型定义模块
   - 创建 crates/tools/src/cron_types.rs 文件
   - 定义 CronSchedule、CronPayload、CronJobState、CronJob、CronStore 结构体
   - 为所有类型实现 Serialize/Deserialize trait
   - _需求：1.1、1.2、1.3、1.4、1.5、1.6、1.7_

- [ ] 3. 实现持久化存储功能
   - 创建 crates/tools/src/cron/storage.rs 模块
   - 实现 CronStore 的 save 和 load 方法
   - 处理文件不存在和文件损坏的情况
   - _需求：4.1、4.2、4.3、4.4_

- [ ] 4. 实现下次运行时间计算
   - 创建 crates/tools/src/cron/scheduler.rs 模块
   - 实现一次性任务（at 类型）的时间计算
   - 实现周期性任务（every 类型）的时间计算
   - 实现 cron 表达式解析和下次时间计算
   - 支持时区处理
   - _需求：3.1、3.2、3.3、3.4、3.5_

- [ ] 5. 实现 CronService 服务
   - 创建 crates/tools/src/cron/service.rs 模块
   - 实现 CronService 结构体，包含 store_path 和 on_job 回调
   - 实现 start() 和 stop() 方法
   - 实现任务执行逻辑和状态更新
   - 实现定时器管理和任务调度
   - _需求：2.1、2.2、2.3、2.4、2.5、2.6、2.7、2.8_

- [ ] 6. 实现 CronTool 工具接口
   - 创建 crates/tools/src/cron/mod.rs 模块
   - 实现 CronTool 结构体，包含 CronService 引用和上下文信息
   - 实现 Tool trait，包括 name() 和 description() 方法
   - 定义工具参数结构（CronToolArgs）
   - _需求：5.1、5.2、5.3_

- [ ] 7. 实现 CronTool 的执行逻辑
   - 实现 add 操作：创建任务并返回 ID 和名称
   - 实现 list 操作：返回所有已启用任务
   - 实现 remove 操作：删除指定任务
   - 实现参数验证和错误处理
   - _需求：5.4、5.5、5.6、5.7、5.8、5.9_

- [ ] 8. 实现会话上下文支持
   - 为 CronTool 实现 set_context 方法
   - 在添加任务时自动填充 channel 和 to 字段
   - 实现上下文缺失的错误处理
   - _需求：6.1、6.2、6.3_

- [ ] 9. 模块导出和集成
   - 在 crates/tools/src/lib.rs 中添加模块声明
   - 导出 cron 和 cron_types 模块
   - _需求：8.1_

- [ ] 10. 注册 CronTool 到工具注册表
   - 在应用初始化时调用 ToolRegistry 的 register 函数注册 CronTool
   - 确保传入必要的 CronService 实例或相关依赖
   - _需求：8.2、8.3_
