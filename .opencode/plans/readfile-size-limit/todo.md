# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/tools/src/fs.rs` | 修改 | ReadFileTool 添加大小检查和截断逻辑 |
| `crates/tools/src/fs/tests.rs` | 新增 | ReadFileTool 大小限制测试 |

## 任务列表

### ✅ 1. 添加大小常量和预读检查

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/tools/src/fs.rs`
- 验收标准: `metadata.len() > MAX_BYTES` 时返回 `ToolError::Execution`，包含文件大小和引导信息
- 风险/注意点: 常量定义在模块级或 ReadFileTool impl 块内均可，保持与现有代码风格一致
- 信心评估: 5
- 步骤:
  - [ ] 在 `ReadFileTool` impl 块上方添加常量 `const MAX_CHARS: u64 = 128_000` 和 `const MAX_BYTES: u64 = MAX_CHARS * 4`
  - [ ] 在 `execute()` 中 `metadata.is_file()` 检查之后、`read_to_string` 之前，插入 `metadata.len() > MAX_BYTES` 检查
  - [ ] 超过时返回 `ToolError::execution(format!("文件过大 ({} bytes)，超过 {} bytes 限制。请使用 exec 工具的 head/tail/grep 命令处理大文件。", metadata.len(), MAX_BYTES))`

### ✅ 2. 添加读取后字符截断

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/tools/src/fs.rs`
- 验收标准: `content.chars().count() > MAX_CHARS` 时截断到 `MAX_CHARS` 字符并追加提示，正常文件不受影响
- 风险/注意点: 用 `content.len() > MAX_CHARS as usize` 做快速路径检查（字节数 < 字符数上限则字符数必然不超），避免对每个文件都做 `chars().count()`
- 信心评估: 5
- 步骤:
  - [ ] 在 `read_to_string` 之后，检查 `content.len() > MAX_CHARS as usize`（快速路径：字节数不超则字符数必然不超）
  - [ ] 快速路径不通过时，计算 `char_count = content.chars().count()`，若 `char_count > MAX_CHARS as usize`，截断 content 到前 `MAX_CHARS` 个字符
  - [ ] 截断方式：`let truncated: String = content.chars().take(MAX_CHARS as usize).collect()`，然后追加 `format!("\n\n[文件已截断：显示前 {} 字符，共 {} 字符 ({} bytes)。使用 exec 工具的 head/tail/grep 查看完整内容。]", MAX_CHARS, char_count, metadata.len())`
  - [ ] 返回截断后的内容

### ✅ 3. 将 fs.rs 重构为目录模块以添加测试

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: `crates/tools/src/fs.rs` → `crates/tools/src/fs/mod.rs`, `crates/tools/src/fs/tests.rs`
- 验收标准: 模块结构正确，现有代码不变，新增 `#[cfg(test)] mod tests;` 声明
- 风险/注意点: 项目规范禁止同名 `foo.rs` 和 `foo/` 共存，必须将 `fs.rs` 移为 `fs/mod.rs`
- 信心评估: 5
- 步骤:
  - [ ] `mkdir crates/tools/src/fs && mv crates/tools/src/fs.rs crates/tools/src/fs/mod.rs`
  - [ ] 在 `mod.rs` 末尾添加 `#[cfg(test)] mod tests;`

### ✅ 4. 编写测试

- 优先级: P0
- 依赖项: 3
- 涉及文件: `crates/tools/src/fs/tests.rs`
- 验收标准: 3 个测试覆盖正常读取、超大文件拒绝、截断场景，全部通过
- 风险/注意点: 测试需创建临时文件，使用 `tempfile` crate（已在 workspace dev-dependencies 中）
- 信心评估: 5
- 步骤:
  - [ ] 创建 `tests.rs`，`use super::*;`
  - [ ] 测试 `read_normal_file`：创建小文件（< 128K chars），验证完整读取成功
  - [ ] 测试 `read_oversized_file_rejected`：创建 > 512KB 文件，验证返回 `ToolError::Execution` 且消息包含 "文件过大"
  - [ ] 测试 `read_large_file_truncated`：创建 130K 字符的文件（< 512KB bytes），验证返回内容被截断且包含截断提示 "[文件已截断"

### ✅ 5. 验证

- 优先级: P0
- 依赖项: 1, 2, 3, 4
- 涉及文件: 无
- 验收标准: 四项检查全部通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `cargo +nightly fmt`
  - [ ] `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args`
  - [ ] `cargo test -p nanobot-tools`
  - [ ] `cargo doc --no-deps`

## 实现建议

- `MAX_CHARS` 和 `MAX_BYTES` 作为模块级常量（非 impl 关联常量），因为未来其他工具可能复用
- 截断用 `chars().take(N).collect::<String>()` 保证 UTF-8 安全，不会切断多字节字符
- 快速路径 `content.len() > MAX_CHARS as usize` 利用了 UTF-8 中字节数 >= 字符数的性质
