# 实施计划

- [ ] 1. 创建 .dockerignore 文件
   - 排除 target 目录、Git 相关文件、IDE 配置文件等无关内容
   - 确保构建上下文仅包含必要的源码和配置文件
   - _需求：2.1、2.2_

- [ ] 2. 创建 Dockerfile 基础结构
   - 定义 ARG 参数 `RUST_VERSION`，默认值为 `1.93.0`
   - 设置构建阶段基础镜像为 `rust:${RUST_VERSION}-trixie`
   - 安装 cargo-chef 工具用于依赖缓存优化
   - 配置工作目录和环境变量
   - _需求：1.1、1.4_

- [ ] 3. 使用 cargo-chef 准备依赖层
   - 复制 Cargo.toml 和 Cargo.lock 文件
   - 运行 `cargo chef prepare --recipe-path recipe.json` 生成依赖配方
   - 运行 `cargo chef cook --release --recipe-path recipe.json` 编译并缓存依赖
   - _需求：1.2_

- [ ] 4. 编译 CLI 二进制文件
   - 复制项目源码到构建镜像
   - 执行 `cargo build --release` 编译 nanobot-cli
   - _需求：1.3_

- [ ] 5. 配置运行时镜像
   - 设置运行时基础镜像为 `trixie-20260223-slim`
   - 从构建阶段复制编译后的二进制文件到运行时镜像
   - _需求：3.1、3.2_

- [ ] 6. 配置 CLI 运行环境
   - 创建 `/opt/nanobot/bin` 目录
   - 将编译后的二进制文件重命名为 `nanobot` 并复制到 `/opt/nanobot/bin/nanobot`
   - 将 `/opt/nanobot/bin` 添加到 PATH 环境变量
   - 配置 ENTRYPOINT 或 CMD 以执行 nanobot 命令
   - _需求：4.1、4.2、4.3、4.4_

- [ ] 7. 添加构建参数和元数据标签
   - 使用 ARG 定义可配置的构建参数（如 Rust 版本、应用版本等）
   - 添加 LABEL 元数据（版本、描述、维护者等）
   - _需求：5.1、5.2_

- [ ] 8. 测试和验证
   - 构建 Docker 镜像并验证构建成功
   - 运行容器并测试 nanobot 命令及其子命令
   - 验证镜像大小和运行时行为符合预期
   - _需求：全部_
