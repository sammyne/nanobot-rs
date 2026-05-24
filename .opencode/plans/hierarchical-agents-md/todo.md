# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `AGENTS.md` | 修改 | 新增 "Crate 依赖关系" 章节和层级说明 |
| `crates/nanobot/AGENTS.md` | 新增 | binary crate 的关键类型、内部依赖 |
| `crates/agent/AGENTS.md` | 新增 | agent crate 的关键类型、内部依赖 |
| `crates/provider/AGENTS.md` | 新增 | provider crate 的关键类型、内部依赖 |
| `crates/config/AGENTS.md` | 新增 | config crate 的关键类型、内部依赖 |
| `crates/tools/AGENTS.md` | 新增 | tools crate 的关键类型、内部依赖 |
| `crates/mcp/AGENTS.md` | 新增 | mcp crate 的关键类型、内部依赖 |
| `crates/session/AGENTS.md` | 新增 | session crate 的关键类型、内部依赖 |
| `crates/memory/AGENTS.md` | 新增 | memory crate 的关键类型、内部依赖 |
| `crates/context/AGENTS.md` | 新增 | context crate 的关键类型、内部依赖 |
| `crates/channels/AGENTS.md` | 新增 | channels crate 的关键类型、内部依赖 |
| `crates/skills/AGENTS.md` | 新增 | skills crate 的关键类型、内部依赖 |
| `crates/subagent/AGENTS.md` | 新增 | subagent crate 的关键类型、内部依赖 |
| `crates/cron/AGENTS.md` | 新增 | cron crate 的关键类型、内部依赖 |
| `crates/heartbeat/AGENTS.md` | 新增 | heartbeat crate 的关键类型、内部依赖 |
| `crates/templates/AGENTS.md` | 新增 | templates crate 的关键类型、内部依赖 |
| `crates/utils/AGENTS.md` | 新增 | utils crate 的关键类型、内部依赖 |

## 任务列表

### 1. ✅ 更新根 AGENTS.md

- 优先级: P0
- 依赖项: 无
- 涉及文件: `AGENTS.md`
- 验收标准: "项目结构" 章节末尾新增一行说明层级 AGENTS.md 的存在；新增 "Crate 依赖关系" 章节包含依赖树和统计信息；原有章节内容不变
- 风险/注意点: 只做追加，不改动已有内容
- 步骤:
  - [ ] 读取 `AGENTS.md` 确认当前内容
  - [ ] 在 "项目结构" 代码块结束后、"工作空间规范" 之前，插入层级说明行：`每个 crate 目录下有独立的 AGENTS.md，包含该 crate 的关键类型和公共 API。`
  - [ ] 在同一位置插入 "Crate 依赖关系" 章节，内容包含依赖树（从各 crate 的 Cargo.toml 中提取 `nanobot-*` 依赖）和统计信息（叶子 crate、被依赖最多的 crate）
  - [ ] 确认原有章节未被修改

### 2. ✅ 创建核心 crate 的 AGENTS.md（agent, provider, context, tools）

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/{agent,provider,context,tools}/AGENTS.md`
- 验收标准: 每个文件包含一句话职责、关键公共类型（含核心方法签名）、内部依赖列表；类型名和方法签名与源码一致
- 风险/注意点: 这 4 个 crate 是开发中最常改动的，信息准确性最重要
- 步骤:
  - [ ] 读取 `crates/agent/src/lib.rs` 和 `crates/agent/Cargo.toml`，提取公共类型和内部依赖，编写 `crates/agent/AGENTS.md`
  - [ ] 读取 `crates/provider/src/lib.rs` 和 `crates/provider/Cargo.toml`，提取公共类型和内部依赖，编写 `crates/provider/AGENTS.md`
  - [ ] 读取 `crates/context/src/lib.rs` 和 `crates/context/Cargo.toml`，提取公共类型和内部依赖，编写 `crates/context/AGENTS.md`
  - [ ] 读取 `crates/tools/src/lib.rs` 和 `crates/tools/Cargo.toml`，提取公共类型和内部依赖，编写 `crates/tools/AGENTS.md`
  - [ ] 抽查各文件中的方法签名是否与对应源文件一致

### 3. ✅ 创建基础设施 crate 的 AGENTS.md（config, session, memory, skills, mcp）

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/{config,session,memory,skills,mcp}/AGENTS.md`
- 验收标准: 同任务 2
- 风险/注意点: 无
- 步骤:
  - [ ] 读取 `crates/config/src/lib.rs` 和 `Cargo.toml`，编写 `crates/config/AGENTS.md`
  - [ ] 读取 `crates/session/src/lib.rs` 和 `Cargo.toml`，编写 `crates/session/AGENTS.md`
  - [ ] 读取 `crates/memory/src/lib.rs` 和 `Cargo.toml`，编写 `crates/memory/AGENTS.md`
  - [ ] 读取 `crates/skills/src/lib.rs` 和 `Cargo.toml`，编写 `crates/skills/AGENTS.md`
  - [ ] 读取 `crates/mcp/src/lib.rs` 和 `Cargo.toml`，编写 `crates/mcp/AGENTS.md`

