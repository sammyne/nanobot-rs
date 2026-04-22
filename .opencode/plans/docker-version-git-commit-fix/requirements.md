# 需求

## 目标与背景

修复 Docker 镜像打包后进程打印的版本号缺少 git commit-id 的问题。当前 Docker 构建过程中 `.git/` 目录被 `.dockerignore` 排除，导致 `build.rs` 执行 `git rev-parse --short HEAD` 时失败，版本号变为 `1.5.1-unknown` 而非正确的 `1.5.1-<git-commit-id>`。

## 功能需求列表

### 核心功能
- 修改 Dockerfile，在 builder 阶段添加 `.git/` 目录的复制，使 build.rs 能够正确获取 git commit id
- 由于 Docker 多阶段构建的特性，builder 阶段的 .git 目录不会进入最终 runtime 镜像，不会有体积影响

### 扩展功能
- 无

## 非功能需求

- **兼容性**：修改不能破坏本地开发环境的构建流程
- **安全性**：不引入额外的安全风险
- **可维护性**：改动应简洁明了，易于理解

## 边界与不做事项

- 不修改 `.dockerignore` 的 `.git/` 排除规则
- 不修改项目源码结构

## 假设与约束

- **技术假设**：Docker 构建环境中 git 命令可用
- **资源约束**：无（builder 阶段的 .git 不会进入最终镜像）
- **环境约束**：Rust 版本 >= 1.93