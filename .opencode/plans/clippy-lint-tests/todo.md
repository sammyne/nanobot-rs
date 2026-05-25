# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `AGENTS.md` | 修改 | 更新 clippy 命令加 `--all-targets` |
| `.github/workflows/build.yml` | 修改 | 更新 CI clippy 命令加 `--all-targets` |
| `crates/config/src/schema/tests.rs` | 修改 | 修复 4 处 `field_reassign_with_default` lint 错误 |

## 任务列表

### ✅ 1. 更新 clippy 命令加 `--all-targets`

- 优先级: P0
- 依赖项: 无
- 涉及文件: `AGENTS.md`, `.github/workflows/build.yml`
- 验收标准: 两个文件中的 clippy 命令均包含 `--all-targets` 标志
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 在 `AGENTS.md` 第 149 行，将 `cargo clippy -- -D warnings -D clippy::uninlined_format_args` 改为 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 在 `.github/workflows/build.yml` 第 56 行，将 `cargo clippy -- -D warnings -D clippy::uninlined_format_args` 改为 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args`

### ✅ 2. 修复 `crates/config/src/schema/tests.rs` 中的 lint 错误

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/tests.rs`
- 验收标准: `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 对 `nanobot-config` crate 无错误
- 风险/注意点: 修复方式是将 `let mut config = Config::default(); config.providers = ...;` 改为 `let config = Config { providers: ..., ..Default::default() };`，需确认 `Config` 的字段可见性允许这样构造
- 信心评估: 4（clippy 已给出具体修复建议，但需确认 `Config` 结构体字段是否 pub）
- 步骤:
  - [ ] 修复第 77-82 行（`validate_invalid_api_base`）：将 `let mut config = Config::default(); config.providers = ...;` 改为 `let config = Config { providers: ProvidersConfig::Custom(ProviderConfig { api_key: "test-key".to_string(), api_base: Some("invalid-url".to_string()), extra_headers: None }), ..Default::default() };`
  - [ ] 修复第 88-93 行（`validate_short_api_key`）：同上模式，使用 `Config { providers: ProvidersConfig::Custom(...), ..Default::default() }`
  - [ ] 修复第 129-134 行（`masked_api_key_short`）：同上模式
  - [ ] 修复第 909-914 行（`providers_config_anthropic_validate_invalid_api_base`）：同上模式，使用 `ProvidersConfig::Anthropic(...)`

### ✅ 3. 检查并修复其他 crate 的测试 lint 错误

- 优先级: P0
- 依赖项: 2
- 涉及文件: 待定（取决于 clippy 输出）
- 验收标准: `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 全量通过，零错误零警告
- 风险/注意点: 任务 2 修复后 clippy 才能继续检查后续 crate，可能暴露更多错误
- 信心评估: 3（不确定其他 crate 是否有错误，需实际运行确认）
- 步骤:
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args`，检查是否还有其他 crate 的测试 lint 错误
  - [ ] 如有错误，逐个修复（仅修改 lint 问题，不改测试逻辑）
  - [ ] 重复运行 clippy 直到全量通过

### ✅ 4. 验证

- 优先级: P0
- 依赖项: 1, 2, 3
- 涉及文件: 无
- 验收标准: 以下命令全部通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 确认零错误
  - [ ] 运行 `cargo test` 确认测试仍全部通过

## 实现建议

- clippy 给出的修复建议可直接采用：用 `Config { providers: ..., ..Default::default() }` 替代先 default 再赋值的模式
- 如果 `Config` 字段非 pub 导致无法直接构造，可改用 `#[allow(clippy::field_reassign_with_default)]` 局部抑制，但优先尝试 clippy 建议的写法
