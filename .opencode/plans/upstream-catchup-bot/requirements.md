# 需求

## 目标与背景

本项目计划基于 Rust 重新实现上游 `HKUDS/nanobot`（Python）的功能。为了跟踪上游进展，需要一个 opencode primary agent，给定上游 main 分支上的两个 commit SHA，找出这两个 commit 之间合入的所有 PR，按合入顺序整理成一个 `_catchup-hkuds.md` 文件，作为后续用 planner/builder agent 逐个实现的输入。

## 方案比较

### 方案 1: Primary Agent（通过 GitHub API）

- 思路: 创建 primary agent，用 `gh` CLI 调用 GitHub API 获取 PR 列表和详情，生成 markdown 文件。
- 优点: PR 信息完整（标题、描述、标签、review 评论），不依赖本地 git 仓库有上游历史
- 缺点: 依赖 `gh` CLI 已登录，API 有速率限制

### 方案 2: Primary Agent（通过 git log + gh CLI）

- 思路: 创建 primary agent，先 `git fetch` 上游，然后用 `git log --first-parent <start>..<end>` 找出主线提交，从 commit message 中提取 PR 编号，再用 `gh pr view` 和 `gh pr diff` 获取详情。
- 优点: 合入顺序精确（基于 git 历史），可以直接获取每个 PR 的 diff，diff 信息对后续 Rust 实现更有参考价值
- 缺点: 需要本地添加 upstream remote 并 fetch

### 推荐

推荐**方案 2**。git log 能精确给出合入顺序，且 `gh pr diff` 获取的 diff 信息对后续 Rust 实现更有参考价值。`gh pr view` 补充 PR 的描述信息。两者结合信息最全。

## 功能需求列表

### 核心功能

1. **Agent 入口**：primary mode agent，用户切换后直接输入 `<start-sha> <end-sha>`
2. **上游 remote 管理**：检查 `upstream` remote 是否存在，不存在则添加 `https://github.com/HKUDS/nanobot.git`，然后 `git fetch upstream`
3. **PR 发现**：通过 `git log --first-parent <start>..<end>`（不带 `--merges`）列出主线提交，用正则 `#(\d+)` 从 commit message 提取 PR 编号。兼容 regular merge（`Merge pull request #1234`）和 squash merge（`Some title (#1234)`）两种格式
4. **PR 详情获取**：对每个 PR 编号，用 `gh pr view <number> --repo HKUDS/nanobot --json title,body,files,mergedAt,labels` 获取元数据，再用 `gh pr diff <number> --repo HKUDS/nanobot` 获取实际 diff 用于分析变更细节
5. **PR 分析与分类**：基于 PR 标题前缀（feat/fix/refactor）、标签、diff 内容，判断每个 PR 的类别和涉及模块
6. **已实现识别**：扫描本项目 git 历史，通过 commit message 中的 `HKUDS/nanobot#<编号>` 模式识别已实现的上游 PR
7. **输出文件生成**：生成 `_catchup-hkuds.md`，按合入顺序列出每个 PR 的详细分析，标注实现状态

### 输出文件格式

```markdown
# Catchup: HKUDS/nanobot

> 范围: `<start-sha-short>` .. `<end-sha-short>`
> 生成时间: YYYY-MM-DD HH:MM:SS (Asia/Shanghai)
> PR 总数: N（待实现: X，已实现: Y）

## 1. PR #1234: 标题

- **状态**: 待实现 | ✅ 已实现
- **类别**: feat | fix | refactor
- **合入时间**: YYYY-MM-DD HH:MM:SS (Asia/Shanghai)
- **涉及模块**: agent/loop, channels/telegram, config/schema（从文件路径推断，列出 nanobot/ 下的子目录级别）
- **变更文件**:
  - `nanobot/agent/loop.py`（修改）
  - `nanobot/channels/telegram.py`（修改）
  - `tests/test_agent.py`（新增）
- **概述**: 一句话说明这个 PR 解决什么问题或实现什么功能
- **关键变更**:
  - 在 AgentLoop 中新增了 `handle_timeout()` 方法，处理 LLM 响应超时的场景
  - Telegram channel 增加了 media group 消息的聚合逻辑，将同一组图片合并为一条消息处理
  - 新增配置项 `agent.timeout_seconds`，默认值 30
- **上下文/动机**: 从 PR body 中提取的背景信息，说明为什么需要这个变更

---

## 2. PR #1235: 标题

...
```

### 提交消息约定

实现上游 PR 时，commit message 中须包含 `HKUDS/nanobot#<PR编号>`（GitHub 标准跨仓库引用格式），以便 agent 自动识别已实现的 PR。

```
feat: implement agent timeout handling (HKUDS/nanobot#1234)
fix: port telegram media-group fix (HKUDS/nanobot#1258)
```

## 非功能需求

- **兼容性**：依赖 `git` 和 `gh` CLI
- **安全**：Agent 仅需 bash 中的 `git` 和 `gh` 只读命令权限，以及写入 `_catchup-hkuds.md` 的文件编辑权限

## 边界与不做事项

- 不执行任何代码合入、cherry-pick 或合并操作
- 不修改项目源代码
- 不负责后续的 Rust 实现（由 planner/builder agent 处理）
- 输出文件 `_catchup-hkuds.md` 不纳入版本控制，需在 `.gitignore` 中添加 `_catchup-hkuds.md` 规则（现有 `/_*/` 仅匹配目录，不覆盖该文件）

## 假设与约束

- **技术假设**：本地已安装 `gh` CLI 并已通过 `gh auth login` 登录
- **技术假设**：用户提供的两个 commit SHA 都在上游 `main` 分支上

## 待确认事项

无
