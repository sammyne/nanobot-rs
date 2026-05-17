# 需求

## 目标与背景

项目有 3 个 tag 触发的 CI 流水线：`dockerize.yml`、`release-bin.yml`、`changelog.yml`。其中 `dockerize` 和 `release-bin` 都支持 `workflow_dispatch` 手动触发，唯独 `changelog` 不支持。

当 tag 推送出现问题（如打在错误的 commit 上、删除重推后 GitHub 不再触发）时，`dockerize` 和 `release-bin` 可以通过手动触发补救，而 `changelog` 无法补救，导致 GitHub Release 缺失。

## 方案比较

### 方案 1: 为 changelog.yml 添加 workflow_dispatch 触发器

- 思路: 参照 `dockerize.yml` 和 `release-bin.yml` 的模式，为 `changelog.yml` 添加 `workflow_dispatch` 触发器，接受 `tag` 输入参数，并更新 `resolve-tag` 步骤以区分 push 和手动触发两种来源。
- 优点: 改动最小，与现有两个 workflow 保持一致，解决问题直接。
- 缺点: 无明显缺点。

### 方案 2: 将 changelog 合并到 release-bin workflow 中

- 思路: 把 changelog 生成逻辑作为 `release-bin.yml` 的一个 job，减少独立 workflow 数量。
- 优点: 减少重复的版本校验逻辑，workflow 数量更少。
- 缺点: 改动范围大，两个功能耦合在一起，单独重跑 changelog 或 release-bin 不方便。

### 推荐

方案 1。改动最小，风险最低，与现有模式一致。

## 功能需求列表

### 核心功能

- 为 `changelog.yml` 添加 `workflow_dispatch` 触发器，接受必填的 `tag` 字符串输入参数
- 更新 `resolve-tag` 步骤：`workflow_dispatch` 时从 `github.event.inputs.tag` 读取 tag，`push` 时从 `GITHUB_REF` 解析 tag

## 非功能需求

- **一致性**：触发器定义和 tag 解析逻辑与 `dockerize.yml`、`release-bin.yml` 保持风格一致
- **兼容性**：不影响现有 tag push 触发的行为

## 边界与不做事项

- 不修改 `dockerize.yml` 和 `release-bin.yml`
- 不修改版本校验、changelog 生成、Release 创建等后续步骤的逻辑
- 不调整 `build.yml`

## 假设与约束

- **技术假设**：`actions/checkout@v6` 在 `workflow_dispatch` 触发时默认检出默认分支（main），配合 `fetch-depth: 0` 可获取完整历史和所有 tag，`git-cliff --current` 能正确基于指定 tag 生成 changelog

## 待确认事项

无
