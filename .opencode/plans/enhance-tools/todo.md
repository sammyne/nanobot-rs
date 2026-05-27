# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/tools/src/fs/mod.rs` | 修改 | ReadFileTool 分页、EditFileTool replace_all、ListDirTool max_entries |
| `crates/tools/src/fs/tests.rs` | 修改 | 新增分页/replace_all/max_entries 测试 |
| `crates/tools/src/shell/mod.rs` | 修改 | ExecTool head+tail 截断 |
| `crates/tools/src/shell/utils/mod.rs` | 修改 | truncate_output 改为 head+tail 模式 |
| `crates/tools/src/shell/utils/tests.rs` | 修改 | 新增 head+tail 截断测试 |
| `crates/tools/tests/filesystem.rs` | 修改 | 集成测试适配新参数 |

## 任务列表

### 1. ReadFileTool 分页

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/tools/src/fs/mod.rs`, `crates/tools/src/fs/tests.rs`
- 验收标准: `read_file` 接受 `offset`（起始行号，1-indexed，默认 1）和 `limit`（最大行数，默认 2000）参数；输出每行带行号前缀 `{line_no}: {content}`；offset 超出文件行数时返回空内容并提示总行数
- 风险/注意点: 行号从 1 开始（与 opencode 的 Read 工具对齐）；现有的字符截断逻辑保留作为最终安全网
- 信心评估: 5
- 步骤:
  - [ ] `ReadFileArgs` 新增 `offset: Option<u64>`（描述：起始行号，1-indexed）和 `limit: Option<u64>`（描述：最大读取行数）
  - [ ] `execute()` 中读取文件后，按行分割，根据 offset/limit 截取行范围
  - [ ] 输出格式改为每行 `{line_no}: {content}`，末尾附加 `(Showing lines {start}-{end} of {total}. Use offset={end+1} to continue.)`
  - [ ] 新增测试：默认参数读取全文、offset+limit 读取中间段、offset 超出范围
  - [ ] 运行 `cargo test -p nanobot-tools` 验证通过

### 2. EditFileTool replace_all

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/tools/src/fs/mod.rs`, `crates/tools/src/fs/tests.rs`
- 验收标准: `edit_file` 接受 `replace_all: bool` 参数（默认 false）；为 true 时替换所有匹配并返回替换数量；为 false 时保持现有行为（多匹配报错）
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `EditFileArgs` 新增 `replace_all: Option<bool>`
  - [ ] `execute()` 中当 `replace_all == true` 且匹配数 > 0 时，使用 `content.replace(&old_text, &new_text)` 替换所有匹配，返回 `"已替换 {n} 处匹配"`
  - [ ] 新增测试：replace_all=true 替换多处、replace_all=false 多匹配报错（现有行为）
  - [ ] 运行 `cargo test -p nanobot-tools` 验证通过

### 3. ListDirTool max_entries

- 优先级: P1
- 依赖项: 无
- 涉及文件: `crates/tools/src/fs/mod.rs`, `crates/tools/src/fs/tests.rs`
- 验收标准: `list_dir` 接受 `max_entries: u64` 参数（默认 200）；超出时截断并附加 `"... 共 {total} 个条目，已显示前 {max_entries} 个"`
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `ListDirArgs` 新增 `max_entries: Option<u64>`
  - [ ] `execute()` 中收集条目后，如果数量超过 max_entries，截断并附加提示
  - [ ] 新增测试：条目数超过 max_entries 时截断
  - [ ] 运行 `cargo test -p nanobot-tools` 验证通过

### 4. ExecTool head+tail 截断

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/tools/src/shell/mod.rs`, `crates/tools/src/shell/utils/mod.rs`, `crates/tools/src/shell/utils/tests.rs`
- 验收标准: 大输出保留前 5000 + 后 5000 字符，中间用 `\n...(truncated {omitted} chars)...\n` 连接；短输出不变；退出码始终可见
- 风险/注意点: 需要修改 `truncate_output` 函数的截断策略
- 信心评估: 5
- 步骤:
  - [ ] 修改 `truncate_output(s, max_len)` 为 head+tail 模式：如果 `s.len() > max_len`，取前 `max_len/2` 字符 + `\n...(truncated {omitted} chars)...\n` + 后 `max_len/2` 字符
  - [ ] 新增测试：短字符串不截断、长字符串保留头尾、截断提示包含省略字符数
  - [ ] 运行 `cargo test -p nanobot-tools` 验证通过

## 实现建议

- 任务 1-4 互不依赖，可并行实现
- ReadFileTool 的行号输出格式与 opencode 的 Read 工具一致，方便 LLM 理解
- ExecTool 的 head+tail 截断确保错误信息（通常在尾部）不被丢失
