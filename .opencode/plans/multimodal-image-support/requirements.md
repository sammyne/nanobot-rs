# 需求

## 目标与背景

nanobot-rs 当前不支持向 LLM 发送图片。base64 编码基础设施已就绪（`encode_image_to_base64()` 可用），但输出被丢弃，LLM 只看到 `[Image attached: path]` 文本占位符。`Message::User` 的 `content` 字段是纯 `String`，无法携带结构化的多模态内容。

需要打通从消息接收到 LLM 调用的完整图片管道，并在 session 持久化时剥离 base64 数据防止文件膨胀（对应 Python 版 PR #1191）。

## 方案比较

### 方案 1: 在 Message::User 中新增 images 字段

- 思路: `Message::User { content: String, images: Vec<ImageData> }`，`ImageData` 包含 `media_type` 和 `data`（裸 base64）。provider 转换时检查 images 是否非空，生成对应的多模态请求
- 优点: 改动最小，不改变 content 字段的类型，向后兼容现有 serde 格式（images 默认空 Vec，skip_serializing_if）
- 缺点: 图片和文本是分离的，不能表达"文本-图片-文本"交错排列；与 OpenAI/Anthropic API 的 content 模型不对齐

### 方案 2: 将 content 改为结构化枚举

- 思路: `content: UserContent`，其中 `UserContent` 为 `Text(String)` 或 `Parts(Vec<ContentPart>)`，`ContentPart` 为 `Text { text }` 或 `Image { media_type, data }`
- 优点: 与 OpenAI/Anthropic API 的 content 模型完全对齐，支持任意交错排列；未来扩展其他媒体类型只需新增 `ContentPart` 变体
- 缺点: 所有读取 `content` 的代码都需要适配；需要通过 `#[serde(untagged)]` 保持旧 JSONL 兼容

### 推荐

推荐方案 2。与两个 LLM API 的 content 模型对齐，`#[serde(untagged)]` 可保持旧 JSONL 向后兼容。

## 方案 2 详细设计

### 核心类型设计

```rust
/// 用户消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserContent {
    /// 纯文本（向后兼容旧 JSONL）
    Text(String),
    /// 多模态内容（文本 + 图片混合）
    Parts(Vec<ContentPart>),
}

/// 内容片段
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// 文本片段
    Text { text: String },
    /// 图片片段
    Image {
        /// MIME 类型，如 "image/png"
        media_type: String,
        /// 裸 base64 编码数据（不含 data:...;base64, 前缀）
        data: String,
    },
}
```

### serde 向后兼容

`UserContent` 使用 `#[serde(untagged)]`：
- 旧 JSONL 中 `"content": "hello"` → 反序列化为 `UserContent::Text("hello")`
- 新 JSONL 中 `"content": [{"type": "text", "text": "hello"}, {"type": "image", ...}]` → 反序列化为 `UserContent::Parts([...])`

### Provider 转换

**OpenAI**：
- `UserContent::Text(s)` → `ChatCompletionRequestUserMessageContent::Text(s)`
- `UserContent::Parts(parts)` → `ChatCompletionRequestUserMessageContent::Array([...])`
  - `ContentPart::Text { text }` → `TextPart(text)`
  - `ContentPart::Image { media_type, data }` → `ImageUrlPart { url: "data:{media_type};base64,{data}" }`

**Anthropic**：
- `UserContent::Text(s)` → `[ContentBlock::Text { text: s }]`
- `UserContent::Parts(parts)` → 逐项转换
  - `ContentPart::Text { text }` → `ContentBlock::Text { text }`
  - `ContentPart::Image { media_type, data }` → `ContentBlock::Image { source: { type: "base64", media_type, data } }`

### ImageData 字段设计

存储拆分后的 `media_type` + 裸 `data`，而非完整的 data URL：
- Anthropic 直接使用 `media_type` 和 `data`
- OpenAI 拼接为 `format!("data:{media_type};base64,{data}")`

## 功能需求列表

### 核心功能

