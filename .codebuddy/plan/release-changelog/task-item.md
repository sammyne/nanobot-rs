# 实施计划

- [ ] 1. 创建 git-cliff 配置文件
   - 创建 `cliff.toml` 配置文件
   - 配置 Conventional Commits 解析规则（feat、fix、docs、chore、refactor、test、style、perf）
   - 配置 Keep a Changelog 格式模板
   - 配置 breaking change 识别和展示规则
   - _需求：1.1、1.2、1.3、1.4、2.1、2.2、2.5_

- [ ] 2. 修改 release workflow 添加 changelog 生成 job
   - 在 `.github/workflows/release.yml` 中添加新的 job `update-changelog`
   - 配置 job 依赖关系（在 Docker 镜像构建成功后执行）
   - 设置 job 权限（contents: write）
   - 配置所需的环境变量（GITHUB_TOKEN）
   - _需求：5.1、5.2、5.3_

- [ ] 3. 集成 git-cliff-action 生成 CHANGELOG
   - 使用 `orhun/git-cliff-action@v4` 生成变更历史
   - 配置参数：config、args（指定 tag）
   - 将生成的 changelog 内容输出到变量
   - 添加详细的执行日志输出
   - _需求：1.1、1.2、1.3、1.4、2.1、2.2、2.5、5.3_

- [ ] 4. 实现 CHANGELOG.md 文件更新和推送
   - 检查 CHANGELOG.md 文件是否存在
   - 使用 git-cliff 的 `--output` 参数更新 CHANGELOG.md 文件
   - 配置 Git 用户信息（github-actions[bot]）
   - 创建提交并推送到仓库（消息格式：`chore: update CHANGELOG.md for v{version}`）
   - 实现推送重试机制（最多 3 次）
   - 输出提交 SHA 和变更摘要
   - _需求：2.3、2.4、3.1、3.2、3.3、3.4_

- [ ] 5. 使用 softprops/action-gh-release 创建 GitHub Release
   - 使用 `softprops/action-gh-release@v2` 创建 Release
   - 将版本号作为标题
   - 将 git-cliff 生成的变更历史作为发布说明内容
   - 处理 Release 已存在的情况（更新现有 Release）
   - 输出 Release URL
   - _需求：4.1、4.2、4.3、4.4、4.5_

- [ ] 6. 实现边界情况处理
   - 处理两个 Tag 之间没有提交记录的情况（git-cliff 会生成空内容，需要添加 "No changes"）
   - 处理首次发布（没有上一个 Tag）的情况（git-cliff 支持从初始提交开始）
   - 处理 Git 历史获取失败的情况（记录错误并跳过）
   - 处理 CHANGELOG.md 文件格式异常的情况
   - _需求：5.4、6.1、6.2、6.3、6.4_

- [ ] 7. 测试和验证
   - 测试首次发布场景（从初始提交到第一个 Tag）
   - 测试常规发布场景（两个 Tag 之间有提交）
   - 测试无提交记录场景（两个 Tag 之间无提交）
   - 验证生成的 CHANGELOG.md 格式是否符合 Keep a Changelog 规范
   - 验证 GitHub Release 页面的发布说明是否正确
   - 测试 workflow 集成的完整流程
   - _需求：所有需求_
