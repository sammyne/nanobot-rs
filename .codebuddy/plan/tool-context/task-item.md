# 实施计划

- [ ] 1. 定义 ToolContext 类型
   - 在 `crates/tools/src/core.rs` 中定义 `ToolContext` 结构体
   - 包含 `channel: String` 和 `chat_id: String` 字段
   - 实现 `new` 构造函数和只读 getter 方法
   - _需求：1.1、1.2、1.3、1.4_

- [ ] 2. 拓展 Tool trait execute 方法签名
   - 修改 `Tool` trait 的 `execute` 方法，添加 `ctx: &ToolContext` 参数
   - 新签名：`async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<Value>`
   - _需求：2.1、2.2_

- [ ] 3. 更新 ToolRegistry execute 方法
   - 修改 `ToolRegistry::execute` 方法签名，添加 `ctx: &ToolContext` 参数
   - 调用内部工具时传递 ctx 参数
   - _需求：3.1、3.2、3.3_

- [ ] 4. 更新 tools crate 中的 Tool 实现
   - 更新 `ShellTool::execute` 方法签名
   - 更新 `ReadFileTool::execute` 方法签名
   - 更新 `WriteFileTool::execute` 方法签名
   - 更新 `EditFileTool::execute` 方法签名
   - 更新 `ListDirTool::execute` 方法签名
   - _需求：4.1、4.2、4.3、4.4、4.5_

- [ ] 5. 更新 cron crate 中的 CronTool 实现
   - 更新 `CronTool::execute` 方法签名
   - _需求：4.6_

- [ ] 6. 更新 Agent 层调用
   - 在 Agent 的 ReAct 循环中从 `InboundMessage` 提取 `channel` 和 `chat_id`
   - 构建 `ToolContext` 实例
   - 调用 `ToolRegistry::execute` 时传递 ctx 参数
   - _需求：5.1、5.2、5.3_

- [ ] 7. 编译验证
   - 执行 `cargo build` 确保所有模块编译通过
   - 修复任何编译错误或警告
   - _需求：全部_