### 4. ✅ 创建服务层 crate 的 AGENTS.md（channels, subagent, cron, heartbeat）

- 优先级: P1
- 依赖项: 无
- 涉及文件: `crates/{channels,subagent,cron,heartbeat}/AGENTS.md`
- 验收标准: 同任务 2
- 风险/注意点: 无
- 步骤:
  - [ ] 读取 `crates/channels/src/lib.rs` 和 `Cargo.toml`，编写 `crates/channels/AGENTS.md`
  - [ ] 读取 `crates/subagent/src/lib.rs` 和 `Cargo.toml`，编写 `crates/subagent/AGENTS.md`
  - [ ] 读取 `crates/cron/src/lib.rs` 和 `Cargo.toml`，编写 `crates/cron/AGENTS.md`
  - [ ] 读取 `crates/heartbeat/src/lib.rs` 和 `Cargo.toml`，编写 `crates/heartbeat/AGENTS.md`

### 5. ✅ 创建辅助 crate 的 AGENTS.md（nanobot, templates, utils）

- 优先级: P1
- 依赖项: 无
- 涉及文件: `crates/{nanobot,templates,utils}/AGENTS.md`
- 验收标准: 同任务 2；templates 和 utils 作为叶子 crate 内部依赖为空
- 风险/注意点: nanobot 是 binary crate，关键类型是 clap 命令结构体
- 步骤:
  - [ ] 读取 `crates/nanobot/src/lib.rs` 和 `Cargo.toml`，编写 `crates/nanobot/AGENTS.md`
  - [ ] 读取 `crates/templates/src/lib.rs` 和 `Cargo.toml`，编写 `crates/templates/AGENTS.md`
  - [ ] 读取 `crates/utils/src/lib.rs` 和 `Cargo.toml`，编写 `crates/utils/AGENTS.md`

### 6. ✅ 清理废弃的规划文件

- 优先级: P2
- 依赖项: 无
- 涉及文件: `.opencode/plans/project-scan-cache/`
- 验收标准: 目录已删除
- 风险/注意点: 无
- 步骤:
  - [ ] 删除 `.opencode/plans/project-scan-cache/` 目录

### 7. ✅ 为复杂 crate 补充架构描述

- 优先级: P0
- 依赖项: 任务 2-5（已完成）
- 涉及文件: `crates/{agent,provider,channels,tools,skills,cron,session,subagent}/AGENTS.md`
- 验收标准: 每个文件新增 `## 架构` 章节（位于职责描述和关键类型之间），简明描述核心设计模式和内部组件关系；架构描述与实际代码一致
- 风险/注意点: 需要读取各 crate 的核心源文件理解内部架构，不能仅靠 lib.rs 的导出
- 步骤:
  - [ ] agent: 读取 `src/loop/mod.rs`、`src/cmd/mod.rs`，描述 ReAct 循环流程和命令分发机制
  - [ ] provider: 读取 `src/base/mod.rs`、`src/any/mod.rs`，描述 Provider trait 抽象 + 双实现
  - [ ] channels: 读取 `src/traits/mod.rs`、`src/manager/mod.rs`，描述 Channel trait + ChannelManager 路由
  - [ ] tools: 读取 `src/core.rs`、`src/registry.rs`，描述 Tool trait + ToolRegistry 分发模式
  - [ ] skills: 读取 `src/loader/mod.rs`、`src/dependency/mod.rs`，描述双目录扫描 + 依赖检查
  - [ ] cron: 读取 `src/service/mod.rs`、`src/storage/mod.rs`，描述 Service/Storage/Scheduler 分层
  - [ ] session: 读取 `src/manager.rs`、`src/session.rs`，描述 JSONL 持久化 + 内存缓存
  - [ ] subagent: 读取 `src/manager.rs`、`src/tool.rs`，描述任务生命周期管理

### 8. ✅ 将架构描述替换为带方框的 ASCII 图

- 优先级: P0
- 依赖项: 任务 7（已完成）
- 涉及文件: `crates/{agent,provider,channels,tools,skills,cron,session,subagent}/AGENTS.md`
- 验收标准: 每个文件的 `## 架构` 章节使用带方框（`┌─┐└─┘│`）的 ASCII 图展示核心组件关系和数据流，辅以必要的文字说明；图的内容与实际代码一致
- 风险/注意点: ASCII 图需在等宽字体下对齐；保持简洁，不要过度细化
- 步骤:
  - [ ] 为 8 个 crate 分别设计 ASCII 图，替换现有的文字描述
  - [ ] 验证每个文件的 `## 关键类型` 和 `## 内部依赖` 章节未被修改

## 实现建议

- 每个 crate 的 AGENTS.md 统一格式：`# {crate名} crate` 标题 → 一句话职责 → `## 关键类型` 列表 → `## 内部依赖` 列表
- 提取公共类型时，以 `src/lib.rs` 的 `pub use` / `pub mod` 导出为准，按需读取具体源文件确认方法签名
- 任务 2-5 之间无依赖，可并行执行
