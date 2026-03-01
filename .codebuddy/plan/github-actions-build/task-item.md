# 实施计划

- [ ] 1. 创建 GitHub Actions 工作流目录结构
   - 在项目根目录创建 `.github/workflows/` 目录
   - 创建 `build.yml` 工作流配置文件
   - _需求：4.1、4.2_

- [ ] 2. 配置工作流触发条件和路径过滤
   - 配置 `on: pull_request` 触发器，目标分支为 `main`
   - 配置 `on: workflow_dispatch` 触发器，支持手动触发
   - 添加 `paths-ignore` 配置排除文档文件（`*.md`、`docs/**`）
   - 添加 `paths` 配置包含 Rust 源代码和 Cargo 配置文件
   - _需求：1.1、1.2、1.3、1.4、5.1、5.2_

- [ ] 3. 配置 Rust 构建环境
   - 使用 `actions/checkout@v4` 检出代码
   - 使用 `dtolnay/rust-toolchain@stable` action 并指定 Rust 1.93.0 版本
   - 配置 cargo 缓存以加速构建
   - _需求：4.4、4.5_

- [ ] 4. 实现代码格式化检查步骤
   - 添加名为 "Format check" 的工作流步骤
   - 执行 `cargo fmt --check` 命令
   - 确保步骤失败时阻止后续步骤执行
   - _需求：2.1、2.2、2.3_

- [ ] 5. 实现单元测试执行步骤
   - 添加名为 "Run tests" 的工作流步骤
   - 配置步骤依赖关系，确保格式化检查通过后才执行
   - 执行 `cargo test` 命令运行单元测试
   - 配置测试失败时标记步骤为失败
   - _需求：3.1、3.2、3.3、3.4_

- [ ] 6. 验证工作流配置
   - 在本地使用 `act` 工具或 GitHub Actions 验证工作流语法
   - 创建测试 PR 验证触发条件
   - 确认路径过滤规则正确生效
   - 验证手动触发功能正常工作
   - _需求：全部_