1. **Message 类型改造**（provider crate）：
   - `Message::User` 的 `content` 从 `String` 改为 `UserContent`
   - 新增 `UserContent`、`ContentPart` 类型定义
   - `Message::user()` 构造函数保持 `impl Into<String>` 签名，内部包装为 `UserContent::Text`
   - 新增 `Message::user_with_parts(parts: Vec<ContentPart>)` 构造函数
   - `Message::content()` 返回文本内容（`UserContent::Text` 直接返回；`Parts` 拼接所有 Text 片段）
2. **Anthropic provider 适配**（provider crate）：
   - `ContentBlock` 新增 `Image` 变体：`Image { source: ImageSource }`，`ImageSource` 包含 `r#type: String`、`media_type: String`、`data: String`
   - `convert_messages()` 对 `UserContent::Parts` 逐项转换为 `ContentBlock::Text` 或 `ContentBlock::Image`
3. **OpenAI provider 适配**（provider crate）：
   - `TryFrom<&Message>` 对 `Message::User` 检查 content 类型：
     - `UserContent::Text` → 现有逻辑不变
     - `UserContent::Parts` → `ChatCompletionRequestUserMessageContent::Array([...])`
4. **Context builder 适配**（context crate）：
   - `build_user_content()` 返回 `UserContent` 而非 `String`
   - 有图片时返回 `UserContent::Parts([Text { text }, Image { ... }, ...])`
   - 无图片时返回 `UserContent::Text(text)`
   - `encode_image_to_base64()` 输出拆分为 `(media_type, data)` 元组
5. **Session 持久化去 base64**（session crate）：
   - `save_turn()` 对 `Message::User` 的 `UserContent::Parts` 进行处理：将 `ContentPart::Image` 替换为 `ContentPart::Text { text: "[image]" }`
   - 如果处理后所有 parts 都是 Text，合并为 `UserContent::Text`
6. **Agent loop 传递 media**（agent crate）：
   - `process_message()` 将 `InboundMessage.media` 传递给 `build_messages()` 而非固定传 `None`
7. **所有读取 content 的代码适配**：
   - `Message::content()` 方法需要处理 `UserContent` 枚举，提取纯文本
   - `strip_think()`、session 持久化、memory 整合等使用 `content()` 的地方无需修改（它们只关心文本）
8. **CLI 图片输入**（nanobot crate）：
   - `nanobot agent -m` 新增 `--image` / `-i` 参数，接受一个或多个图片文件路径
   - `run_once()` 将图片路径传给 `process_direct()` 的 media 参数
   - `process_direct()` 将 media 传递给 `build_messages()`
   - 示例：`nanobot agent -m "这张图片里有什么？" -i photo.png`
9. **Channel 图片接收**（channels crate）：
   - 飞书 channel：接收图片消息时通过飞书 API 下载图片，保存到临时文件，填充 `InboundMessage.media`
   - 钉钉 channel：接收图片消息时通过钉钉 API 下载图片，保存到临时文件，填充 `InboundMessage.media`

## 非功能需求

- **向后兼容**：旧 session JSONL 文件（content 为纯字符串）能正常反序列化（`#[serde(untagged)]` 的 `Text` 变体匹配）
- **性能**：base64 图片数据不持久化到 session，避免文件膨胀
- **测试要求**：
  - `UserContent` 的 serde 序列化/反序列化测试（纯文本、多模态、旧格式兼容）
  - OpenAI/Anthropic provider 的多模态消息转换测试
  - Session `save_turn()` 的图片剥离测试

## 边界与不做事项

- 不支持 assistant 消息中的图片（LLM 生成图片是另一个功能）
- 不支持视频/音频等其他媒体类型（`ContentPart` 枚举可未来扩展）

## 假设与约束

- **技术假设**：async-openai 0.28 已支持 `ChatCompletionRequestUserMessageContent::Array` 和 `ImageUrl` 类型
- **技术假设**：Anthropic Messages API 支持 `{"type": "image", "source": {"type": "base64", "media_type": "...", "data": "..."}}` content block
- **约束**：`ContentPart::Image` 存储裸 base64 数据和 MIME 类型，不存储完整 data URL

## 待确认事项

- session 持久化时是否需要保留图片的元信息（如文件名、尺寸），还是只保留 `[image]` 占位符？暂定只保留占位符，与 Python 版 PR #1191 行为一致
