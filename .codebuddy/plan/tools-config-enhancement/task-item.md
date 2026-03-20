# 实施计划

- [ ] 1. 定义 ExecConfig 配置结构
   - 在 `crates/config/src/schema/tools.rs` 中创建 `ExecConfig` 结构体
   - 添加 `timeout: u64` 字段（默认值 60）和 `path_append: String` 字段（默认值 ""）
   - 使用 `#[serde(rename_all = "camelCase", default)]` 属性确保序列化规范和向后兼容
   - 实现 `Default` trait
   - _需求：3.1、3.2、3.3_

- [ ] 2. 扩展 ToolsConfig 结构
   - 在 `ToolsConfig` 中添加 `restrict_to_workspace: bool` 字段（默认值 false）
   - 在 `ToolsConfig` 中添加 `exec: ExecConfig` 字段
   - 为新字段添加 `#[serde(default)]` 属性确保向后兼容
   - _需求：1.1、1.3、2.1_

- [ ] 3. 更新 ExecToolOptions 结构
   - 确认 `ExecToolOptions` 已包含 `restrict_to_workspace` 字段
   - 添加 `path_append: String` 字段（如不存在）
   - 确保 `timeout` 字段可被外部配置覆盖
   - _需求：2.2、2.3、2.5_

- [ ] 4. 修改 ToolRegistry 配置集成
   - 在 `ToolRegistry::new()` 或相关初始化方法中读取 `ToolsConfig`
   - 将 `ToolsConfig.exec` 配置应用到 `ExecToolOptions`
   - 将 `ToolsConfig.restrict_to_workspace` 传递给 `ExecToolOptions`
   - _需求：4.1、4.2_

- [ ] 5. 实现 PATH 环境变量追加逻辑
   - 在 `ExecTool` 执行命令前，检查 `path_append` 配置
   - 将配置的路径追加到命令执行环境的 PATH 变量中
   - 确保路径追加操作安全且跨平台兼容
   - _需求：2.3_

- [ ] 6. 添加配置单元测试
   - 为 `ExecConfig` 的序列化/反序列化编写测试
   - 为 `ToolsConfig` 新字段的默认值行为编写测试
   - 测试配置文件缺失字段时的向后兼容性
   - _需求：1.1、1.3、2.4、3.1、3.2_

- [ ] 7. 添加功能集成测试
   - 测试 `restrict_to_workspace: true` 时的工作空间限制行为
   - 测试 `exec.timeout` 配置对命令超时的影响
   - 测试 `exec.pathAppend` 配置对 PATH 环境变量的影响
   - _需求：1.2、2.2、2.3、4.3_

- [ ] 8. 更新配置文档和示例
   - 在项目文档中添加新配置字段的说明
   - 提供配置示例 JSON 文件
   - _需求：1.1、2.1、2.4_
