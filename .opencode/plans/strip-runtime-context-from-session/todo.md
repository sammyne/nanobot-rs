# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/session/src/session.rs` | 修改 | 新增 `strip_runtime_context()` 函数，在 `save_turn()` 中调用 |
| `crates/session/tests/session.rs` | 修改 | 新增 `save_turn` 剥离 runtime context 的集成测试 |

## 任务列表

### 1. ✅ 实现 strip_runtime_context 并集成到 save_turn

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/session/src/session.rs`
- 验收标准: `save_turn()` 保存 `Message::User` 时，剥离文本末尾的 `\n\n[Runtime Context]\n...` 块；无 runtime context 的消息不受影响
- 风险/注意点: `[Runtime Context]` 标记字符串需与 `crates/context/src/builder/mod.rs:354` 中的格式保持一致，用注释标注
- 信心评估: 5（`strip_images` 提供了完整的参考模式）
- 步骤:
  - [ ] 在 `session.rs` 的 `strip_images` 函数之后，新增常量 `const RUNTIME_CONTEXT_MARKER: &str = "\n\n[Runtime Context]\n";`，添加注释 `// 须与 crates/context/src/builder/mod.rs 中 inject_runtime_context 的格式保持一致`
  - [ ] 新增 `strip_runtime_context(content: &UserContent) -> UserContent` 函数，处理两种变体：
    - `Text(text)`: 用 `text.find(RUNTIME_CONTEXT_MARKER)` 查找标记位置，找到则截断到该位置，否则原样返回
    - `Parts(parts)`: 检查最后一个 part 是否为 `ContentPart::Text` 且以 `"\n\n[Runtime Context]\n"` 开头，是则移除该 part，否则原样返回
  - [ ] 修改 `save_turn()` 第 127 行，将 `strip_images(content)` 改为 `strip_runtime_context(&strip_images(content))`，先剥离图片再剥离 runtime context
  - [ ] 运行 `cargo clippy -p nanobot-session -- -D warnings -D clippy::uninlined_format_args` 确认无警告

### 2. ✅ 添加测试

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/session/tests/session.rs`
- 验收标准: 测试覆盖 save_turn 对含 runtime context 的用户消息的剥离行为，以及无 runtime context 时的 no-op 行为
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 新增测试 `save_turn_strips_runtime_context`：构造含 `\n\n[Runtime Context]\nCurrent Time: 2026-05-24 10:00 (Saturday) (+08:00)` 后缀的用户消息，调用 `save_turn()`，断言保存后的消息不含 `[Runtime Context]`，仅保留原始用户文本
  - [ ] 新增测试 `save_turn_preserves_message_without_runtime_context`：构造不含 runtime context 的普通用户消息，调用 `save_turn()`，断言消息内容不变
  - [ ] 新增测试 `save_turn_strips_runtime_context_with_channel_info`：构造含 channel 和 chat_id 信息的 runtime context（`Current Time: ...\nChannel: feishu\nChat ID: oc_xxx`），断言全部剥离
  - [ ] 运行 `cargo test -p nanobot-session` 确认全部通过

## 实现建议

- `strip_runtime_context` 与 `strip_images` 并列放置，保持代码组织一致
- 调用顺序：先 `strip_images`（可能将 Parts 合并为 Text），再 `strip_runtime_context`（处理合并后的 Text）
- `RUNTIME_CONTEXT_MARKER` 使用 `\n\n[Runtime Context]\n` 而非 `[Runtime Context]`，避免误匹配用户消息中碰巧包含该文本的情况
