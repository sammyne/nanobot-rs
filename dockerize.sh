#!/bin/bash

# 获取脚本所在目录（项目根目录）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 获取短版 git commit id
GIT_COMMIT_ID=$(git rev-parse --short HEAD)
if [ -z "$GIT_COMMIT_ID" ]; then
    echo "错误：无法获取 git commit id"
    exit 1
fi

# 从 crates/cli/Cargo.toml 提取 package.version
CARGO_TOML_PATH="crates/cli/Cargo.toml"
if [ ! -f "$CARGO_TOML_PATH" ]; then
    echo "错误：找不到 $CARGO_TOML_PATH"
    exit 1
fi

PACKAGE_VERSION=$(grep -E '^version\s*=\s*"' "$CARGO_TOML_PATH" | head -1 | sed 's/version\s*=\s*"\([^"]*\)"/\1/')
if [ -z "$PACKAGE_VERSION" ]; then
    echo "错误：无法从 $CARGO_TOML_PATH 提取版本号"
    exit 1
fi

# 拼接 docker 镜像 tag
DOCKER_IMAGE_NAME="registry.cn-hangzhou.aliyuncs.com/sammyne/nanobot"
DOCKER_TAG="${PACKAGE_VERSION}-${GIT_COMMIT_ID}"
DOCKER_IMAGE="${DOCKER_IMAGE_NAME}:${DOCKER_TAG}"

echo "=========================================="
echo "Docker 镜像构建信息"
echo "=========================================="
echo "Package 版本: $PACKAGE_VERSION"
echo "Git Commit ID: $GIT_COMMIT_ID"
echo "Docker 镜像: $DOCKER_IMAGE"
echo "=========================================="

# 构建 Docker 镜像
docker build -t "$DOCKER_IMAGE" .

if [ $? -eq 0 ]; then
    echo "=========================================="
    echo "构建成功！"
    echo "镜像: $DOCKER_IMAGE"
    echo "=========================================="
else
    echo "构建失败！"
    exit 1
fi
