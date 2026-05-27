# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/agent/src/loop/mod.rs` | 修改 | `strip_think` 正则扩展，匹配孤立 `</think>` |
| `crates/agent/src/loop/tests.rs` | 修改 | 新增孤立 `</think>` 测试用例 |
| `crates/channels/src/feishu/mod.rs` | 修改 | `send()` 中工具提示格式化为代码块 |

## 任务列表

### 1. 修复 strip_think 处理孤立 </think>

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/agent/src/loop/tests.rs`
- 验收标准: `strip_think("内容</think>尾部")` 返回 `"内容尾部"`；`strip_think("</think>")` 返回 `""`；现有测试不受影响
- 信心评估: 5
- 步骤:
  - [ ] 修改 `strip_think` 中的正则为 `<think>[\s\S]*?</think>|</think>`，先匹配成对标签，再匹配孤立闭合标签
  - [ ] 在 `tests.rs` 的 `strip_think_removes_tags` 测试向量中新增用例：`"孤立闭合标签"` → `"内容</think>尾部"` → `"内容尾部"`；`"仅孤立闭合标签"` → `"</think>"` → `""`
  - [ ] `cargo test -p nanobot-agent -- strip_think` 确认通过

### 2. 飞书工具提示代码块格式化

- 优先级: P1
- 依赖项: 无（与任务 1 并行）
- 涉及文件: `crates/channels/src/feishu/mod.rs`
- 验收标准: 当 `msg.is_tool_hint()` 为 true 时，飞书卡片中工具提示以代码块格式显示；普通消息和 progress 消息不受影响
- 信心评估: 5
- 步骤:
  - [ ] 在 `send()` 方法中，构造 `markdown_content` 之前，检查 `msg.is_tool_hint()`
  - [ ] 若为 tool hint：将 `msg.content` 中的 `, ` 分隔的工具调用拆分为多行，包裹在 markdown 代码块中（`` ```\ntool1(...)\ntool2(...)\n``` ``）
  - [ ] 若非 tool hint：保持现有 `**Nanobot Reply**\n\n{content}` 格式
  - [ ] `cargo check -p nanobot-channels` 确认编译通过

## 实现建议

- `strip_think` 正则用 `|` 分支即可：`<think>[\s\S]*?</think>|</think>`
- 飞书代码块：tool hint 的 content 格式为 `"tool1(arg=\"val\"), tool2(arg=\"val\")`，按 `, ` split 后每行一个，包裹在 `` ``` `` 中
