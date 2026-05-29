# 需求

## 目标与背景

将上游 PR #2717 的两阶段记忆系统迁移到 Rust 版。当前 Rust 版使用单阶段 LLM 驱动整合（`tool_choice=Required` 调用 `save_memory` 工具），存在以下问题：

1. **LLM 耦合**：MemoryStore 直接调用 `provider.chat()`，不是纯 I/O 层
2. **provider 兼容性**：`tool_choice=Required` 部分 provider 不支持
3. **质量退化**：每次整合需要 LLM 输出完整 MEMORY.md，随着记忆增长质量下降
4. **无版本控制**：记忆文件无历史记录，误修改不可恢复

上游方案：Consolidator（每轮纯文本摘要）+ Dream（cron 定时 LLM 驱动增量编辑 + git 版本控制）。

## 方案比较（强制）

### 方案 1: 分阶段逐步迁移 ✅ 已选定

- 思路: 按依赖关系分阶段实现，每阶段可独立合入
- 优点: 风险可控，每步可验证
- 缺点: 需多轮迭代
- 工作量估算: L

### 方案 2: 一次性全量迁移

- 思路: 一个大 PR
- 优点: 一步到位
- 缺点: 改动巨大（跨 6 个 crate），review 困难
- 工作量估算: XL

### 推荐

方案 1。

## 功能需求列表

### 前置: 调研文档

- 编写 `docs/dreaming.md`，覆盖设计理念、架构、文件布局、两阶段流程、GitStore、命令、配置
- 参考上游 `docs/MEMORY.md`，适配 Rust 版实现差异（如 GitStore 使用 git CLI 而非 dulwich）

### 阶段 1: DreamConfig（极小）

- `AgentDefaults` 新增 `dream: Option<DreamConfig>` 字段
- `DreamConfig` 结构体：`cron`（默认 `"0 */2 * * *"`）、`model`（可选覆盖）、`max_batch_size`（默认 20）、`max_iterations`（默认 10）

### 阶段 2: MemoryStore 重写 + Consolidator 简化

- MemoryStore 解耦 LLM：移除 `provider`/`options` 字段，变为纯 I/O 层
- `HISTORY.md` → `history.jsonl`：结构化条目 + 自增 cursor
- Consolidator 简化：移除 `save_memory` 工具和 `tool_choice=Required`，改为纯文本 LLM 摘要
- 追加 cursor 字段到 MemoryStore，防止重复处理
- 自动 compaction（超过 1000 条时截断旧条目）

### 阶段 3: GitStore

- 新增 git 版本控制模块（通过 `std::process::Command` 调用 git CLI）
- 支持 init、add、commit、log、diff、revert 操作
- 自动生成 `.gitignore` 只跟踪记忆文件
- Dream 运行后自动 commit
- git 不可用时必须报错，禁止静默处理

### 阶段 4: Dream + 命令

- Dream 两阶段处理器：
  - Phase 1：纯 LLM 调用，对比 history.jsonl 和记忆文件，输出 `[FILE] atomic fact` 行
  - Phase 2：使用 AgentRunner + read_file/edit_file 工具进行增量编辑
- CronService 新增 `register_system_job()` 支持内部系统任务
- 新增 `/dream`、`/dream-log`、`/dream-restore` 命令

## 非功能需求

- 向后兼容：已有 HISTORY.md 内容不丢失（迁移或保留）
- 现有测试通过
- 每阶段独立可编译和测试

## 边界与不做事项

- 不实现 SOUL.md / USER.md 自动管理（保持手动编辑）

## 待确认事项

无
