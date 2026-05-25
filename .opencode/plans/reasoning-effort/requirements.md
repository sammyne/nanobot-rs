# 需求

## 目标与背景

用户希望通过 `config.json` 的 `reasoningEffort` 字段启用 o-series（o1/o3/o4-mini）、DeepSeek-R1 等模型的思维模式。Python 版 PR #1351 已实现此功能。

当前 Rust 版存在两个问题：
1. **缺少 `reasoning_effort` 配置项**：无法启用思维模式
2. **pre-existing bug**：`AgentLoop::call_llm()` 和 `SubagentManager::run_subagent()` 使用 `Options::default()` 硬编码，config 中的 `max_tokens` 和 `temperature` 从未传递到 provider。添加 `reasoning_effort` 时必须一并修复此问题，否则新字段也不会生效

## 方案比较（强制）

### 方案 1: 仅添加 reasoning_effort，不修复 Options 传播

- 思路: 在 config 和 Options 中添加字段，但保持 `Options::default()` 调用不变，仅在 provider 层读取
- 优点: 改动最小
- 缺点: **不可行** — `Options::default()` 的 `reasoning_effort` 为 `None`，config 值永远不会到达 provider
- 工作量估算: S

### 方案 2: 添加 reasoning_effort + 修复 Options 传播（推荐）

- 思路: 在 config 和 Options 中添加字段，同时修复 `call_llm()` 和 `run_subagent()` 从 config 构造 Options。heartbeat 和 memory 保持 `Options::default()`（它们不需要 reasoning_effort，且有独立的用途）
- 优点: reasoning_effort 能正确生效，顺带修复 max_tokens/temperature 传播
- 缺点: 改动涉及多个 crate
- 工作量估算: S-M

### 推荐

方案 2。修复 Options 传播是 reasoning_effort 生效的前提条件，不是可选项。

## 功能需求列表

### 核心功能

1. **Config 层**：`AgentDefaults` 新增 `reasoning_effort: Option<ReasoningEffort>` 字段，JSON key 为 `reasoningEffort`，默认 `None`。`ReasoningEffort` 枚举定义在 config crate（`Copy + Clone + Serialize + Deserialize`，`#[serde(rename_all = "lowercase")]`），provider crate 通过已有的 config 依赖使用
2. **Options 层**：`Options` 新增 `reasoning_effort: Option<ReasoningEffort>` 字段，默认 `None`，保持 `Copy` trait
3. **Options 构造修复**：
   - `AgentLoop::call_llm()` 从 `self.config` 构造 `Options`（max_tokens、temperature、reasoning_effort）
   - `SubagentManager::run_subagent()` 从 `self.temperature`、`self.max_tokens` 构造 `Options`（reasoning_effort 为 `None`，子代理不需要）
4. **OpenAI provider**：`chat()` 中当 `options.reasoning_effort` 为 `Some` 时，映射为 `async_openai::types::ReasoningEffort` 并设置到 builder
5. **Anthropic provider**：忽略 `reasoning_effort`（Anthropic 使用独立的 `thinking` 块机制，不通过此参数控制）

### 扩展功能

- 无

## 非功能需求

- **兼容性**：`reasoning_effort` 默认 `None`，不影响现有行为；不支持的模型会由 provider API 自行忽略或报错
- **可维护性**：`ReasoningEffort` 枚举为 `Copy`，`Options` 保持 `Copy` trait

## 边界与不做事项

- 不修改 heartbeat 和 memory 的 `Options::default()` 调用（它们有独立用途，不需要 reasoning_effort）
- 不为 Anthropic 实现 thinking budget 映射（Anthropic 的 extended thinking 是独立功能）
- 不做 reasoning_effort 值的校验（无效值在 config 反序列化时由 serde 报错）

## 假设与约束

- **技术假设**：`async-openai` 0.28 的 `CreateChatCompletionRequestArgs` builder 自动生成 `.reasoning_effort()` 方法（已验证）
- **资源约束**：无需新增外部依赖

## 设计决策：ReasoningEffort 枚举

`ReasoningEffort` 定义在 config crate 中（与 `AgentDefaults` 同处），provider crate 通过已有的 config 依赖使用。枚举定义：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}
```

- `Copy + Clone`：保持 `Options` 的 `Copy` 语义
- `serde(rename_all = "lowercase")`：config JSON 中写 `"low"` / `"medium"` / `"high"`，serde 自动解析
- 定义在 config crate 而非 provider crate：因为 provider 依赖 config（反向不成立），且 `ReasoningEffort` 本质是配置值

## 待确认事项

- 无
