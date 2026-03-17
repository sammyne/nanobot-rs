# =============================================================================
# 构建参数定义
# =============================================================================
ARG RUST_VERSION=1.93.0
ARG CARGO_CHEF_VERSION=0.1.77

# =============================================================================
# 阶段 1: Chef - 安装 cargo-chef（供后续阶段复用）
# =============================================================================
FROM rust:${RUST_VERSION}-trixie AS chef

ARG CARGO_CHEF_VERSION

# 安装 cargo-chef（固定版本）
RUN cargo install cargo-chef --version ${CARGO_CHEF_VERSION}

# =============================================================================
# 阶段 2: Planner - 使用 cargo-chef 生成依赖配方
# =============================================================================
FROM chef AS planner

WORKDIR /app

# 复制 Cargo 配置文件
COPY Cargo.toml Cargo.lock ./

# 复制 crates 目录（cargo metadata 需要检查 target 类型）
COPY crates/ ./crates/

# 生成依赖配方
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# 阶段 3: Cacher - 编译并缓存依赖
# =============================================================================
FROM chef AS cacher

WORKDIR /app

# 从 planner 阶段复制配方文件
COPY --from=planner /app/recipe.json recipe.json

# 编译依赖（此层将被 Docker 缓存）
RUN cargo chef cook --release --recipe-path recipe.json

# =============================================================================
# 阶段 4: Builder - 编译项目
# =============================================================================
FROM cacher AS builder

WORKDIR /app

# 复制项目源码
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# 编译 CLI 二进制文件
RUN cargo build --release --bin nanobot

# 设置 HOME 环境变量并运行 onboard 命令生成默认配置文件
ENV HOME=/root
RUN ./target/release/nanobot onboard

# =============================================================================
# 阶段 5: Runtime - 最终运行时镜像
# =============================================================================
FROM debian:trixie-20260223-slim AS runtime

# 构建参数（用于标签）
ARG MAINTAINER="sammyne"

# 元数据标签
LABEL org.opencontainers.image.title="nanobot" \
      org.opencontainers.image.description="Nanobot - AI agent command line interface" \
      org.opencontainers.image.authors="${MAINTAINER}" \
      org.opencontainers.image.source="https://github.com/sammyne/nanobot-rs"

WORKDIR /opt/nanobot

# 安装运行时依赖（如需要）
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl jq && \
    rm -rf /var/lib/apt/lists/*

# 创建安装目录
RUN mkdir -p /opt/nanobot/bin

# 从 builder 阶段复制编译后的二进制文件
COPY --from=builder /app/target/release/nanobot /opt/nanobot/bin/nanobot

# 从 builder 阶段复制默认配置文件到用户目录
COPY --from=builder /root/.nanobot /root/.nanobot

# 配置环境变量
ENV PATH="/opt/nanobot/bin:${PATH}"

# 设置入口点
ENTRYPOINT ["nanobot"]
CMD ["gateway"]
