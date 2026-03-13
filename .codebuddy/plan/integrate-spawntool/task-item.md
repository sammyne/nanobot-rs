# 实施计划

- [ ] 1. 添加 nanobot-subagent 依赖到 agent crate
   - 在 `crates/agent/Cargo.toml` 中添加 `nanobot-subagent.workspace = true`
   - _需求：5.1_

- [ ] 2. 修改 AgentLoop 结构体和构造函数
   - 在 AgentLoop 结构体中添加 `subagent_manager: Arc<SubagentManager<P>>` 字段
   - 修改 AgentLoop::new 和 new_direct 签名，添加必选参数 `subagent_manager: Arc<SubagentManager<P>>`
   - _需求：1.1、1.2、4.1、4.2_

- [ ] 3. 在 AgentLoop::new 内注册 SpawnTool
   - 使用注入的 SubagentManager 创建 SpawnTool 实例
   - 调用 `tool_registry.register()` 注册 SpawnTool
   - 确保工具定义在 `provider.bind_tools()` 调用时包含 spawn 工具
   - _需求：1.3、2.1、2.2、2.3、2.4、4.3_

- [ ] 4. 在 loop.rs 中添加必要的 use 声明
   - 添加 `use nanobot_subagent::{SpawnTool, SubagentManager};`
   - _需求：5.2_

- [ ] 5. 更新 CLI agent 命令调用
   - 在调用 AgentLoop::new 之前创建 SubagentManager
   - 传入正确的 SubagentManager 参数
   - _需求：6.1、6.3_

- [ ] 6. 更新 gateway 命令调用
   - 在调用 AgentLoop::new 之前创建 SubagentManager
   - 传入正确的 SubagentManager 参数
   - _需求：6.2、6.3_

- [ ] 7. 验证编译和运行
   - 运行 `cargo clippy --all-targets --all-features -- -D warnings -D clippy::uninlined_format_args` 确保无警告
   - 确保所有调用方正确创建并传入 SubagentManager
   - _需求：6.4_

## 关键设计决策

1. **SubagentManager 外部注入**：`AgentLoop::new` 接受必选的 `Arc<SubagentManager<P>>` 参数，由调用方负责创建。

2. **依赖注入模式**：采用依赖注入设计，AgentLoop 不负责创建 SubagentManager，只负责使用。

3. **SpawnTool 始终注册**：由于 SubagentManager 是必选参数，SpawnTool 将始终被创建和注册到工具注册表。

4. **配置责任转移**：SubagentManager 的配置（temperature、max_tokens、inbound_tx）由调用方负责设置。
