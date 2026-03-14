# 实施计划

- [x] 1. 重命名工作流文件和更新工作流名称
   - 将 `.github/workflows/dockerize.yml` 重命名为 `.github/workflows/release.yml`
   - 将工作流 `name` 字段从 `dockerize` 更改为 `release`
   - _需求：1.1、1.2_

- [x] 2. 实现版本一致性校验步骤
   - 在 "Checkout repository" 步骤之后添加新的校验步骤
   - 使用 shell 命令从 `crates/cli/Cargo.toml` 中提取 `package.version` 字段值
   - 使用 `${{ github.ref_name }}` 获取 Git Tag 版本号
   - 比较两个版本号是否一致
   - _需求：2.2、2.3、3.1、3.2、3.3_

- [x] 3. 添加版本不一致时的错误处理逻辑
   - 当版本不一致时，使用 `exit 1` 终止工作流
   - 输出包含 Tag 版本和 Cargo.toml 版本的详细错误信息
   - _需求：2.4、2.5、2.6_

- [x] 4. 实现手动触发模式的条件判断
   - 添加条件判断逻辑，检测是否为 `workflow_dispatch` 触发
   - 手动触发时跳过版本校验步骤并输出提示信息
   - _需求：4.1、4.2_

- [x] 5. 更新手动触发模式的默认标签逻辑
   - 修改 `workflow_dispatch` 的 `tag` 输入参数默认值
   - 实现动态生成 `alpha-{git commit id}` 格式的默认标签
   - _需求：4.3_

- [ ] 6. 测试工作流的完整功能
   - 测试 Git Tag 触发时的版本校验功能（版本一致和不一致两种情况）
   - 测试手动触发时跳过版本校验的功能
   - 验证手动触发时默认标签 `alpha-{git commit id}` 的正确性
   - _需求：1.3、2.1、2.4、2.5、4.1、4.3_
