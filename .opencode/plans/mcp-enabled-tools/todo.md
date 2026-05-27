# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/config/src/schema/mcp.rs` | 修改 | 两个变体新增 `enabled_tools` 字段 + `enabled_tools()` 访问方法 |
| `crates/mcp/src/wrapper.rs` | 修改 | `connect()` 中按 `enabled_tools` 过滤工具 |
| `crates/config/AGENTS.md` | 修改 | 更新 `McpServerConfig` 文档 |

## 任务列表

### 1. 配置新增 enabled_tools 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/mcp.rs`
- 验收标准: `McpServerConfig` 两个变体都有 `enabled_tools` 字段；JSON `{"command":"x","enabledTools":["a"]}` 可正确反序列化；不设置时默认空 Vec
- 信心评估: 5
- 步骤:
  - [ ] `Stdio` 变体新增 `#[serde(default)] enabled_tools: Vec<String>` 字段
  - [ ] `Http` 变体新增同样字段
  - [ ] 新增 `pub fn enabled_tools(&self) -> &[String]` 方法，match 两个变体返回引用
  - [ ] 更新 `stdio()` 和 `http()` 构造方法，初始化 `enabled_tools: Vec::new()`
  - [ ] `cargo check -p nanobot-config` 确认编译通过

### 2. connect() 中过滤工具

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/mcp/src/wrapper.rs`
- 验收标准: `enabled_tools` 非空时仅注册列表中的工具；空时全部注册；日志记录跳过的工具
- 信心评估: 5
- 步骤:
  - [ ] 在 `connect()` 的 `for tool in tools.tools` 循环开头，获取 `config.enabled_tools()`
  - [ ] 若 `enabled_tools` 非空且不包含 `tool.name`，`debug!` 记录并 `continue`
  - [ ] 调整 `info!` 日志：区分总工具数和注册工具数（如 "registered 3/10 tools"）
  - [ ] `cargo check -p nanobot-mcp` 确认编译通过

### 3. 更新文档

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/config/AGENTS.md`
- 验收标准: AGENTS.md 反映 `McpServerConfig` 新增的 `enabled_tools` 字段
- 信心评估: 5
- 步骤:
  - [ ] 在 `McpServerConfig` 描述中添加 `enabled_tools` 字段说明

## 实现建议

- `enabled_tools` 匹配用原始工具名（`tool.name`），不是包装后的 `mcp_server_tool` 前缀名
- 过滤逻辑放在 `for tool in tools.tools` 循环内，用 `contains` 检查即可（工具数量通常 < 100，无需 HashSet）
