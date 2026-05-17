# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `opencode.json` | 新增 | opencode 项目配置，bash 权限规则 |
| `.opencode/agents/catchup-hkuds.md` | 新增 | catchup-hkuds primary agent 定义，包含系统提示词和权限配置 |
| `.gitignore` | 修改 | 添加 `_catchup-hkuds.md` 排除规则 |
| `README.md` | 修改 | 添加上游同步提交消息约定 |

## 任务列表

### ✅ 1. 创建 opencode.json 项目配置

- 优先级: P0
- 依赖项: 无
- 涉及文件: `opencode.json`
- 验收标准: opencode 启动时能加载配置，bash 权限规则生效
- 风险/注意点: 项目当前无 opencode.json，是全新创建；`$schema` 字段必须指向 `https://opencode.ai/config.json`
- 步骤:
  - [x] 创建 `opencode.json`，包含 `$schema` 声明
  - [x] 在 `permission.bash` 中允许 `git *` 和 `gh *`，其余 `*` 设为 `ask`

### ✅ 2. 创建 catchup-hkuds agent 定义文件

- 优先级: P0
- 依赖项: 1
- 涉及文件: `.opencode/agents/catchup-hkuds.md`
- 验收标准: 在 opencode 中切换到 catchup-hkuds agent，输入 `<start-sha> <end-sha>` 后，agent 能按步骤执行并生成 `_catchup-hkuds.md`
- 风险/注意点: agent 的 `mode` 为 `primary`，用户通过切换 agent 使用；系统提示词需要覆盖完整工作流步骤
- 步骤:
  - [x] 创建 `.opencode/agents/` 目录
  - [x] 创建 `catchup-hkuds.md`，frontmatter 中设置 `description`、`mode: primary`、`permission`（bash 中允许 `git *` 和 `gh *`，edit 允许）
  - [x] 编写系统提示词，包含以下工作流步骤：
    1. 检查 `upstream` remote 是否存在，不存在则执行 `git remote add upstream https://github.com/HKUDS/nanobot.git`
    2. 执行 `git fetch upstream`
    3. 执行 `git log --first-parent --format='%H %s' <start>..<end>` 获取主线提交列表
    4. 对每条 commit message 用正则 `#(\d+)` 提取 PR 编号；无法提取的记录为"直接提交"
    5. 扫描本项目 git 历史（`git log --all --oneline --grep='HKUDS/nanobot#'`），识别已实现的上游 PR
    6. 对每个 PR 编号执行 `gh pr view <number> --repo HKUDS/nanobot --json title,body,files,mergedAt,labels` 获取元数据
    7. 对每个 PR 执行 `gh pr diff <number> --repo HKUDS/nanobot` 获取 diff，分析关键变更点
    8. 基于 PR 标题前缀（`feat:`/`fix:`/`refactor:`）、标签、diff 内容判断类别（feat/fix/refactor）
    9. 从变更文件路径提取涉及模块（`nanobot/` 下的子目录级别，如 `agent/loop`、`channels/telegram`、`config/schema`）
    10. 将结果按合入顺序写入 `_catchup-hkuds.md`，每个 PR 包含：状态（待实现/已实现）、类别、合入时间（精确到秒，Asia/Shanghai 时区）、涉及模块、变更文件（标注新增/修改/删除）、概述、关键变更（列出具体的函数/类/配置项变动）、上下文/动机
  - [x] 在提示词中明确输出文件格式模板（与 requirements.md 中定义的格式一致）
  - [x] 在提示词中说明时区转换规则：GitHub API 返回的 `mergedAt` 是 UTC 时间（ISO 8601 格式），需转换为 Asia/Shanghai（UTC+8），输出格式为 `YYYY-MM-DD HH:MM:SS`
  - [x] 在提示词中说明：如果 PR 的 diff 过大（超过 500 行），只分析核心文件的变更，跳过测试文件和自动生成文件的逐行分析
  - [x] 在提示词中说明已实现识别规则：扫描本项目 git 历史中的 `HKUDS/nanobot#<编号>` 模式

### ✅ 3. 更新 .gitignore

- 优先级: P1
- 依赖项: 无
- 涉及文件: `.gitignore`
- 验收标准: `git status` 不显示 `_catchup-hkuds.md`
- 风险/注意点: 在文件末尾追加即可，不改动现有规则
- 步骤:
  - [x] 在 `.gitignore` 末尾添加 `_catchup-hkuds.md`

### ✅ 4. 更新 README.md

- 优先级: P1
- 依赖项: 无
- 涉及文件: `README.md`
- 验收标准: README 中包含上游同步章节和提交消息约定
- 风险/注意点: 在 `🤝 Contribute & Roadmap` 之前插入，不改动其他内容
- 步骤:
  - [x] 添加"🔄 上游同步（Upstream Catchup）"章节，包含提交消息约定和示例

## 实现建议

- agent 提示词中对 `git log` 输出的解析应容错：有些 commit 可能没有 PR 编号（直接 push 到 main），这些应在输出文件中标记为"直接提交"并附上 commit message
- `gh pr view` 的 `--json` 输出是结构化 JSON，agent 可以直接解析 `title`、`body`、`files`、`mergedAt`、`labels` 字段
- `gh pr diff` 返回标准 unified diff 格式，agent 应从中提取：新增/修改/删除了哪些函数、类、配置项，而非逐行罗列 diff
- 上游项目的模块划分：`agent/`（核心引擎）、`bus/`（消息总线）、`channels/`（通信渠道）、`config/`（配置）、`providers/`（LLM 提供者）、`skills/`（技能）、`cli/`（命令行）、`utils/`（工具函数）、`session/`（会话管理）、`cron/`（定时任务）、`heartbeat/`（心跳）、`templates/`（模板）——agent 提示词中应列出这些模块名，便于分类
- 修改完成后需重启 opencode 使配置生效
