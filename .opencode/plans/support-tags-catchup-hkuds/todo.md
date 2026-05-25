# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `.opencode/agents/catchup-hkuds.md` | 修改 | agent 指令文件，增加 tag 输入支持、验证解析步骤、输出格式适配 |

## 任务列表

### ✅ 1. 更新 frontmatter 和开头描述

- 优先级: P0
- 依赖项: 无
- 涉及文件: `.opencode/agents/catchup-hkuds.md`
- 验收标准: frontmatter description 和开头段落准确反映支持 SHA 和 tag 两种输入
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [x] 将 frontmatter `description` 从 "两个 commit 之间" 改为 "两个 commit 引用（SHA 或 tag）之间"
  - [x] 将第 12 行开头段落中 "两个 commit SHA" 改为 "两个 commit 引用（SHA 或 tag）"

### ✅ 2. 扩展输入格式说明

- 优先级: P0
- 依赖项: 1
- 涉及文件: `.opencode/agents/catchup-hkuds.md`
- 验收标准: 输入部分明确说明支持 SHA、tag 及混合格式，并给出示例
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [x] 将 "## 输入" 部分的 "两个 commit SHA（start 和 end）" 改为 "两个 commit 引用（start 和 end），支持 commit SHA 或 git tag"
  - [x] 将格式示例从单一的 `<start-sha> <end-sha>` 扩展为多种格式示例：纯 SHA（`abc1234 def5678`）、纯 tag（`v0.1.4 v0.2.0`）、混合（`v0.1.4 def5678`）

### ✅ 3. 更新步骤 2 以拉取 tags

- 优先级: P0
- 依赖项: 2
- 涉及文件: `.opencode/agents/catchup-hkuds.md`
- 验收标准: `git fetch upstream` 命令包含 `--tags` 参数
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [x] 将步骤 2 的命令从 `git fetch upstream` 改为 `git fetch upstream --tags`

### ✅ 4. 新增步骤 2.5：引用验证与解析

- 优先级: P0
- 依赖项: 3
- 涉及文件: `.opencode/agents/catchup-hkuds.md`
- 验收标准: 在步骤 2 和步骤 3 之间插入新步骤，包含 `git rev-parse --verify` 验证命令和解析逻辑说明
- 风险/注意点: 步骤编号使用 2.5 以保持与现有 4.5 的半步编号风格一致，避免全局重编号
- 信心评估: 5
- 步骤:
  - [x] 在步骤 2 之后、步骤 3 之前插入 "### 步骤 2.5: 验证并解析输入引用"
  - [x] 内容包含：对 start 和 end 分别执行 `git rev-parse --verify <ref>` 验证引用存在性，失败则报错并终止
  - [x] 内容包含：记录原始输入（`start_ref`、`end_ref`）和解析后的 SHA（`start_sha`、`end_sha`），后续步骤使用解析后的 SHA 进行 git log 操作
  - [x] 内容包含：判断输入类型的说明——如果输入匹配 40 位十六进制则为 SHA，否则视为 tag

### ✅ 5. 更新步骤 9 输出格式

- 优先级: P0
- 依赖项: 4
- 涉及文件: `.opencode/agents/catchup-hkuds.md`
- 验收标准: 输出文件头部的范围行在输入为 tag 时展示 `tag (sha-7位)`，纯 SHA 时保持原有的 `sha-7位` 格式
- 风险/注意点: 需同时更新模板示例和格式要求说明
- 信心评估: 5
- 步骤:
  - [x] 将步骤 9 中范围行模板从 `` > 范围: `<start-sha-7位>` .. `<end-sha-7位>` `` 改为 `` > 范围: `<start_ref>` (`<start-sha-7位>`) .. `<end_ref>` (`<end-sha-7位>`) ``
  - [x] 添加说明：当输入本身就是 SHA 时，范围行简化为 `` > 范围: `<sha-7位>` .. `<sha-7位>` ``（不重复显示）

## 实现建议

- 所有改动集中在单个 markdown 文件，按任务顺序从上到下依次修改即可
- 步骤 2.5 的 `git rev-parse --verify` 命令对 SHA 和 tag 都有效，无需区分处理
- 上游 tag 命名规范为 `v` 前缀（如 `v0.1.4`、`v0.2.0`），本地 tag 无 `v` 前缀（如 `1.0.0`），agent 指令中不应硬编码前缀假设
