# TODO

## 任务列表

### 1. 在 Dockerfile builder 阶段添加 .git 目录复制

- 优先级: P0
- 依赖项: 无
- 风险/注意点:
  - Docker 多阶段构建中，builder 阶段的 .git 不会进入最终 runtime 镜像
  - 只需在 builder 阶段添加，不需要修改 runtime 阶段

## 实现建议

- 在 Dockerfile 的 builder 阶段（第 49-58 行之间），添加一行 `COPY .git/ ./` 或 `COPY .git ./`
- 由于 .dockerignore 排除了 .git/，需要使用 `--include` 或在 builder 阶段之前临时包含
- 建议在 `COPY crates/ ./crates/` 之后添加 `COPY .git/ ./.git/` 以确保 build.rs 能正常执行 git 命令

## 验证方法

- 构建 Docker 镜像后，运行 `docker run --rm <image> gateway --version` 或类似命令检查版本号
- 版本号应显示为 `1.5.1-<git-commit-id>` 格式（如 `1.5.1-a1b2c3d`）