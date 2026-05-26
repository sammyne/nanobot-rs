# 需求

## 目标与背景

当前 Rust 版本的上下文管理完全基于消息计数（`memory_window` 默认 100 条），存在以下问题：

1. **无法精确控制输入大小**：不同消息长度差异巨大（一条工具结果可能几千 token，一条用户消息可能几个 token），消息计数无法反映实际 token 消耗
2. **ReAct 循环内无截断**：单轮对话中工具调用迭代时，消息列表无限增长，可能超出模型上下文窗口
3. **系统提示无大小检查**：MEMORY.md + bootstrap files + skills 组装的系统提示大小不受控
4. **整合触发不精确**：基于消息计数的整合阈值无法反映实际上下文压力

对应上游 PR：HKUDS/nanobot#1704（refactor: implement token-based context compression mechanism）。

现状分析（Rust 版本）：
- `Session::get_history(memory_window)` 按消息计数截取最近 N 条
- `save_turn()` 对工具结果截断到 500 字符（仅持久化时）
- 无 token 计数能力
- 整合触发条件：`messages.len() - last_consolidated >= memory_window`

## 方案比较（强制）

### 方案 1: 基于 token 预算的上下文压缩（最小可行版）

- 思路: 引入 token 估算（基于字符数的简单估算，1 token ~= 4 字符），在 `build_messages()` 或 `re_act()` 入口处检查总 token 数，超出预算时从历史消息头部开始丢弃（保持 user-alignment），整合触发也改为 token-based
- 优点:
  - 实现简单，不依赖外部 tokenizer 库
  - 覆盖主要场景：防止超出模型上下文窗口
  - 与上游逻辑对齐
- 缺点:
  - 字符估算不精确（中文/代码/特殊 token 差异大）
  - 不支持按模型选择 tokenizer
- 工作量估算: M

### 方案 2: 精确 token 计数 + 渐进式压缩（理想架构）

- 思路: 引入 `tiktoken-rs` 做精确 token 计数，按模型选择 tokenizer（cl100k_base / o200k_base），实现渐进式压缩（先丢弃旧历史 → 再截断工具结果 → 最后压缩系统提示），支持 `compression_start_ratio` / `compression_target_ratio` 配置
- 优点:
  - 精确的 token 计数
  - 多级压缩策略，最大化保留有用上下文
  - 完全对齐上游行为
- 缺点:
  - `tiktoken-rs` 引入额外依赖和编译时间
  - 实现复杂度高
  - 需要维护模型到 tokenizer 的映射
- 工作量估算: L

### 推荐

方案 1（基于 token 预算的上下文压缩）。字符估算虽不精确，但足以防止上下文溢出。后续可平滑升级到精确 tokenizer。上游 Python 版本也使用 tiktoken 做估算而非精确计数。

## 功能需求列表

### 核心功能

1. **Token 估算工具**：新增 `estimate_tokens(text) -> usize` 函数，基于字符数估算 token 数（1 token ~= 4 字节，中文 1 字符 ~= 2 token）
2. **新增配置项**：在 `AgentDefaults` 中添加 `max_input_tokens: usize`（默认 128000），表示 LLM 输入的 token 预算
3. **历史消息截断改为 token-based**：`Session::get_history()` 增加 token 预算参数，从最新消息向前累加 token，超出预算时停止（保持 user-alignment）
4. **ReAct 循环内 token 检查**：每次调用 `call_llm()` 前检查总 token 数，超出预算时从历史消息头部丢弃（保留系统提示 + 最近的工具调用上下文）
5. **整合触发改为 token-based**：当未整合消息的 token 总量超过 `max_input_tokens * 0.5` 时触发整合

### 扩展功能

- 无

## 非功能需求

- **性能**：token 估算应为 O(n) 字符扫描，不引入外部依赖
- **安全**：无新增安全考量
- **兼容性**：`memory_window` 配置保留但降级为备用限制（token 预算优先）
- **可维护性**：token 估算逻辑集中在 utils crate，便于后续替换为精确 tokenizer
- **测试要求**：token 估算函数测试、历史截断边界测试、ReAct 循环 token 检查测试

## 边界与不做事项

- 不引入 `tiktoken-rs` 或其他外部 tokenizer（本次使用字符估算）
- 不实现渐进式多级压缩（本次只做历史消息丢弃）
- 不修改系统提示的大小控制（保持现状）
- 不修改 `save_turn()` 的工具结果截断逻辑（保持 500 字符）

## 假设与约束

- **技术假设**：1 token ~= 4 字节对英文/代码足够准确；中文场景可后续调优
- **资源约束**：无
- **环境约束**：无

## 待确认事项

- 无
