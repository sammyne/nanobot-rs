# 实施计划

- [ ] 1. 创建 templates crate 并添加依赖
   - 在项目中创建新的 templates crate
   - 在 Cargo.toml 中添加 `include_dir` crate 依赖
   - 配置 workspace 依赖管理
   - _需求：1.1_

- [ ] 2. 在 templates crate 中嵌入模板资源
   - 创建模板目录结构，从 Python 版复制模板文件内容
   - 使用 `include_dir!` 宏在编译时嵌入模板资源
   - 提供获取模板内容的公共接口
   - _需求：1.2、1.3、6.1、6.2_

- [ ] 3. 实现工作空间目录创建逻辑
   - 从配置中获取工作空间路径（`agents.defaults.workspace`）
   - 实现目录创建功能，包含错误处理和用户反馈
   - _需求：2.1、2.2、2.3、7.2_

- [ ] 4. 实现根级别模板文件创建
   - 实现 USER.md、AGENTS.md、SOUL.md、TOOLS.md 四个模板文件的创建逻辑
   - 添加文件存在性检查，跳过已存在的文件
   - 输出创建状态信息到控制台
   - _需求：3.1、3.2、3.3、7.1_

- [ ] 5. 实现 Memory 子目录与文件创建
   - 创建 memory 子目录
   - 从模板创建 MEMORY.md 文件
   - 创建空的 HISTORY.md 文件
   - _需求：4.1、4.2、4.3、4.4_

- [ ] 6. 实现 Skills 子目录创建
   - 创建 skills 子目录
   - 处理目录已存在的情况
   - _需求：5.1、5.2_

- [ ] 7. 完善 onboard 命令集成
   - 将上述功能集成到 onboard 命令执行流程中
   - 在 CLI crate 中添加对 templates crate 的依赖
   - 确保错误不中断整体流程，尽可能完成其他操作
   - _需求：7.3_

- [ ] 8. 编写单元测试
   - 测试 templates crate 的模板资源嵌入和读取
   - 测试目录和文件创建逻辑
   - 测试错误处理场景
   - _需求：全量验收标准_
