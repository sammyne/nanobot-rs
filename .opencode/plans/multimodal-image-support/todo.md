# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/base/mod.rs` | 修改 | `UserContent`、`ContentPart` 类型定义；`Message::User` content 改为 `UserContent` |
| `crates/provider/src/base/tests.rs` | 修改 | UserContent serde 测试 |
| `crates/provider/src/anthropic/mod.rs` | 修改 | `ContentBlock::Image` 变体；`convert_messages()` 多模态转换 |
| `crates/provider/src/anthropic/tests.rs` | 修改 | 多模态消息转换测试 |
| `crates/provider/src/openai/mod.rs` | 修改 | `TryFrom<&Message>` 多模态分支 |
| `crates/provider/src/openai/tests.rs` | 修改 | 多模态消息转换测试 |
| `crates/context/src/builder/mod.rs` | 修改 | `build_user_content()` 返回 `UserContent`；`encode_image_to_base64()` 返回拆分字段 |
| `crates/context/src/builder/tests.rs` | 修改 | 多模态 build_user_content 测试 |
| `crates/session/src/session.rs` | 修改 | `save_turn()` 剥离图片 base64 |
| `crates/agent/src/loop/mod.rs` | 修改 | `process_message()` 传递 media；`process_direct()` 新增 media 参数 |
| `crates/nanobot/src/commands/agent/mod.rs` | 修改 | `AgentCmd` 新增 `--image` 参数 |
| `crates/channels/src/feishu/mod.rs` | 修改 | 接收图片消息，下载图片，填充 media |
| `crates/channels/src/dingtalk/mod.rs` | 修改 | 接收图片消息，下载图片，填充 media |

## 任务列表

### 1. Message 类型改造

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/base/mod.rs`、`crates/provider/src/base/tests.rs`
- 验收标准: `Message::User` 使用 `UserContent` 枚举；`Message::user()` 保持 `impl Into<String>` 签名；`Message::content()` 对 `Parts` 返回拼接的文本；旧 JSONL 格式能正常反序列化
- 步骤:
  - [ ] 在 `base/mod.rs` 中新增 `UserContent` 枚举（`#[serde(untagged)]`）：`Text(String)` 和 `Parts(Vec<ContentPart>)`
  - [ ] 新增 `ContentPart` 枚举（`#[serde(tag = "type", rename_all = "snake_case")]`）：`Text { text: String }` 和 `Image { media_type: String, data: String }`
  - [ ] 为 `UserContent` 实现 `From<String>` 和 `From<&str>`（包装为 `Text`）
  - [ ] 修改 `Message::User` 的 `content` 字段类型从 `String` 改为 `UserContent`
  - [ ] `Message::user()` 内部改为 `UserContent::Text(content.into())`
  - [ ] 新增 `Message::user_with_parts(parts: Vec<ContentPart>)` 构造函数
  - [ ] 修改 `Message::content()` 方法：`UserContent::Text(s)` 返回 `s`；`UserContent::Parts(parts)` 拼接所有 `ContentPart::Text` 的 text 字段
  - [ ] 在 `tests.rs` 中新增 serde 测试：纯文本序列化/反序列化、多模态序列化/反序列化、旧 JSONL 格式（`"content": "hello"`）反序列化为 `UserContent::Text`
  - [ ] 运行 `cargo clippy -p nanobot-provider -- -D warnings -D clippy::uninlined_format_args`

### 2. Anthropic provider 适配

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/anthropic/mod.rs`、`crates/provider/src/anthropic/tests.rs`
- 验收标准: `convert_messages()` 对 `UserContent::Parts` 生成 `[ContentBlock::Text, ContentBlock::Image, ...]`；图片使用 `{"type": "image", "source": {"type": "base64", "media_type": "...", "data": "..."}}` 格式
- 步骤:
  - [ ] 新增 `ContentBlock::Image { source: ImageSource }` 变体
  - [ ] 新增 `ImageSource` 结构体：`r#type: String`（固定 `"base64"`）、`media_type: String`、`data: String`
  - [ ] 修改 `convert_messages()` 中 `Message::User` 分支：匹配 `UserContent::Text` 保持现有逻辑；匹配 `UserContent::Parts` 逐项转换为 `ContentBlock::Text` 或 `ContentBlock::Image`
  - [ ] 在 `tests.rs` 中新增测试：包含图片的用户消息正确转换为 Anthropic 格式
  - [ ] 运行 `cargo clippy -p nanobot-provider -- -D warnings -D clippy::uninlined_format_args`

