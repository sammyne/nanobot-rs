# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/channels/src/feishu/post/mod.rs` | 新增 | post 消息内容提取逻辑 |
| `crates/channels/src/feishu/post/tests.rs` | 新增 | extract_post_content 单元测试 |
| `crates/channels/src/feishu/mod.rs` | 修改 | 注册 post 模块，添加 "post" 消息类型分支 |

## 任务列表

### 1. ✅ 新增 post 模块实现 extract_post_content

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/feishu/post/mod.rs`
- 验收标准: `extract_post_content(&serde_json::Value) -> (String, Vec<String>)` 函数能正确解析三种 post 格式，返回 (文本, image_keys)
- 风险/注意点: 三种格式的 fallback 顺序需与上游 Python 版一致
- 信心评估: 5
- 步骤:
  - [ ] 创建 `crates/channels/src/feishu/post/mod.rs`
  - [ ] 定义 post 元素的 serde 数据模型（PostElement enum: Text/A/At/Img + Unknown fallback）
  - [ ] 实现 `extract_post_content()`：先尝试 `json["post"]` 解包 → 再尝试 locale keys (zh_cn/en_us/ja_jp) → 再尝试直接 `content` 字段 → fallback 到首个 dict 子项
  - [ ] 从 content 二维数组中提取文本（text/a 的 text 字段）和 image_keys（img 的 image_key 字段）
  - [ ] title 存在时拼接到文本开头

### 2. ✅ 新增 post 模块单元测试

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/channels/src/feishu/post/tests.rs`
- 验收标准: 覆盖三种格式 + 空内容 + 无 title + 含图片 场景
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 创建 `crates/channels/src/feishu/post/tests.rs`
  - [ ] 测试格式 1：`{"post": {"zh_cn": {...}}}` 包裹格式
  - [ ] 测试格式 2：`{"zh_cn": {...}}` 直接本地化格式
  - [ ] 测试格式 3：`{"title": "...", "content": [...]}` 直接内容格式
  - [ ] 测试空内容返回空字符串和空 vec
  - [ ] 测试含 img 元素时 image_keys 正确提取

### 3. ✅ 在 process_message 中接入 post 消息类型

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/channels/src/feishu/mod.rs`
- 验收标准: 消息类型白名单包含 `"post"`，收到 post 消息时调用 `extract_post_content()` 提取文本
- 风险/注意点: image_keys 暂不处理（不下载），仅提取文本
- 信心评估: 5
- 步骤:
  - [ ] 在 `mod.rs` 顶部添加 `mod post;`
  - [ ] 修改消息类型白名单：添加 `&& message_type != "post"` 条件
  - [ ] 在 match 中新增 `"post"` 分支，调用 `post::extract_post_content()`，将返回的文本设为 `content`
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 确认无警告
  - [ ] 运行 `cargo test` 确认所有测试通过

## 实现建议

- 按 requirements.md「数据模型定义」章节定义三个结构体：`PostContent`（顶层，含三种格式的字段）、`LocaleContent`（locale 内容，含 title + content 二维数组）、`PostElement`（enum，按 tag 区分 Text/A/At/Img/Unknown）
- `PostElement` 使用 `#[serde(tag = "tag", rename_all = "snake_case")]`，参考 `interactive/mod.rs` 的 `CardElement` 风格
- `PostContent` 使用 `#[serde(default)]` 使所有字段可选，一次反序列化即可覆盖三种格式
- `extract_post_content()` 的签名返回 tuple `(String, Vec<String>)` 而非仅 String，为后续图片下载预留接口
- 解析优先级：`post` 字段（格式 1）→ `zh_cn`/`en_us`/`ja_jp` 字段（格式 2）→ `title`+`content` 字段（格式 3）→ `post` HashMap 中任意首个值
