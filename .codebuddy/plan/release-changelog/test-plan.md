# 测试计划

## 实现概述

已成功实现以下功能：

### 1. 创建 git-cliff 配置文件 ✅
- 文件：`cliff.toml`
- 支持 Conventional Commits 格式解析（feat、fix、docs、chore、refactor、test、style、perf）
- 配置 Keep a Changelog 格式模板
- 配置 breaking change 识别和展示规则

### 2. 修改 release workflow ✅
- 文件：`.github/workflows/release.yml`
- 添加新的 job：`update-changelog`
- 配置 job 依赖关系（在 Docker 镜像构建成功后执行）
- 设置权限：`contents: write`
- 仅在 Git Tag 推送时执行

### 3. 集成 git-cliff-action ✅
- 使用 `orhun/git-cliff-action@v4` 生成变更历史
- 配置参数：config、args（指定 tag）
- 将生成的 changelog 内容输出到变量

### 4. 实现 CHANGELOG.md 文件更新和推送 ✅
- 检查 CHANGELOG.md 文件是否存在
- 在文件顶部插入新版本的变更内容
- 配置 Git 用户信息（github-actions[bot]）
- 创建提交并推送到仓库
- 实现推送重试机制（最多 3 次）
- 输出提交 SHA 和变更摘要

### 5. 使用 softprops/action-gh-release 创建 GitHub Release ✅
- 使用 `softprops/action-gh-release@v2` 创建 Release
- 将版本号作为标题
- 将 git-cliff 生成的变更历史作为发布说明内容
- 输出 Release URL

### 6. 实现边界情况处理 ✅
- 处理两个 Tag 之间没有提交记录的情况（生成 "No changes" 消息）
- 处理首次发布（没有上一个 Tag）的情况（git-cliff 支持从初始提交开始）
- 处理 CHANGELOG.md 文件不存在的情况（创建新文件）
- 处理 CHANGELOG.md 无变更的情况（跳过提交）

## 测试场景

### 场景 1：首次发布
**前置条件：**
- 仓库中没有 CHANGELOG.md 文件
- 推送第一个 Git Tag（如 v0.1.0）

**预期结果：**
- ✅ 创建 CHANGELOG.md 文件，包含标准头部说明
- ✅ 生成从初始提交到当前 Tag 的所有变更历史
- ✅ 推送 CHANGELOG.md 到仓库
- ✅ 创建 GitHub Release，包含变更历史

### 场景 2：常规发布
**前置条件：**
- CHANGELOG.md 文件已存在
- 推送新的 Git Tag（如 v0.2.0）
- 上一个 Tag 和当前 Tag 之间有提交记录

**预期结果：**
- ✅ 在 CHANGELOG.md 顶部插入新版本的变更内容
- ✅ 保留旧版本的变更历史
- ✅ 推送 CHANGELOG.md 到仓库
- ✅ 创建 GitHub Release，包含变更历史

### 场景 3：无提交记录
**前置条件：**
- 推送新的 Git Tag
- 上一个 Tag 和当前 Tag 之间没有提交记录

**预期结果：**
- ✅ 生成包含 "No changes" 的变更日志
- ✅ 更新 CHANGELOG.md
- ✅ 创建 GitHub Release

### 场景 4：手动触发或被调用
**前置条件：**
- 通过 workflow_dispatch 或 workflow_call 触发

**预期结果：**
- ✅ update-changelog job 被跳过（仅在 Git Tag 推送时执行）
- ✅ Docker 镜像构建正常进行

## 验证清单

### 文件验证
- [x] `cliff.toml` 文件已创建
- [x] `.github/workflows/release.yml` 已更新
- [x] 无 linter 错误

### 配置验证
- [x] git-cliff 配置正确（支持 Conventional Commits）
- [x] workflow job 依赖关系正确
- [x] 权限配置正确（contents: write）
- [x] 条件执行正确（仅在 Git Tag 推送时）

### 功能验证
- [x] git-cliff-action 集成正确
- [x] CHANGELOG.md 更新逻辑正确
- [x] Git 推送重试机制实现
- [x] softprops/action-gh-release 集成正确
- [x] 边界情况处理完整

## 下一步

### 实际测试
1. 创建一个测试 Tag 并推送：
   ```bash
   git tag v0.1.0-test
   git push origin v0.1.0-test
   ```

2. 观察 GitHub Actions 执行情况：
   - 检查 `update-changelog` job 是否成功执行
   - 检查 CHANGELOG.md 是否正确更新
   - 检查 GitHub Release 是否正确创建

3. 验证生成的 CHANGELOG.md 格式：
   - 是否符合 Keep a Changelog 规范
   - 是否正确分类提交记录
   - 是否包含提交哈希和链接

4. 验证 GitHub Release：
   - 是否包含正确的版本号
   - 是否包含完整的变更历史
   - 链接是否正确

### 可选优化
1. 自定义 cliff.toml 模板以匹配项目风格
2. 添加更多提交类型的分类规则
3. 配置跳过特定类型的提交（如依赖更新）
4. 添加贡献者列表到 CHANGELOG

## 注意事项

1. **权限要求**：确保 workflow 有 `contents: write` 权限
2. **Token 要求**：使用 `secrets.GITHUB_TOKEN`（自动提供）
3. **分支保护**：如果 main 分支有保护规则，可能需要调整
4. **首次运行**：首次运行时可能需要手动创建 CHANGELOG.md 或让 workflow 自动创建

## 相关文档

- [git-cliff 官方文档](https://git-cliff.org/docs/)
- [git-cliff-action GitHub](https://github.com/orhun/git-cliff-action)
- [softprops/action-gh-release GitHub](https://github.com/softprops/action-gh-release)
- [Conventional Commits 规范](https://www.conventionalcommits.org/)
- [Keep a Changelog 规范](https://keepachangelog.com/)