### 3. OpenAI provider 适配

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/openai/mod.rs`、`crates/provider/src/openai/tests.rs`
- 验收标准: `TryFrom<&Message>` 对 `UserContent::Parts` 生成 `ChatCompletionRequestUserMessageContent::Array([...])`；图片使用 `data:{media_type};base64,{data}` 格式的 image_url
- 步骤:
  - [ ] 修改 `TryFrom<&Message>` 中 `Message::User` 分支：匹配 `UserContent::Text` 保持现有逻辑；匹配 `UserContent::Parts` 构建 `ChatCompletionRequestUserMessageContent::Array(parts)`，其中 `ContentPart::Text` → `TextPart`，`ContentPart::Image` → `ImageUrlPart { url: format!("data:{media_type};base64,{data}") }`
  - [ ] 在 `tests.rs` 中新增测试：包含图片的用户消息正确转换为 OpenAI 格式
  - [ ] 运行 `cargo clippy -p nanobot-provider -- -D warnings -D clippy::uninlined_format_args`

### 4. Context builder 适配

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/context/src/builder/mod.rs`、`crates/context/src/builder/tests.rs`
- 验收标准: `build_user_content()` 有图片时返回 `UserContent::Parts`；无图片时返回 `UserContent::Text`；`encode_image_to_base64()` 返回 `(media_type, data)` 元组
- 步骤:
  - [ ] 修改 `encode_image_to_base64()` 返回类型从 `Result<Option<String>>` 改为 `Result<Option<(String, String)>>`（`(media_type, base64_data)`），不再拼接 `data:...;base64,...` 前缀
  - [ ] 修改 `build_user_content()` 返回类型从 `Result<String>` 改为 `Result<UserContent>`：无图片时返回 `UserContent::Text(text)`；有图片时构建 `UserContent::Parts(vec![Text { text }, Image { media_type, data }, ...])`
  - [ ] 修改 `build_messages()` 中调用 `build_user_content()` 的地方：将返回的 `UserContent` 传给 `Message::User` 构造（需要新增接受 `UserContent` 的构造方式或直接构造枚举变体）
  - [ ] 更新 `tests.rs` 中相关测试
  - [ ] 运行 `cargo clippy -p nanobot-context -- -D warnings -D clippy::uninlined_format_args`

