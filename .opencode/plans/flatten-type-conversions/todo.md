# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/openai/mod.rs` | 修改 | 提取 From 实现，简化 TryFrom<&Message> |
| `crates/provider/src/anthropic/mod.rs` | 修改 | 提取 From 实现，简化 convert_messages |
| `crates/provider/src/base/mod.rs` | 修改 | 简化 strip_images |

## 任务列表

### A. OpenAI provider 扁平化

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/openai/mod.rs`
- 验收标准: TryFrom<&Message> 的 User 分支缩减为一行；现有测试通过
- 信心评估: 5
- 步骤:
  - [ ] 实现 `From<&ContentPart> for ChatCompletionRequestUserMessageContentPart`
  - [ ] 实现 `From<&UserContent> for ChatCompletionRequestUserMessageContent`
  - [ ] 简化 `TryFrom<&Message>` 的 `User` 分支为 `Ok(Self::User(content.into()))`
  - [ ] `cargo test -p nanobot-provider` 验证

### B. Anthropic provider 扁平化

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/anthropic/mod.rs`
- 验收标准: convert_messages 的 User/Parts 分支简化；现有测试通过
- 信心评估: 5
- 步骤:
  - [ ] 实现 `From<&ContentPart> for ContentBlock`（Anthropic 格式）
  - [ ] 简化 `convert_messages()` 中 `UserContent::Parts` 的 `.map()` 闭包为 `.map(Into::into)`
  - [ ] `cargo test -p nanobot-provider` 验证

### C. strip_images 扁平化

- 优先级: P1
- 依赖项: 无
- 涉及文件: `crates/provider/src/base/mod.rs`
- 验收标准: strip_images 嵌套从 7 层降到 4-5 层；现有测试通过
- 信心评估: 5
- 步骤:
  - [ ] 提取内层 ContentPart 替换逻辑为 Message 上的方法或独立函数
  - [ ] `cargo test -p nanobot-provider` 验证

## 实现建议

- From 实现放在对应 provider 模块中（openai/mod.rs、anthropic/mod.rs），因为目标类型来自外部 crate
- strip_images 的内层逻辑可以提取为 `ContentPart::strip_image() -> ContentPart` 方法或 `UserContent::strip_images() -> UserContent` 方法
