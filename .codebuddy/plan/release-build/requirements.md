# 需求文档

## 引言

本功能旨在为 Rust 项目 `nanobot` 新增一条 GitHub Actions 流水线，在创建 Git Tag 时自动构建跨平台可执行文件，并将构建产物上传到 GitHub Release 页面。该流水线将与现有的 `changelog.yml` 流水线协同工作，实现完整的版本发布自动化流程。

## 需求

### 需求 1：流水线触发与版本验证

**用户故事：** 作为项目维护者，我希望在创建 Git Tag 时自动触发构建流程，并验证 Tag 版本与代码版本一致，以确保发布版本的准确性。

#### 验收标准

1. WHEN 创建新的 Git Tag THEN 系统 SHALL 自动触发构建流水线
2. WHEN 流水线启动 THEN 系统 SHALL 验证 Tag 版本号与 `crates/nanobot/Cargo.toml` 中的 `package.version` 一致
3. IF Tag 版本与 Cargo.toml 版本不一致 THEN 系统 SHALL 终止流水线并输出错误信息

### 需求 2：多平台构建

**用户故事：** 作为项目维护者，我希望流水线能够同时构建 Linux、Windows 和 macOS 三个平台的可执行文件，以便用户可以在不同操作系统上使用该工具。

#### 验收标准

1. WHEN 构建流程启动 THEN 系统 SHALL 在多个独立的 Job 中并行构建各平台可执行文件
2. WHEN 构建 Linux 可执行文件 THEN 系统 SHALL 使用 `ubuntu-24.04` 运行环境，构建 x86_64 架构
3. WHEN 构建 Windows 可执行文件 THEN 系统 SHALL 使用 `windows-2022` 运行环境，构建 x86_64 架构
4. WHEN 构建 macOS 可执行文件 THEN 系统 SHALL 使用 `macos-26` 运行环境，分别构建 x86_64 和 aarch64 两个架构版本
5. WHEN 各平台构建完成 THEN 系统 SHALL 生成对应平台的可执行文件，文件名包含架构和平台标识

### 需求 3：构建优化

**用户故事：** 作为项目维护者，我希望构建过程能够利用缓存和优化策略，以便减少构建时间和资源消耗。

#### 验收标准

1. WHEN 执行构建任务 THEN 系统 SHALL 使用 Rust 构建缓存（`Swatinem/rust-cache`）加速编译
2. WHEN 构建 Linux 可执行文件 THEN 系统 SHALL 使用 `cargo-chef` 进行依赖缓存优化
3. WHEN 构建可执行文件 THEN 系统 SHALL 使用 `--release` 模式进行优化编译

### 需求 4：制品上传

**用户故事：** 作为项目维护者，我希望构建产物能够直接上传到 GitHub Release 页面，以便用户可以直接下载使用。

#### 验收标准

1. WHEN 所有平台构建完成 THEN 系统 SHALL 将可执行文件直接上传到对应的 GitHub Release 页面
2. IF 对应的 GitHub Release 不存在 THEN 系统 SHALL 创建新的 Release
3. WHEN 上传制品 THEN 系统 SHALL 使用与 Tag 相同的 Release 名称

### 需求 5：构建产物命名规范

**用户故事：** 作为项目用户，我希望下载的文件有清晰的命名，以便我能快速识别适合自己系统的版本。

#### 验收标准

1. WHEN 命名 Linux 可执行文件 THEN 系统 SHALL 命名为 `nanobot-{version}-x86_64-linux`
2. WHEN 命名 Windows 可执行文件 THEN 系统 SHALL 命名为 `nanobot-{version}-x86_64-windows.exe`
3. WHEN 命名 macOS x86_64 可执行文件 THEN 系统 SHALL 命名为 `nanobot-{version}-x86_64-macos`
4. WHEN 命名 macOS aarch64 可执行文件 THEN 系统 SHALL 命名为 `nanobot-{version}-aarch64-macos`
5. WHEN 版本号提取 THEN 系统 SHALL 使用 Tag 名称作为版本号

### 需求 6：流水线权限与安全

**用户故事：** 作为项目维护者，我希望流水线具有适当的权限配置，以便安全地创建 Release 和上传制品。

#### 验收标准

1. WHEN 流水线运行 THEN 系统 SHALL 具有 `contents: write` 权限以创建 Release 和上传制品
2. WHEN 上传制品到 Release THEN 系统 SHALL 使用 `GITHUB_TOKEN` 进行身份验证

### 需求 7：构建状态反馈

**用户故事：** 作为项目维护者，我希望能够清晰地了解构建过程的执行状态，以便快速定位问题。

#### 验收标准

1. WHEN 构建任务执行 THEN 系统 SHALL 输出详细的构建日志
2. WHEN 构建成功 THEN 系统 SHALL 在 Job Summary 中显示构建产物信息
3. IF 构建失败 THEN 系统 SHALL 输出详细的错误信息以便排查
