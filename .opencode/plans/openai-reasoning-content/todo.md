# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `Cargo.toml` (workspace) | 修改 | async-openai 启用 byot feature |
| `crates/provider/src/openai/mod.rs` | 修改 | chat() 改用 create_byot，解析 Value 响应 |
| `crates/provider/src/openai/tests.rs` | 修改 | 新增 reasoning_content 提取测试 |

## 任务列表

### 1. 启用 byot feature + 重构 chat() 方法

- 优先级: P0
- 依赖项: 无
- 涉及文件: `Cargo.toml`, `crates/provider/src/openai/mod.rs`
- 验收标准: chat() 使用 create_byot 获取 Value 响应，正确提取 content/tool_calls/usage/reasoning_content
- 信心评估: 4
- 步骤:
  - [ ] workspace Cargo.toml 中 async-openai 启用 `byot` feature
  - [ ] `OpenAILike::chat()` 中将 `self.client.chat().create(request)` 改为 `self.client.chat().create_byot(request)`，响应类型为 `serde_json::Value`
  - [ ] 实现 `parse_value_response(value: &Value) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>, Option<serde_json::Value>)>` 从 Value 中提取 content、tool_calls、usage、reasoning_content
  - [ ] 有 reasoning_content 时用 `Message::assistant_with_thinking()` 构造消息
  - [ ] `cargo check -p nanobot-provider` 验证
  - [ ] `cargo test -p nanobot-provider` 验证现有测试通过

### 2. 新增 reasoning_content 测试

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/provider/src/openai/tests.rs`
- 验收标准: 测试覆盖有/无 reasoning_content 的响应解析
- 信心评估: 5
- 步骤:
  - [ ] 测试：标准响应（无 reasoning_content）正确解析
  - [ ] 测试：带 reasoning_content 的响应正确提取并存入 thinking 字段
  - [ ] `cargo test -p nanobot-provider` 验证

## 实现建议

- `create_byot()` 接受与 `create()` 相同的请求类型，只是响应类型变为泛型
- 从 Value 提取字段时使用 `value["choices"][0]["message"]["reasoning_content"]`
- reasoning_content 存为 `serde_json::Value::String`，与 Anthropic thinking 的 `Value` 格式一致
