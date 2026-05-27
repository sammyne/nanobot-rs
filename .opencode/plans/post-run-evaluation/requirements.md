# 需求

## 目标与背景

cron 和 heartbeat 后台任务执行后，agent 的输出会直接发送到 channel（钉钉/飞书）。当 agent 的回复是"一切正常，无需操作"之类的例行确认时，用户会收到无意义的通知骚扰。

Python 版本（#59/#60/#61）经历了从 `<SILENT_OK>` 魔法标记到结构化 LLM 评估的演进。最终方案：任务执行后，用一次轻量 LLM tool-call 判断输出是否值得通知用户。

**现状问题**：
- cron 回调（`gateway/mod.rs:389-393`）：`payload.deliver && payload.to.is_some()` 时无条件发送
- heartbeat `tick()`（`heartbeat/service/mod.rs:182-184`）：`on_execute` 返回非空结果后无条件调用 `on_notify`
- 用户在群聊中频繁收到"已检查，一切正常"类消息

## 方案比较（强制）

### 方案 1: LLM tool-call 评估（理想架构）

- 思路: 任务执行后，用独立的 LLM 调用（`evaluate_notification` 工具）判断是否通知。对齐 Python 版本的最终方案。
- 优点:
  - 判断准确，能理解语义（"没有新任务"→ 抑制，"发现异常"→ 通知）
  - fail-open 设计，LLM 出错时默认通知，不会漏发重要消息
  - 评估逻辑与任务 prompt 解耦，不污染会话历史
- 缺点:
  - 每次后台任务额外一次 LLM 调用（`max_tokens=256`，成本低但非零）
  - 增加延迟（约 1-2 秒）
- 工作量估算: M

### 方案 2: 规则/启发式评估（最小可行版）

- 思路: 用简单规则判断：空/过短回复 → 抑制；包含关键词（"error"、"failed"、"完成"）→ 通知；其余 → 通知。
- 优点:
  - 零额外 LLM 成本
  - 无延迟
  - 实现简单
- 缺点:
  - 判断粗糙，容易误判（"我检查了所有任务，一切正常，没有错误" 包含"错误"会误通知）
  - 需要维护关键词列表，多语言场景难覆盖
  - Python 版本已验证此方案不够好（从 `<SILENT_OK>` 演进到 LLM 评估）
- 工作量估算: S

### 推荐

方案 1（LLM tool-call 评估）。Python 版本已验证启发式方案的局限性，LLM 评估的额外成本（每次 256 token）相对于后台任务本身的 LLM 调用可忽略。

## 功能需求列表

### 核心功能

1. **评估函数** `evaluate_response`：接收 agent 响应和任务上下文，通过 LLM tool-call 返回 `should_notify: bool`
2. **heartbeat 集成**：`tick()` 中 `on_execute` 返回后、`on_notify` 调用前插入评估
3. **cron 集成**：cron 回调中 `process_direct` 返回后、`outbound_tx.send()` 前插入评估

### 扩展功能

- 评估结果日志记录（`should_notify` + `reason`），便于调试通知行为

## 非功能需求

- **可靠性**：fail-open — 评估失败（LLM 错误、解析失败、超时）时默认 `should_notify=true`，不漏发重要消息
- **性能**：评估调用使用 `max_tokens=256, temperature=0.0`，最小化延迟和成本
- **可维护性**：评估逻辑集中在一处，cron 和 heartbeat 共用

## 边界与不做事项

- 不做可配置的通知规则引擎
- 不做用户级通知偏好设置
- 不修改 heartbeat 的两阶段决策逻辑（Phase 1 decide + Phase 2 execute）
- 不处理 `enabled_channels` 为空导致 heartbeat 通知被静默丢弃的问题（已有 bug，与本需求无关）

## 假设与约束

- **技术假设**：评估使用与主 agent 相同的 provider/model，通过 clone + bind_tools 获得独立的工具集
- **资源约束**：评估 LLM 调用的 token 消耗极低（系统提示 ~100 token + 输入 + 256 max output）

## 待确认事项

- 无
