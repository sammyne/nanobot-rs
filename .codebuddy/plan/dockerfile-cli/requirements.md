# 需求文档

## 引言

为 nanobot-rs 项目的 CLI 进程编写 Dockerfile，实现容器化部署。采用多阶段构建（Multi-stage build）方式，确保镜像精简、安全，并优化构建缓存层，提升构建效率。

## 需求

### 需求 1：多阶段构建 Dockerfile

**用户故事：** 作为一名开发者，我希望使用多阶段构建的 Dockerfile，以便生成精简的生产镜像并优化构建缓存。

#### 验收标准

1. WHEN 构建 Docker 镜像 THEN 系统 SHALL 使用多阶段构建方式，分离构建环境与运行环境
2. WHEN 执行依赖安装步骤 THEN 系统 SHALL 使用 cargo-chef 工具生成 recipe.json 并缓存依赖层，以优化 Docker 缓存效率
3. WHEN 构建完成 THEN 系统 SHALL 仅保留编译后的二进制文件，不包含构建工具和中间产物
4. IF 使用 Rust 官方镜像 THEN 系统 SHALL 使用 `rust:${RUST_VERSION}-trixie` 作为构建阶段基础镜像，其中 `RUST_VERSION` 可通过 ARG 指定，默认值为 `1.93.0`

### 需求 2：配置 .dockerignore 文件

**用户故事：** 作为一名开发者，我希望配置 .dockerignore 文件，以便排除不必要的文件，减少构建上下文大小并提升构建速度。

#### 验收标准

1. WHEN 构建 Docker 镜像 THEN 系统 SHALL 自动排除 target 目录、Git 相关文件、IDE 配置文件等无关内容
2. WHEN .dockerignore 文件存在 THEN 系统 SHALL 确保构建上下文仅包含必要的源码和配置文件

### 需求 3：运行时镜像优化

**用户故事：** 作为一名运维人员，我希望运行时镜像尽可能精简，以便减少镜像体积和攻击面。

#### 验收标准

1. WHEN 选择运行时基础镜像 THEN 系统 SHALL 使用 `trixie-20260223-slim` 作为运行时镜像
2. WHEN 最终镜像生成 THEN 系统 SHALL 仅包含运行 CLI 所需的二进制文件和必要的系统库

### 需求 4：CLI 运行配置

**用户故事：** 作为一名用户，我希望容器能够正确运行 nanobot CLI 命令，以便在容器化环境中使用所有功能。

#### 验收标准

1. WHEN 容器启动 THEN 系统 SHALL 能够执行 `nanobot` 命令及其子命令（onboard、agent、gateway、cron）
2. WHEN 运行 CLI THEN 系统 SHALL 正确处理环境变量和配置文件挂载
3. WHEN 安装可执行文件 THEN 系统 SHALL 将二进制文件重命名为 `nanobot` 并放置在 `/opt/nanobot/bin/nanobot` 目录
4. WHEN 配置环境变量 THEN 系统 SHALL 将 `/opt/nanobot/bin` 目录添加到 PATH 环境变量

### 需求 5：构建参数和标签

**用户故事：** 作为一名开发者，我希望 Dockerfile 支持构建参数和元数据标签，以便灵活配置和追踪镜像版本。

#### 验收标准

1. WHEN 构建 Docker 镜像 THEN 系统 SHALL 支持通过 ARG 指定 Rust 版本或其他构建参数
2. WHEN 镜像构建完成 THEN 系统 SHALL 包含适当的 LABEL 元数据（如版本、描述、维护者等）
