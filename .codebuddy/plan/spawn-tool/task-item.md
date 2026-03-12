# 实施计划

- [ ] 1. 创建 SpawnTool<P> 泛型结构体
   - 定义 `SpawnTool<P: Provider>` 结构体
   - 添加 `manager: Arc<SubagentManager<P>>` 字段
   - 添加 `origin_channel: String` 和 `origin_chat_id: String` 字段
   - 实现 `new()` 构造函数，设置默认上下文值
   - 实现 `set_context()` 方法更新上下文信息
   - _需求：1.1、1.2、1.3_

- [ ] 2. 为 SpawnTool<P> 实现 Tool trait
   - 添加泛型约束：`P: Provider + Clone + Send + Sync + 'static`
   - 实现 `name()` 方法返回 "spawn"
   - 实现 `description()` 方法返回英文功能描述
   - 实现 `parameters()` 方法返回 JSON Schema（包含 task 和 label 参数）
   - 实现 `execute()` 异步方法调用 `self.manager.spawn()`
   - _需求：2.1、2.2、2.3、2.4_

- [ ] 3. 更新模块导出和依赖配置
   - 在 lib.rs 中导出 SpawnTool
   - 确认 nanobot-tools 依赖已正确配置
   - 验证项目编译通过
   - _需求：3.1、3.2、4.1、4.2_
