# 需求文档

## 引言
本需求旨在改进 GitHub Actions 工作流，将现有的 `dockerize.yml` 工作流重命名为 `release.yml`，并增加版本一致性校验步骤，确保 Git Tag 与 `crates/cli/Cargo.toml` 中的 `package.version` 保持一致，从而保证发布版本的准确性和可追溯性。

## 需求

### 需求 1：工作流重命名

**用户故事：** 作为一名开发者，我希望工作流名称能够准确反映其功能（发布而非仅构建镜像），以便更好地理解和管理 CI/CD 流程。

#### 验收标准

1. WHEN 工作流文件被修改 THEN 系统 SHALL 将文件名从 `dockerize.yml` 更改为 `release.yml`
2. WHEN 工作流文件被修改 THEN 系统 SHALL 将工作流 `name` 字段从 `dockerize` 更改为 `release`
3. WHEN 工作流重命名完成后 THEN 系统 SHALL 保持所有现有功能正常运行不变

### 需求 2：版本一致性校验

**用户故事：** 作为一名开发者，我希望在发布新版本时自动校验 Git Tag 与代码中的版本号是否一致，以便避免版本不匹配导致的发布错误。

#### 验收标准

1. WHEN 工作流由 Git Tag 推送触发 THEN 系统 SHALL 在构建镜像之前执行版本校验步骤
2. WHEN 执行版本校验 THEN 系统 SHALL 从 Git Tag 中提取版本号（Tag 格式为 `0.1.1`，无需去除前缀）
3. WHEN 执行版本校验 THEN 系统 SHALL 从 `crates/cli/Cargo.toml` 文件中读取 `package.version` 字段值
4. IF Git Tag 版本号与 `Cargo.toml` 版本号一致 THEN 系统 SHALL 继续执行后续构建步骤
5. IF Git Tag 版本号与 `Cargo.toml` 版本号不一致 THEN 系统 SHALL 终止工作流并输出明确的错误信息
6. WHEN 版本校验失败 THEN 系统 SHALL 在错误信息中同时显示 Tag 版本和 `Cargo.toml` 版本，便于开发者排查

### 需求 3：校验步骤的位置与依赖关系

**用户故事：** 作为一名开发者，我希望版本校验在构建流程的早期执行，以便快速发现问题，避免浪费构建资源。

#### 验收标准

1. WHEN 工作流执行 THEN 系统 SHALL 在"Checkout repository"步骤之后、"Get Git commit short SHA"步骤之前执行版本校验
2. WHEN 版本校验步骤执行 THEN 系统 SHALL 能够访问已检出的代码仓库
3. WHEN 版本校验步骤执行 THEN 系统 SHALL 能够获取触发工作流的 Git Tag 信息

### 需求 4：手动触发模式的兼容性

**用户故事：** 作为一名开发者，我希望手动触发工作流时不受版本校验的影响，以便在特殊情况下（如紧急修复）能够灵活发布。

#### 验收标准

1. WHEN 工作流由手动触发（workflow_dispatch）THEN 系统 SHALL 跳过版本一致性校验步骤
2. WHEN 手动触发工作流 THEN 系统 SHALL 输出提示信息说明版本校验已跳过
3. WHEN 手动触发工作流 THEN 系统 SHALL 继续使用输入的 tag 参数或默认的 `alpha-{git commit id}` 标签进行构建
