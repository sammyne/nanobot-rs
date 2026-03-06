# 实施计划

- [x] 1. 实现版本管理工具模块
   - 创建 `version.rs` 模块，实现从 `Cargo.toml` 读取 crate 版本号的功能
   - 实现读取和写入 `VERSION` 文件的工具函数
   - 实现版本比较逻辑
   - _需求：1.2、1.3_

- [x] 2. 实现内置 Skills 目录复制功能
   - 创建 `builtin.rs` 模块，实现递归复制 `builtin/` 目录到 `workspace/builtin-skills` 的功能
   - **关键变更：** 使用 `include_dir` crate 在编译时嵌入 `builtin/` 目录，运行时从嵌入资源提取文件
   - 保持完整的目录结构（包括子目录如 `scripts/`）
   - 添加错误处理和日志记录
   - _需求：1.1、1.5、1.6_

- [x] 3. 实现版本检查与自动更新逻辑
   - 在初始化时检查 `workspace/builtin-skills/VERSION` 文件是否存在
   - 比较版本号，不匹配时删除旧目录并重新复制
   - 实现优雅降级（错误时记录日志但继续运行）
   - _需求：1.3、1.4、1.6、4.2_

- [x] 4. 修改 SkillsLoader 初始化逻辑
   - 在 `SkillsLoader::new()` 中集成版本管理和目录初始化逻辑
   - 确保 API 兼容性，不破坏现有接口
   - 添加可选的日志输出
   - _需求：1.1、1.3、4.1、4.3_

- [x] 5. 实现内置 Skills 加载功能
   - 修改 skills 注册逻辑，识别并注册 `workspace/builtin-skills` 目录下的所有 skills
   - 确保 `list_skills`、`load_skill`、`load_skills_for_context` 方法正确处理内置 skills
   - _需求：2.1、2.2、2.3、2.4_

- [x] 6. 实现 Skills 优先级管理
   - 确保 `workspace/skills` 中的 skill 优先于 `workspace/builtin-skills` 中的同名 skill
   - 在 skill 信息中正确报告来源路径
   - _需求：3.1、3.2、3.4_

- [x] 7. 编写单元测试
   - 测试版本读取和写入功能
   - 测试版本匹配检查逻辑
   - 测试目录复制功能
   - _需求：5.1、5.2_

- [x] 8. 编写集成测试
   - 测试首次初始化场景（目录不存在）
   - 测试版本不匹配时的自动更新场景
   - 测试 workspace skill 与 builtin skill 的优先级
   - _需求：5.3、5.4_

- [x] 9. 添加配置项到 Cargo.toml
   - 在 `Cargo.toml`（工作区）添加 `include_dir = "0.7"` 依赖
   - 在 `crates/skills/Cargo.toml` 引入 `include_dir.workspace = true`
   - 确保 `builtin/` 目录在构建时被嵌入到二进制文件
   - _需求：1.1_

- [x] 10. 编写文档和示例
   - 更新 `crates/skills/README.md`，说明内置 skills 的使用方式和版本管理机制
   - 添加 `include_dir` 编译时嵌入资源的技术实现说明
   - 添加代码注释，说明版本检查和自动更新的行为
   - _需求：4.1、4.3_
