# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `.github/workflows/changelog.yml` | 修改 | 添加 `workflow_dispatch` 触发器，更新 `resolve-tag` 步骤支持手动触发 |

## 任务列表

### 1. ✅ 添加 workflow_dispatch 触发器

- 优先级: P0
- 依赖项: 无
- 涉及文件: `.github/workflows/changelog.yml`
- 验收标准: `on:` 部分包含 `workflow_dispatch`，带必填的 `tag` 字符串输入参数（含中文 description），格式与 `release-bin.yml` 一致
- 风险/注意点: 无
- 步骤:
  - [x] 在 `on.push.tags` 之后添加 `workflow_dispatch` 块，包含 `inputs.tag`（`description: '要生成 changelog 的 tag 版本号'`，`required: true`，`type: string`）

### 2. ✅ 更新 resolve-tag 步骤支持双触发来源

- 优先级: P0
- 依赖项: 1
- 涉及文件: `.github/workflows/changelog.yml`
- 验收标准: `workflow_dispatch` 触发时从 `github.event.inputs.tag` 读取 tag，验证 tag 存在后 checkout 到该 tag；`push` 触发时保持原有的 `GITHUB_REF` 解析逻辑不变
- 风险/注意点: 确保 `push` 触发路径的行为与修改前完全一致
- 步骤:
  - [x] 将 `resolve-tag` 步骤中的 `TAG="${GITHUB_REF#refs/tags/}"` 改为条件分支：`github.event_name == 'workflow_dispatch'` 时使用 `github.event.inputs.tag`，否则使用 `GITHUB_REF#refs/tags/`
  - [x] `workflow_dispatch` 分支中增加 tag 存在性验证和 `git checkout "$TAG"`（参照 `release-bin.yml`）

### 3. ✅ 手动触发验证

- 优先级: P0
- 依赖项: 2
- 涉及文件: 无（在 GitHub 上操作）
- 验收标准: `gh workflow run changelog.yml -f tag=1.6.0` 成功触发并通过所有步骤，GitHub Release 的 1.6.0 页面出现 changelog 内容
- 风险/注意点: 需要先将修改推送到 main 分支；如果 1.6.0 Release 已由 `release-bin` 创建，`softprops/action-gh-release` 会更新已有 Release 而非重复创建
- 步骤:
  - [x] 提交修改并推送到 main 分支（PR #93 + PR #94）
  - [x] 执行 `gh workflow run changelog.yml --repo sammyne/nanobot-rs -f tag=1.6.0`
  - [x] 在 GitHub Actions 页面确认 workflow 运行成功（run 25983656373 全部通过）
  - [x] 检查 GitHub Release 1.6.0 页面确认 changelog 内容已生成
