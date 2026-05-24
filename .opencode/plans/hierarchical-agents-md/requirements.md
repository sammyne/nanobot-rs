# 需求

## 目标与背景

使用 opencode 开发 nanobot-rs 时，每次重启 opencode 后 agent 对项目一无所知，需要通过大量 `glob`/`grep`/`read` 重新探索代码库才能理解架构，浪费时间。

根本原因：当前根 `AGENTS.md` 只有 crate 列表和编码规范，缺少 agent 开发时最需要的信息——各 crate 的内部文件结构、关键类型、公共 API、跨 crate 依赖关系。agent 每次都要自己扫描推断这些信息。

目标：建立层级式 `AGENTS.md` 体系，把项目架构信息预写好，让 agent 启动时直接读取即可工作，无需重复扫描。

## 方案比较

### 方案 1: 单文件扩充

- 思路：把所有 crate 的详细信息全部写入根 `AGENTS.md`
- 优点：一个文件搞定，简单
- 缺点：文件会膨胀到数百行，大量信息在 agent 只操作单个 crate 时是噪音，浪费 token

### 方案 2: 层级式 AGENTS.md

- 思路：根 `AGENTS.md` 保留全局概览（架构、依赖图、编码规范），每个 `crates/*/AGENTS.md` 放该 crate 的关键类型和公共 API
- 优点：opencode 会自动加载工作目录及祖先目录的 `AGENTS.md`，agent 在某个 crate 下工作时自动获得全局 + 局部上下文；信息按需加载，不浪费 token
- 缺点：需要维护多个文件，代码变更时需同步更新对应 crate 的 AGENTS.md

### 推荐

推荐**方案 2（层级式 AGENTS.md）**。信息分层存放，agent 按需获取，既减少启动扫描时间又不浪费 token。

## 功能需求列表

### 核心功能

1. **根 AGENTS.md 补充依赖图**：在现有内容基础上，增加 "Crate 依赖关系" 章节，展示 crate 间的依赖关系和统计信息；增加一行说明"每个 crate 目录下有独立的 AGENTS.md"
2. **16 个 crate 级 AGENTS.md**：为 `crates/` 下每个 crate 创建 `AGENTS.md`，包含：
   - 一句话 crate 职责描述
   - 关键公共类型及其核心方法签名
   - 该 crate 的内部依赖列表
3. **复杂 crate 补充架构描述**：对有内部架构的 crate，在 AGENTS.md 中增加 `## 架构` 章节，用带方框的 ASCII 图描述核心组件关系和数据流，辅以必要的文字说明。仅对以下 crate 添加：
   - `agent` — ReAct 循环流程、命令分发机制
   - `provider` — Provider trait 抽象 + OpenAI/Anthropic 双实现
   - `channels` — Channel trait + ChannelManager + 钉钉/飞书实现
   - `tools` — Tool trait + ToolRegistry 分发模式
   - `skills` — 双目录扫描（workspace + builtin）+ 依赖检查
   - `cron` — CronService/CronStorage/Scheduler 分层
   - `session` — JSONL 持久化 + 内存缓存
   - `subagent` — SubagentManager 任务生命周期管理
   - 简单 crate（utils, templates, mcp, config, memory, context, heartbeat）不需要架构描述

### 扩展功能

- 无

## 非功能需求

- **准确性**：AGENTS.md 中的类型名、方法签名必须与实际代码一致
- **简洁性**：每个 crate 的 AGENTS.md 只记录关键公共类型和 API，不列源文件清单（agent 可通过一次 `read` 目录获取），不重复根 AGENTS.md 中的编码规范
- **可维护性**：后续代码变更时，对应 crate 的 AGENTS.md 应同步更新

## 边界与不做事项

- 不修改任何 Rust 源代码
- 不修改根 AGENTS.md 中已有的编码规范、测试实践等章节（仅新增内容）
- 不为 tests.rs 文件编写文档
- 不记录私有类型和内部实现细节，只记录公共 API

## 假设与约束

- **技术假设**：opencode 会自动加载工作目录及其祖先目录中的 `AGENTS.md` 文件
- **当前状态**：根 `AGENTS.md` 已完成更新；16 个 crate 级 AGENTS.md 已创建（含职责、关键类型、内部依赖）；8 个复杂 crate 已有文字版架构描述，需替换为带方框的 ASCII 图版本

## 待确认事项

- 无
