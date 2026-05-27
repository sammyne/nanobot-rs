# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/base/mod.rs` | 修改 | `ProviderError::is_image_unsupported()` 方法 + `strip_images()` 函数 |
| `crates/provider/src/base/tests.rs` | 修改 | `is_image_unsupported` 和 `strip_images` 单元测试 |
| `crates/provider/src/auto_retry/mod.rs` | 修改 | `chat()` 中添加图片拒绝检测和重试分支 |
| `crates/provider/src/auto_retry/tests.rs` | 修改 | 图片拒绝重试测试 |

## 任务列表

### 1. ProviderError 图片拒绝检测 + strip_images 函数

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/base/mod.rs`, `crates/provider/src/base/tests.rs`
- 验收标准: `ProviderError::Api("image_url is only supported...")` 的 `is_image_unsupported()` 返回 `true`；`strip_images` 正确替换 Image 为 Text 占位符；无图片时返回 `None`
- 信心评估: 5
- 步骤:
  - [ ] 在 `ProviderError` impl 块中新增 `pub fn is_image_unsupported(&self) -> bool`，仅匹配 `Api(msg)` 变体，对 msg 做小写化后检查是否包含以下任一关键词：`"image_url is only supported"`, `"does not support image"`, `"images are not supported"`, `"image input is not supported"`, `"image_url is not supported"`, `"unsupported image input"`
  - [ ] 新增 `pub fn strip_images(messages: &[Message]) -> Option<Vec<Message>>`：遍历消息，对 `Message::User { content: UserContent::Parts(parts) }` 中的 `ContentPart::Image` 替换为 `ContentPart::Text { text: "[image omitted]".to_string() }`；若未发现任何图片返回 `None`
  - [ ] 在 `crates/provider/src/base/tests.rs` 中新增测试：`is_image_unsupported` 对各关键词返回 true、对非图片错误返回 false；`strip_images` 替换图片、无图片返回 None、纯文本消息不变
  - [ ] `cargo test -p nanobot-provider -- image` 确认通过

### 2. AutoRetryProvider 添加图片拒绝重试

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/auto_retry/mod.rs`, `crates/provider/src/auto_retry/tests.rs`
- 验收标准: 图片拒绝错误时自动 strip images 并重试一次；重试成功则返回成功；重试失败则返回错误；无图片可 strip 时直接返回原始错误
- 风险/注意点: `chat()` 接收 `&[Message]`，strip 后需要用新 Vec 调用 `self.inner.chat(&stripped, options)`
- 信心评估: 5
- 步骤:
  - [ ] 在 `chat()` 方法中，现有 `return Err(e)` 分支（非瞬态错误直接返回）之前，插入图片拒绝检测：若 `pe.is_image_unsupported()`，调用 `strip_images(messages)`，若返回 `Some(stripped)`，warn 日志后调用 `self.inner.chat(&stripped, options).await` 并返回结果；若返回 `None`（无图片可 strip），返回原始错误
  - [ ] 在 `tests.rs` 中新增 4 个测试：
    - `image_unsupported_retries_without_images`：首次返回图片拒绝错误 + 消息含图片 → 重试成功，共 2 次调用
    - `image_unsupported_no_images_to_strip`：首次返回图片拒绝错误 + 消息无图片 → 直接返回错误，共 1 次调用
    - `image_unsupported_retry_also_fails`：首次图片拒绝 + 重试也失败 → 返回重试的错误，共 2 次调用
    - `non_image_error_no_image_retry`：非图片错误 → 不触发图片重试，共 1 次调用
  - [ ] `cargo test -p nanobot-provider` 确认全部通过

## 实现建议

- `strip_images` 放在 `base/mod.rs` 中作为公共函数（session 的 `strip_images` 也有类似逻辑可参考，但那个是替换为 `"[image]"` 文本，这里替换为 `"[image omitted]"`）
- 图片拒绝重试不进入指数退避循环，是瞬态重试循环之外的独立一次性重试
- 测试中的 MockProvider 需要能接收并检查 messages 参数，以验证重试时图片已被 strip
