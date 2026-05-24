# templates crate

工作空间初始化模板，编译时通过 `include_dir!` 嵌入二进制。

## 关键类型

纯函数 API，无结构体：

- `get_template(path) -> Option<&'static str>` -- 通用模板文件访问
- `user_template() -> &'static str` -- USER.md 模板
- `agents_template() -> &'static str` -- AGENTS.md 模板
- `soul_template() -> &'static str` -- SOUL.md 模板
- `tools_template() -> &'static str` -- TOOLS.md 模板
- `memory_template() -> &'static str` -- memory/MEMORY.md 模板
- `heartbeat_template() -> &'static str` -- HEARTBEAT.md 模板

## 内部依赖

无（叶子 crate）
