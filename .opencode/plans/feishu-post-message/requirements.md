# 需求

## 目标与背景

飞书 channel 当前仅支持 `text`、`image`、`interactive` 三种消息类型，收到 `post`（富文本）消息时直接忽略。用户在飞书中发送富文本消息（含标题、链接、@提及、图片等）无法被 bot 处理。

上游 HKUDS/nanobot PR #1361 修复了 post 消息的嵌套格式解析问题。本需求在 nanobot-rs 中完整实现 post 消息类型支持，同时包含 PR #1361 的嵌套格式兼容。

## 飞书 post 消息格式

`msg_type` 为 `"post"` 时，`content` 是 JSON 字符串，可能有三种格式：

### 格式 1：带 `post` 包裹（PR #1361 修复的场景）
```json
{"post": {"zh_cn": {"title": "日报", "content": [[{"tag": "text", "text": "完成"}, {"tag": "img", "image_key": "img_1"}]]}}}
```

### 格式 2：直接本地化
```json
{"zh_cn": {"title": "日报", "content": [[{"tag": "text", "text": "完成"}]]}}
```

### 格式 3：直接内容（无 locale key）
```json
{"title": "日报", "content": [[{"tag": "text", "text": "完成"}]]}
```

每行（content 的每个子数组）包含元素，按 `tag` 区分：
- `text`：`{"tag": "text", "text": "文本内容"}`
- `a`：`{"tag": "a", "text": "链接文字", "href": "http://..."}`
- `at`：`{"tag": "at", "user_id": "ou_xxx"}`
- `img`：`{"tag": "img", "image_key": "img_xxx"}`

## 数据模型定义

使用 serde 强类型反序列化，禁止 `serde_json::Value` 手动字段访问。参考 `interactive/mod.rs` 的 `CardElement` enum 风格。

### 顶层结构：PostContent

处理三种格式的统一入口。先尝试 `post` 字段解包，再尝试 locale keys，最后尝试直接内容。

```rust
/// Post 消息顶层结构。
///
/// 兼容三种格式：
/// 1. `{"post": {"zh_cn": {...}}}` — 带 post 包裹
/// 2. `{"zh_cn": {...}}` — 直接本地化（与格式 1 解包后相同）
/// 3. `{"title": "...", "content": [...]}` — 直接内容（即 LocaleContent）
#[derive(Deserialize, Default)]
#[serde(default)]
struct PostContent {
    /// 格式 1 的外层包裹
    post: Option<HashMap<String, LocaleContent>>,
    /// 格式 2 的直接 locale keys（与 post 字段互斥，fallback 使用）
    zh_cn: Option<LocaleContent>,
    en_us: Option<LocaleContent>,
    ja_jp: Option<LocaleContent>,
    /// 格式 3 的直接内容字段
    title: Option<String>,
    content: Option<Vec<Vec<PostElement>>>,
}
```

### 本地化内容：LocaleContent

```rust
/// 单个 locale 下的富文本内容。
#[derive(Deserialize, Default)]
#[serde(default)]
struct LocaleContent {
    /// 可选标题
    title: Option<String>,
    /// 二维数组：每行是一个元素列表
    content: Vec<Vec<PostElement>>,
}
```

### 富文本元素：PostElement

```rust
/// Post 富文本元素，按 `tag` 字段区分类型。
#[derive(Deserialize)]
#[serde(tag = "tag", rename_all = "snake_case")]
enum PostElement {
    /// 纯文本
    Text { text: Option<String> },
    /// 超链接
    A { text: Option<String>, href: Option<String> },
    /// @提及
    At { user_id: Option<String> },
    /// 内嵌图片
    Img { image_key: Option<String> },
    /// 未知 tag（兼容未来新增类型）
    #[serde(other)]
    Unknown,
}
```

### 解析优先级

`extract_post_content()` 按以下顺序查找有效的 `LocaleContent`：

1. `post_content.post`（格式 1）→ 按 locale fallback 顺序取值
2. `post_content.zh_cn` / `en_us` / `ja_jp`（格式 2）→ 取首个 `Some`
3. 构造 `LocaleContent { title: post_content.title, content: post_content.content }`（格式 3）
4. 以上均无有效内容 → 返回空

Locale fallback 顺序：`zh_cn` → `en_us` → `ja_jp` → HashMap 中任意首个值。

## 方案比较（强制）

### 方案 1: 新增 `post` 模块，与 `interactive` 模块平行

- 思路: 在 `crates/channels/src/feishu/` 下新增 `post/mod.rs` + `post/tests.rs`，提供 `extract_post_content()` 函数。`process_message()` 中新增 `"post"` 分支调用该函数
- 优点: 模块职责清晰，与 `interactive` 模块结构对齐，可独立测试
- 缺点: 新增一个模块
- 工作量估算: S

### 方案 2: 在 `process_message()` 中内联解析

- 思路: 直接在 `process_message()` 的 match 分支中用 `serde_json::Value` 手动解析
- 优点: 无新文件
- 缺点: `process_message()` 已经较长（~150 行），内联解析会进一步膨胀；三种格式的 fallback 逻辑不适合内联
- 工作量估算: S

### 推荐

方案 1。与现有 `interactive` 模块结构一致，解析逻辑可充分测试。

## 功能需求列表

### 核心功能

1. 在 `process_message()` 的消息类型白名单中添加 `"post"`
2. 新增 `post` 模块，提供 `extract_post_content(json) -> (String, Vec<String>)` 函数：
   - 返回 `(text_content, image_keys)`
   - 兼容三种格式（带 `post` 包裹、直接本地化、直接内容）
   - 本地化 fallback 顺序：`zh_cn` → `en_us` → `ja_jp` → 任意首个 dict 子项
   - 从 `text`/`a` 元素提取文本，从 `img` 元素提取 `image_key`
   - title 存在时拼接到文本开头
3. 在 `process_message()` 中新增 `"post"` match 分支，调用 `extract_post_content()`，将提取的文本设为 `content`，image_keys 暂不处理（当前 image 下载逻辑仅在 `"image"` 分支中，post 中的内嵌图片作为后续扩展）

### 扩展功能

- post 消息中内嵌图片的下载（需复用 image 下载逻辑，可作为后续 PR）

## 非功能需求

- **测试要求**: 为 `extract_post_content()` 编写单元测试，覆盖三种格式 + 空内容 + 无 title 场景
- **兼容性**: 不影响现有 text/image/interactive 消息处理
- **可维护性**: 模块结构与 `interactive` 对齐

## 边界与不做事项

- 不实现 post 消息中内嵌图片的下载（仅提取 image_keys，不调用下载 API）
- 不修改发送端（`send()` 方法保持 interactive card 格式）
- 不处理 `at` 元素的用户 ID 解析（仅提取为 `@user` 文本占位符）

## 假设与约束

- **技术假设**: 飞书事件推送的 post 消息 content 字段是合法 JSON 字符串
- **资源约束**: 无
- **环境约束**: 无

## 待确认事项

- 无
