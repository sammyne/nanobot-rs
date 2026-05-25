# 需求

## 目标与背景

当前 `cargo clippy` 命令不带 `--all-targets`，导致 `#[cfg(test)]` 代码（即所有 `tests.rs` 文件）不被编译和检查。项目有 41 个 `tests.rs` 文件，其中的 lint 错误完全不可见。

实际验证：加上 `--all-targets` 后，`crates/config/src/schema/tests.rs` 立即暴露 4 个 `clippy::field_reassign_with_default` 错误，编译在此中断，其他 crate 的测试代码尚未被检查。

不修复的后果：测试代码质量无保障，lint 规则形同虚设，随着测试代码增长问题会累积。

## 方案比较（强制）

### 方案 1: 加 `--all-targets`（最小可行版）

- 思路: 在 clippy 命令中加 `--all-targets` 标志，覆盖 lib、bin、tests、examples、benches 所有目标
- 优点: 一行改动，覆盖面最广，是 Rust 社区标准做法
- 缺点: 无明显缺点
- 工作量估算: S

### 方案 2: 加 `--tests`（理想架构）

- 思路: 仅加 `--tests` 标志，只额外覆盖测试目标
- 优点: 更精确，只增加测试目标的检查
- 缺点: 不覆盖 examples 和 benches（虽然当前项目没有，但未来可能添加）
- 工作量估算: S

### 推荐

推荐方案 1（`--all-targets`）。这是 Rust 社区的标准做法，覆盖面更广，且无额外成本。

## 功能需求列表

### 核心功能

- 更新 `AGENTS.md` 中的 clippy 命令，加 `--all-targets`
- 更新 `.github/workflows/build.yml` 中的 clippy 命令，加 `--all-targets`
- 修复所有因此暴露的现有 lint 错误

### 扩展功能

- 无

## 非功能需求

- **兼容性**：修复后 `cargo clippy --all-targets` 和 `cargo test` 均应通过
- **可维护性**：修复方式应遵循 clippy 建议的惯用写法

## 边界与不做事项

- 不修改测试逻辑，只修复 lint 警告
- 不重构测试代码结构
- 不添加新的 clippy lint 规则

## 假设与约束

- **技术假设**：`--all-targets` 不会引入编译时间的显著增加（测试代码本来就在 `cargo test` 时编译）

## 待确认事项

- 无