### 5. Session 持久化去 base64

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/session/src/session.rs`
- 验收标准: `save_turn()` 将 `UserContent::Parts` 中的 `ContentPart::Image` 替换为 `ContentPart::Text { text: "[image]" }`；全部为 Text 时合并为 `UserContent::Text`
- 步骤:
  - [ ] 在 `save_turn()` 中新增对 `Message::User` 的处理：如果 content 是 `UserContent::Parts`，遍历 parts，将 `Image` 替换为 `Text { text: "[image]".to_string() }`；如果替换后全部是 Text，合并为 `UserContent::Text`
  - [ ] 新增单元测试：验证图片被剥离、纯文本消息不受影响
  - [ ] 运行 `cargo test -p nanobot-session`

### 6. Agent loop 传递 media

- 优先级: P0
- 依赖项: 4
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `process_message()` 将 `InboundMessage.media` 转为 `Vec<PathBuf>` 传给 `build_messages()`；`process_direct()` 新增 `media` 参数
- 步骤:
  - [ ] 修改 `process_message()` 中 `InboundMessage` 解构：不再忽略 `media` 字段
  - [ ] 将 `media` 转为 `Vec<PathBuf>`，传给 `build_messages()` 的 media 参数（`Some(&media_paths)` 或 `None`）
  - [ ] 修改 `process_direct()` 签名：新增 `media: Option<&[PathBuf]>` 参数，传递给 `InboundMessage` 或直接传给 `build_messages()`
  - [ ] 同步修改 `process_system_message()` 中的 `build_messages()` 调用（系统消息不携带图片，保持 `None`）
  - [ ] 更新所有 `process_direct()` 调用方（nanobot crate、cron 回调、heartbeat 回调等）
  - [ ] 运行 `cargo clippy -p nanobot-agent -- -D warnings -D clippy::uninlined_format_args`

### 7. CLI 图片输入

- 优先级: P0
- 依赖项: 6
- 涉及文件: `crates/nanobot/src/commands/agent/mod.rs`
- 验收标准: `nanobot agent -m "描述图片" -i photo.png` 能将图片发送给 LLM
- 步骤:
  - [ ] `AgentCmd` 新增 `image: Option<Vec<String>>` 字段，`#[arg(short = 'i', long = "image")]`
  - [ ] `run_once()` 中将 `self.image` 转为 `Vec<PathBuf>`，传给 `process_direct()` 的 media 参数
  - [ ] 交互模式暂不支持图片输入（需要更复杂的输入解析）
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args`

### 8. 飞书 channel 图片接收

- 优先级: P1
- 依赖项: 6
- 涉及文件: `crates/channels/src/feishu/mod.rs`
- 验收标准: 飞书用户发送图片消息时，channel 下载图片并填充 `InboundMessage.media`
- 步骤:
  - [ ] 移除 `message_type != "text"` 的拒绝逻辑，改为支持 `"text"` 和 `"image"` 类型
  - [ ] 对 `message_type == "image"` 的消息：从 content JSON 中提取 `image_key`，调用 `client.im_v1_image().download_image(image_key)` 下载图片
  - [ ] 将下载的图片字节写入临时文件（workspace 下的 `media/` 目录），将路径添加到 `InboundMessage.media`
  - [ ] 对 `message_type == "text"` 保持现有逻辑
  - [ ] 新增单元测试（mock 图片下载）

### 9. 钉钉 channel 图片接收

- 优先级: P1
- 依赖项: 6
- 涉及文件: `crates/channels/src/dingtalk/mod.rs`
- 验收标准: 钉钉用户发送图片消息时，channel 下载图片并填充 `InboundMessage.media`
- 步骤:
  - [ ] 修改 `process_message()` 检测 `msgtype == "picture"` 的消息
  - [ ] 调用 `msg.get_image_list()` 获取 download_code 列表，通过 `replier.get_image_download_url(code)` 获取下载 URL，下载图片字节
  - [ ] 将下载的图片字节写入临时文件，将路径添加到 `InboundMessage.media`
  - [ ] 对文本消息保持现有逻辑
  - [ ] 新增单元测试（mock 图片下载）

### 10. 全量验证

- 优先级: P1
- 依赖项: 1-9
- 涉及文件: 无
- 验收标准: 所有检查通过，手动测试 `nanobot agent -m "描述图片" -i test.png` 能正确发送图片给 LLM
- 步骤:
  - [ ] `cargo +nightly fmt`
  - [ ] `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] `cargo test`
  - [ ] `cargo doc --no-deps`
  - [ ] 手动测试：`nanobot agent -m "这张图片里有什么？" -i /path/to/image.png`

## 实现建议

- 任务 1 是基础，所有其他任务都依赖它。建议先完成任务 1 并确保编译通过后再并行推进 2-5
- `UserContent` 的 `#[serde(untagged)]` 要求 `Text` 变体放在 `Parts` 之前，否则纯字符串会被尝试反序列化为 `Vec<ContentPart>` 而失败
- `Message::content()` 返回 `&str` 的签名需要改变——`UserContent::Parts` 需要拼接文本，无法返回引用。可改为返回 `Cow<'_, str>` 或 `String`
- 飞书和钉钉的图片下载需要异步 HTTP 调用，临时文件建议存放在 workspace 的 `media/` 子目录下，使用 UUID 命名避免冲突
