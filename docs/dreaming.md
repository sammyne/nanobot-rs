# Dream 记忆系统

> 参考上游 HKUDS/nanobot PR #2717 设计，适配 Rust 版实现。

## 设计理念

nanobot 的记忆不是一堆笔记，而是一个安静的注意力系统：注意到值得保留的内容，放下不再需要聚光灯的内容，将经历转化为平静、持久、有用的东西。

记忆被分为不同层次，因为不同类型的记忆需要不同的工具。

## 架构

```
session.messages          短期：当前对话
        │
        ▼ (token 预算超出时)
┌──────────────────┐
│   Consolidator   │     每轮：纯文本 LLM 摘要
│   (Stage 1)      │     不调用工具，不依赖 tool_choice
└────────┬─────────┘
         ▼
memory/history.jsonl      中期：append-only 结构化历史
         │
         ▼ (cron 定时 / /dream 手动)
┌──────────────────┐
│     Dream        │     定期：两阶段 LLM 驱动记忆编辑
│   (Stage 2)      │
│                  │     Phase 1: 分析新历史 vs 现有记忆
│                  │     Phase 2: read_file + edit_file 增量编辑
└────────┬─────────┘
         ▼
SOUL.md / USER.md /       长期：持久知识文件
memory/MEMORY.md          由 Dream 自动维护
         │
         ▼
memory/.git/              版本：git 版本控制
                          每次 Dream 编辑后自动 commit
```

## 文件布局

```
workspace/
├── SOUL.md              # bot 的长期语气和沟通风格
├── USER.md              # 关于用户的稳定知识
└── memory/
    ├── MEMORY.md        # 项目事实、决策和持久上下文
    ├── history.jsonl    # append-only 历史摘要
    ├── .cursor          # Consolidator 写入游标
    ├── .dream_cursor    # Dream 消费游标
    └── .git/            # 长期记忆文件的版本历史
```

各文件职责：

- **SOUL.md** — 记住 nanobot 应该如何表达
- **USER.md** — 记住用户是谁以及他们的偏好
- **MEMORY.md** — 记住关于工作本身的持久事实
- **history.jsonl** — 记住到达这里的过程

## Stage 1: Consolidator

当对话增长到压迫上下文窗口时，Consolidator 将最旧的安全消息切片摘要为文本，追加到 `history.jsonl`。

### 与旧版的区别

| 维度 | 旧版 | 新版 |
|------|------|------|
| LLM 调用方式 | `tool_choice=Required` 调用 `save_memory` 工具 | 纯文本 LLM 摘要（`tools=None`） |
| Provider 兼容性 | 部分 provider 不支持 `tool_choice` | 所有 provider 兼容 |
| 输出格式 | 工具调用参数（JSON） | 纯文本 |
| 存储格式 | HISTORY.md（纯文本） | history.jsonl（结构化 JSONL） |
| 重复处理 | 无防护 | cursor 机制防止重复 |

### history.jsonl 格式

```json
{"cursor": 42, "timestamp": "2026-05-29T10:00:00Z", "content": "- 用户偏好暗色模式\n- 决定使用 PostgreSQL"}
```

特点：
- append-only，cursor 自增
- 超过 1000 条时自动 compaction（截断旧条目）
- 机器消费优先，人类可检索

### 摘要提示词分类

摘要按优先级排列：
1. **用户纠正** — 用户明确纠正的错误理解
2. **解决方案** — 试错过程中发现的有效方案
3. **决策** — 做出的技术或流程决策
4. **事件** — 发生的重要事件
5. **环境事实** — 关于环境的客观信息

## Stage 2: Dream

Dream 是更慢、更深思熟虑的层。默认每 2 小时通过 cron 运行一次，也可通过 `/dream` 手动触发。

### Phase 1: 分析

Dream 读取：
- `history.jsonl` 中未处理的新条目（dream_cursor 之后）
- 当前 `SOUL.md`、`USER.md`、`memory/MEMORY.md`

LLM 对比新历史和现有记忆，输出 `[FILE] atomic fact` 格式的编辑指令：

```
[MEMORY.md] 项目从 SQLite 迁移到 PostgreSQL
[USER.md] 用户偏好使用 vim 键绑定
[SOUL.md] 回复时应避免使用 emoji
```

### Phase 2: 编辑

使用 agent 循环（re_act）+ 受限工具集（只有 `read_file` + `edit_file`），对记忆文件进行增量编辑。最多执行 `max_iterations` 次工具调用（默认 10）。

编辑完成后：
1. 推进 dream_cursor
2. 通过 GitStore 自动 commit

### 为什么是两阶段

单阶段方案（直接让 LLM 重写整个 MEMORY.md）的问题：
- 随着记忆增长，LLM 需要输出越来越多的 token
- 已有事实容易在重写过程中丢失
- token 成本线性增长

两阶段方案的优势：
- Phase 1 只输出需要变更的原子事实（几行文本）
- Phase 2 通过 edit_file 做精确的增量编辑
- 已有内容不会被意外覆盖

## GitStore

通过 `std::process::Command` 调用系统 `git` 命令实现版本控制。

**前置条件**：运行环境必须安装 `git` 命令。git 不可用时报错，不做任何静默处理。

支持的操作：

| 操作 | git 命令 | 用途 |
|------|----------|------|
| init | `git init` + `.gitignore` | 初始化仓库 |
| commit | `git add .` + `git commit -m` | 记录变更 |
| log | `git log --format` | 查看历史 |
| diff | `git diff sha~1 sha` | 查看变更内容 |
| revert | `git checkout sha -- .` + commit | 回退到指定版本 |

`.gitignore` 只跟踪记忆文件（MEMORY.md、SOUL.md、USER.md），排除 history.jsonl 和 cursor 文件。

## 命令

| 命令 | 功能 |
|------|------|
| `/dream` | 立即运行 Dream |
| `/dream-log` | 显示最近一次 Dream 变更 |
| `/dream-log <sha>` | 显示指定 Dream 变更 |
| `/dream-restore` | 列出最近的 Dream 版本 |
| `/dream-restore <sha>` | 回退记忆到指定版本 |

## 配置

```json
{
  "agents": {
    "defaults": {
      "dream": {
        "cron": "0 */2 * * *",
        "model": null,
        "maxBatchSize": 20,
        "maxIterations": 10
      }
    }
  }
}
```

| 字段 | 含义 | 默认值 |
|------|------|--------|
| `cron` | Dream 运行的 cron 表达式 | `"0 */2 * * *"`（每 2 小时） |
| `model` | Dream 使用的模型（null 则与主 agent 相同） | `null` |
| `maxBatchSize` | 每次 Dream 处理的最大历史条目数 | `20` |
| `maxIterations` | Phase 2 的最大工具调用次数 | `10` |

## 迁移

从旧版（HISTORY.md）迁移到新版（history.jsonl）：
- 已有 HISTORY.md 内容保留不删除
- 新的摘要写入 history.jsonl
- cursor 从 0 开始
