# 需求文档

## 引言

本功能旨在为项目的 CI 构建流程增加 Conventional Commit 风格校验，确保所有 Merge Request (MR) 中新增的提交消息符合 Conventional Commits 规范（https://www.conventionalcommits.org/en/v1.0.0/）。这将提高提交历史的可读性，便于自动化工具生成变更日志，并促进团队协作的规范化。

Conventional Commit 格式要求提交消息遵循以下结构：
- `<type>[optional scope]: <description>`
- 可选的 body 和 footer
- 支持破坏性变更标记（`!`）

## 需求

### 需求 1：获取 MR 新增的提交消息

**用户故事：** 作为一名开发者，我希望 CI 系统能够自动获取 MR 中新增的所有提交消息，以便对这些提交进行风格校验。

#### 验收标准

1. WHEN 触发 pull_request 事件 THEN 系统 SHALL 自动获取该 MR 相对于目标分支（main）新增的所有提交消息
2. IF MR 中没有新增提交 THEN 系统 SHALL 跳过校验并标记为通过
3. WHEN 获取提交消息时 THEN 系统 SHALL 获取每个提交的完整消息内容（标题为必需，body 和 footer 为可选）

### 需求 2：校验提交消息格式

**用户故事：** 作为一名项目维护者，我希望所有新增的提交消息都符合 Conventional Commit 规范，以便保持提交历史的一致性和可读性。

#### 验收标准

1. WHEN 校验提交消息时 THEN 系统 SHALL 检查消息是否符合 `<type>[optional scope]: <description>` 格式
2. IF 提交消息包含破坏性变更标记（`!`）THEN 系统 SHALL 允许该标记出现在 type/scope 之后
3. IF 提交消息包含可选的 scope THEN 系统 SHALL 允许 scope 使用括号包裹（如 `feat(api):`）
4. WHEN 校验 type 时 THEN 系统 SHALL 接受以下标准类型：`feat`、`fix`、`docs`、`style`、`refactor`、`test`、`build`、`ci`、`chore`、`revert`
5. IF 提交消息不符合格式要求 THEN 系统 SHALL 拒绝该提交并返回详细的错误信息

### 需求 3：提供清晰的错误反馈

**用户故事：** 作为一名开发者，当我的提交消息不符合规范时，我希望获得清晰的错误提示，以便快速修正问题。

#### 验收标准

1. IF 提交消息校验失败 THEN 系统 SHALL 输出所有失败的提交消息及其错误原因
2. WHEN 输出错误信息时 THEN 系统 SHALL 包含正确的 Conventional Commit 格式示例
3. IF 多个提交消息校验失败 THEN 系统 SHALL 列出所有失败的提交，而不是仅报告第一个错误

### 需求 4：集成到现有构建流程

**用户故事：** 作为一名项目维护者，我希望提交消息校验能够无缝集成到现有的 CI 构建流程中，以便在代码合并前自动执行检查。

#### 验收标准

1. WHEN pull_request 事件触发时 THEN 系统 SHALL 在构建流程中执行提交消息校验步骤
2. IF 提交消息校验失败 THEN 系统 SHALL 阻止后续构建步骤的执行
3. WHEN 执行校验时 THEN 系统 SHALL 使用现有的 GitHub Actions 环境，无需额外配置复杂的依赖
4. IF 校验通过 THEN 系统 SHALL 继续执行后续的构建步骤（格式检查、Clippy、构建、测试）

### 需求 5：支持手动触发

**用户故事：** 作为一名开发者，我希望能够手动触发包含提交消息校验的构建流程，以便在需要时重新执行检查。

#### 验收标准

1. WHEN 通过 workflow_dispatch 触发构建 THEN 系统 SHALL 执行提交消息校验步骤
2. IF 手动触发时不是 pull_request 上下文 THEN 系统 SHALL 跳过提交消息校验并继续执行其他步骤
