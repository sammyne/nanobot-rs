# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/channels/src/feishu/interactive/mod.rs` | 新增 | interactive 卡片消息文本提取函数 |
| `crates/channels/src/feishu/interactive/tests.rs` | 新增 | 提取函数的单元测试 |
| `crates/channels/src/feishu/mod.rs` | 修改 | 添加 `"interactive"` 到消息类型白名单，调用提取函数 |

## 任务列表

### ✅ 1. 实现 interactive 消息文本提取函数

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/feishu/interactive/mod.rs`
- 验收标准: `extract_interactive_content(&Value) -> String` 能从各种 interactive 卡片 JSON 中提取文本
- 风险/注意点: elements 是二维数组（行列表，每行是元素列表）；content 可能是 JSON 字符串需先解析
- 信心评估: 5（Python 版有完整参考实现）
- 步骤:
  - [ ] 创建 `crates/channels/src/feishu/interactive/mod.rs`
  - [ ] 实现 `extract_interactive_content(content: &Value) -> String`：提取 title（dict 或 string）、遍历 elements（双层嵌套）、递归处理 card、提取 header.title
  - [ ] 实现 `fn extract_element_content(element: &Value) -> Vec<String>`：按 tag 分发——markdown/lark_md（取 content）、div（取 text + fields）、a（取 href + text）、button（取 text + url/multi_url）、img（取 alt）、note（递归 elements）、column_set（递归 columns.elements）、plain_text（取 content）、fallback（递归 elements）
  - [ ] 对非预期类型（elements 不是数组、element 不是对象等）打印 `warn!` 日志后跳过
  - [ ] 在 `mod.rs` 末尾添加 `#[cfg(test)] mod tests;`
  - [ ] 运行 `cargo check -p nanobot-channels` 验证编译

### ✅ 2. 在 feishu 消息处理中集成 interactive 类型

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/channels/src/feishu/mod.rs`
- 验收标准: `message_type == "interactive"` 的消息不再被忽略，文本内容被正确提取并转发
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 在 `mod.rs` 顶部添加 `mod interactive;`
  - [ ] 修改消息类型白名单（第 124 行）：从 `!= "text" && != "image"` 改为 `!= "text" && != "image" && != "interactive"`
  - [ ] 在 `match message_type` 中添加 `"interactive"` 分支：解析 content_str 为 JSON，调用 `interactive::extract_interactive_content()`
  - [ ] 运行 `cargo check -p nanobot-channels` 验证编译

### ✅ 3. 新增提取函数单元测试

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/channels/src/feishu/interactive/tests.rs`
- 验收标准: 覆盖主要 tag 类型和边界情况
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 创建 `crates/channels/src/feishu/interactive/tests.rs`
  - [ ] 测试 title 提取（string 类型、dict 类型）
  - [ ] 测试 elements 双层嵌套遍历（markdown、div、plain_text）
  - [ ] 测试 header.title 提取
  - [ ] 测试 card 递归提取
  - [ ] 测试 button（text + url）、a（href + text）、img（alt）
  - [ ] 测试 note 和 column_set 递归
  - [ ] 测试非预期类型（elements 不是数组）不 panic
  - [ ] 测试空 content / 空 elements
  - [ ] 运行 `cargo test -p nanobot-channels` 验证通过

## 实现建议

- `extract_element_content` 中的 tag 分发用 `match tag { ... }` 实现，fallback 分支处理未知 tag（尝试递归 elements 子数组）
- 所有 JSON 字段访问用 `Value::get()` + `as_str()` / `as_array()` / `as_object()` 模式，避免 unwrap
- `extract_interactive_content` 的返回值用 `parts.join("\n")` 拼接，与 Python 版一致
