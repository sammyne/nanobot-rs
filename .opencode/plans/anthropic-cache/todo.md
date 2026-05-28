# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/anthropic/mod.rs` | 修改 | ContentBlock 加 cache_control；system 改为 block 数组；注入断点逻辑 |
| `crates/provider/src/anthropic/tests.rs` | 修改 | 新增 cache_control 注入测试 |

## 任务列表

### 1. ContentBlock 和 AnthropicRequest 结构调整

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/anthropic/mod.rs`
- 验收标准: `cargo check -p nanobot-provider` 通过；序列化输出包含正确的 `cache_control` 字段
- 风险/注意点: `ContentBlock` 使用 `#[serde(tag = "type")]`，新增字段需要 `skip_serializing_if` 避免无 cache_control 时输出 null；`system` 改为 block 数组后序列化格式变化，需确认 Anthropic API 接受数组格式
- 信心评估: 4
- 步骤:
  - [ ] 新增 `CacheControl` 结构体：`#[derive(Debug, Clone, Serialize, Deserialize)] struct CacheControl { r#type: String }`，提供 `fn ephemeral() -> Self`
  - [ ] `ContentBlock::Text` 变体新增 `#[serde(skip_serializing_if = "Option::is_none")] cache_control: Option<CacheControl>` 字段
  - [ ] 新增 `SystemBlock` 结构体：`{ r#type: String, text: String, #[serde(skip_serializing_if = "Option::is_none")] cache_control: Option<CacheControl> }`
  - [ ] `AnthropicRequest.system` 从 `Option<String>` 改为 `#[serde(skip_serializing_if = "Option::is_none")] Option<Vec<SystemBlock>>`
  - [ ] `convert_messages()` 返回值中 system 从 `Option<String>` 改为 `Option<Vec<SystemBlock>>`，将系统提示词包装为 `SystemBlock`
  - [ ] 运行 `cargo check -p nanobot-provider` 验证通过

### 2. 注入 cache_control 断点

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/anthropic/mod.rs`
- 验收标准: `cargo test -p nanobot-provider` 通过；系统消息和倒数第二条消息的最后一个 content block 带有 `cache_control`
- 风险/注意点: 断点 2 仅在 `messages.len() >= 3` 时添加；需要对 `Vec<AnthropicMessage>` 的倒数第二个元素的最后一个 `ContentBlock` 注入 cache_control
- 信心评估: 5
- 步骤:
  - [ ] 在 `chat()` 方法中，`convert_messages()` 返回后、构建 `AnthropicRequest` 前，注入断点：
    - 断点 1：对 `system` 的最后一个 `SystemBlock` 设置 `cache_control = Some(CacheControl::ephemeral())`
    - 断点 2：若 `anthropic_messages.len() >= 3`，对 `anthropic_messages[len-2]` 的最后一个 `ContentBlock`（若为 `Text`）设置 `cache_control = Some(CacheControl::ephemeral())`
  - [ ] 在 `tests.rs` 中新增测试：验证系统消息带 cache_control；验证 >= 3 条消息时倒数第二条带 cache_control；验证 < 3 条消息时不添加第二断点
  - [ ] 运行 `cargo test -p nanobot-provider` 验证通过
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 验证通过

## 实现建议

- `CacheControl::ephemeral()` 返回 `CacheControl { r#type: "ephemeral".to_string() }`
- 断点 2 的注入位置：`anthropic_messages` 的倒数第二个元素（`len - 2`）的 `content` 向量的最后一个元素。只对 `ContentBlock::Text` 注入（ToolUse/ToolResult/Thinking 不需要缓存标记）
- `SystemBlock` 不复用 `ContentBlock` 是因为 system block 的 type 固定为 "text"，且不需要 ToolUse 等变体
