# 实施计划

- [ ] 1. 创建 GitHub Actions 工作流文件
   - 创建 `.github/workflows/release.yml` 文件
   - 配置工作流触发条件为 Tag 创建（`tags: ['v*']`）
   - 配置工作流权限 `contents: write`
   - _需求：1.1、6.1、6.2_

- [ ] 2. 实现版本验证 Job
   - 创建 `verify-version` Job，提取 Tag 版本号
   - 读取 `crates/nanobot/Cargo.toml` 中的 `package.version`
   - 比较两个版本号，不一致时终止流水线并输出错误
   - _需求：1.2、1.3_

- [ ] 3. 配置多平台构建矩阵
   - 定义构建矩阵：Linux x86_64、Windows x86_64、macOS x86_64、macOS aarch64
   - 为每个平台配置对应的运行环境（`ubuntu-24.04`、`windows-2022`、`macos-26`）
   - 配置各平台的 Rust 编译目标（target）
   - _需求：2.1、2.2、2.3、2.4_

- [ ] 4. 配置构建优化策略
   - 集成 `Swatinem/rust-cache` 用于 Rust 构建缓存
   - 为 Linux 平台配置 `cargo-chef` 依赖缓存
   - 使用 `--release` 模式进行优化编译
   - _需求：3.1、3.2、3.3_

- [ ] 5. 实现构建产物命名逻辑
   - 提取 Tag 版本号作为版本标识
   - 根据平台和架构生成标准化的文件名
   - 重命名构建产物为规范格式（如 `nanobot-{version}-x86_64-linux`）
   - _需求：5.1、5.2、5.3、5.4、5.5_

- [ ] 6. 实现 Release 创建和制品上传
   - 创建 `upload-release` Job，依赖所有构建 Job
   - 使用 `softprops/action-gh-release` 创建或更新 GitHub Release
   - 上传所有平台的可执行文件到 Release 页面
   - _需求：4.1、4.2、4.3_

- [ ] 7. 添加构建状态反馈
   - 在各 Job 中配置详细的日志输出
   - 在构建成功时输出产物信息到 Job Summary
   - 在构建失败时输出详细的错误信息
   - _需求：7.1、7.2、7.3_
