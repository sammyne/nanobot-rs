# nanobot-rs 开发指南

## Rust 版本要求

本项目要求 **Rust >= 1.93**。

检查您的 Rust 版本：
```bash
rustc --version
```

To install or update Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

## 项目结构

```
.
├── Cargo.toml  # 工作空间描述文件，包含依赖和元数据
├── crates/
│   ├── *       # 子库或者可执行文件 crates
```

## 工作空间规范

本项目采用 Cargo 工作空间（Workspace）管理多个 crate。以下是必须遵循的规范：

### 成员 crate 位置

所有成员 crate 都必须放在 `crates/` 文件夹下。

```
crates/
├── cli/        # 命令行工具
├── config/     # 配置库
└── core/       # 核心库
```

### 成员 crate 命名

成员 crate 的文件夹名称**不需要**带项目前缀。

- ✅ 正确：`crates/cli/`（crate 名称为 `cli`）
- ❌ 错误：`crates/nanobot-cli/`（不应包含项目前缀）

在 `Cargo.toml` 中的声明：

```toml
[workspace]
members = [
    "crates/cli",
    "crates/config",
    "crates/core",
]
```

### 共用依赖管理

成员 crate 共用的依赖必须声明在工作空间的 `[workspace.dependencies]` 小节，并且所有成员 crate 的对应依赖都需要指向工作空间的版本。

#### 依赖声明规范

遵循以下两条核心规则：

1. **单一配置**：使用简化的点号语法
2. **多个配置**：使用 TOML 表语法

**单一配置示例（点号语法）：**

```toml
# 仅版本号
thiserror = "1.0"

# 引用工作空间依赖
thiserror.workspace = true

# 仅指定 path
nanobot-config.path = "crates/config"
```

**多个配置示例（表语法）：**

```toml
# 版本 + features
[workspace.dependencies.serde]
version = "1.0"
features = ["derive"]

# 版本 + features
[workspace.dependencies.tokio]
version = "1.0"
features = ["full"]

# 版本 + features + optional
[dependencies.reqwest]
version = "0.11"
features = ["json"]
optional = true
```

**完整示例：**

```toml
# === 工作空间根 Cargo.toml ===

[workspace.dependencies]
# 单一配置（点号语法）
thiserror = "1.0"
anyhow = "1.0"
nanobot-config.path = "crates/config"

# 多个配置（表语法）
[workspace.dependencies.serde]
version = "1.0"
features = ["derive"]

[workspace.dependencies.tokio]
version = "1.0"
features = ["rt-multi-thread", "macros"]

# === 成员 crate 的 Cargo.toml ===

[dependencies]
# 引用工作空间依赖（单一配置，点号语法）
thiserror.workspace = true
anyhow.workspace = true
serde.workspace = true

# crate 特有依赖
clap = "4.0"
```

✅ **推荐：**

```toml
# 单一配置：点号语法
thiserror.workspace = true
nanobot-config.path = "crates/config"

# 多个配置：表语法
[workspace.dependencies.tokio]
version = "1.0"
features = ["full"]
```

❌ **避免：**

```toml
# 单一配置不应使用花括号
thiserror = { workspace = true }

# 单一配置不应使用表语法
[workspace.dependencies.thiserror]
version = "1.0"

# 多个配置不应使用花括号（可读性差）
serde = { version = "1.0", features = ["derive"] }
```

**工作空间根 `Cargo.toml`：**

```toml
[workspace]
members = [
    "crates/cli",
    "crates/config",
    "crates/core",
]

[workspace.dependencies]
thiserror = "1.0"
anyhow = "1.0"

[workspace.dependencies.serde]
version = "1.0"
features = ["derive"]

[workspace.dependencies.tokio]
version = "1.0"
features = ["full"]
```

**成员 crate 的 `Cargo.toml`：**

```toml
[package]
name = "cli"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror.workspace = true
anyhow.workspace = true
serde.workspace = true

# crate 特有的依赖可以直接声明
clap = "4.0"
```

### 工作空间规范的优势

1. **统一版本管理**：共用依赖只需在一个地方声明版本，避免版本不一致
2. **简化依赖更新**：更新依赖只需修改工作空间的 `Cargo.toml`
3. **清晰的命名空间**：crate 名称简洁明了，避免冗余前缀
4. **更好的代码组织**：所有成员 crate 集中在 `crates/` 目录下

## 错误处理

本项目根据代码类型采用不同的错误处理策略：

| 代码类型 | 错误处理库 | 说明 |
|---------|-----------|------|
| 库（library） | `thiserror` | 定义清晰的错误类型，便于调用者精确处理 |
| 可执行文件（binary） | `anyhow` | 提供灵活的错误传播和上下文信息，简化应用层错误处理 |

### 库（library）：使用 `thiserror`

`thiserror` 用于库代码，提供以下优点：

- 提供清晰、声明式的错误定义
- 自动实现 `std::error::Error`
- 支持错误来源和回溯
- 最少的样板代码
- 调用者可以精确匹配和处理错误类型

#### 添加 `thiserror` 依赖

在库的 `Cargo.toml` 中：
```toml
[dependencies]
thiserror = "1.0"
```

### 可执行文件（binary）：使用 `anyhow`

`anyhow` 用于可执行文件，提供以下优点：

- 简单的错误传播（使用 `?` 操作符）
- 支持使用 `.context()` 和 `.with_context()` 添加语义化上下文信息
- 自动将任意错误类型转换为 `anyhow::Error`
- 便于应用层统一处理各类错误

#### 添加 `anyhow` 依赖

在可执行文件的 `Cargo.toml` 中：
```toml
[dependencies]
anyhow = "1.0"
```

### 库错误定义模式（thiserror）

在库中创建 `src/error.rs` 文件定义错误类型：

```rust
use thiserror::Error;

/// 库中可能出现的错误
#[derive(Error, Debug)]
pub enum LibraryError {
    /// I/O 操作错误
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 解析错误
    #[error("解析输入失败: {message}")]
    Parse { message: String, line: usize },

    /// 提供的配置无效
    #[error("配置无效: {0}")]
    InvalidConfig(String),

    /// 资源未找到
    #[error("未找到资源 '{name}'")]
    NotFound { name: String },

    /// 网络连接失败
    #[error("网络错误: {0}")]
    Network(String),
}
```

每个错误枚举值都不要带 `Error` 前缀或后缀。

### 在库代码中使用错误（thiserror）

```rust
use crate::error::LibraryError;

pub fn process_data(input: &str) -> Result<String, LibraryError> {
    // 从 io::Error 自动转换
    let file = std::fs::read_to_string("config.toml")?;

    // 自定义错误变体
    if input.is_empty() {
        return Err(LibraryError::InvalidConfig(
            "输入不能为空".to_string()
        ));
    }

    Ok(input.to_uppercase())
}
```

在 `src/lib.rs` 中导出错误模块：

```rust
pub mod error;
pub use error::LibraryError;
```

### 在可执行文件中使用错误（anyhow）

可执行文件通常调用库函数，使用 `anyhow` 处理错误：

```rust
use anyhow::{Context, Result};
use my_library::process_data;

fn main() -> Result<()> {
    // 使用 context() 添加语义化信息
    let config = std::fs::read_to_string("config.toml")
        .context("无法读取配置文件")?;
    
    // 调用库函数，自动转换错误类型
    let result = process_data(&config)
        .context("处理数据失败")?;
    
    println!("结果: {}", result);
    Ok(())
}
```

也可以使用 `with_context()` 延迟计算上下文信息：

```rust
use anyhow::{Context, Result};

fn process_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("无法读取文件: {}", path))?;
    
    Ok(content.to_uppercase())
}
```

## 测试实践

### 单元测试

测试代码和源代码应分离在不同模块。对于模块 `hello`，目录结构如下：

```
|-hello
  |-mod.rs    // 源代码
  |-tests.rs  // 测试代码
```

**源代码文件 `hello/mod.rs`：**

```rust
// hello/mod.rs

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub fn calculate_score(value: u32) -> Result<u32, String> {
    if value == 0 {
        Err("value cannot be zero".to_string())
    } else {
        Ok(value * 2)
    }
}

// 在源文件末尾引入测试模块
#[cfg(test)]
mod tests;  // 引入同目录下的 tests.rs
```

**测试代码文件 `hello/tests.rs`：**

```rust
// hello/tests.rs

use super::*;

#[test]
fn greet() {
    assert_eq!(greet("World"), "Hello, World!");
    assert_eq!(greet("Rust"), "Hello, Rust!");
}

#[test]
fn calculate_score_success() {
    assert_eq!(calculate_score(5).unwrap(), 10);
    assert_eq!(calculate_score(100).unwrap(), 200);
}

#[test]
fn calculate_score_error() {
    assert!(calculate_score(0).is_err());
    assert_eq!(calculate_score(0).unwrap_err(), "value cannot be zero");
}
```

**模块导出文件 `src/modules/mod.rs`：**

```rust
pub mod hello;
```

### 集成测试

在 `tests/` 目录中创建文件：

```rust
// tests/integration_test.rs
use my_library::process_data;

#[test]
fn public_api() {
    assert_eq!(process_data("test").unwrap(), "TEST");
}
```

### 测试组织最佳实践

1. **测试命名**: 使用描述性名称，直接描述被测试的功能或场景，不需要 `test_` 前缀
   - 示例: `greet()`、`calculate_score_success()`、`empty_input_handling()`
2. **模块分离**: 测试代码应与源代码分离到独立的 `tests.rs` 文件
3. **目录结构**: 使用子目录组织模块，源代码和测试代码在同一目录下
4. **Arrange-Act-Assert 模式**: 清晰组织测试结构
5. **测试边界情况**: 包括空输入、边界值和错误条件
6. **使用 `assert!` 和 `assert_eq!`**: 选择合适的断言
7. **测试错误变体**: 验证所有错误分支正确工作

**完整的示例目录结构：**

```
src/
├── lib.rs
├── error.rs
└── modules/
    ├── mod.rs           # 模块导出
    ├── hello/
    │   ├── mod.rs       # hello 模块源代码
    │   └── tests.rs     # hello 模块测试代码
    └── calculator/
        ├── mod.rs       # calculator 模块源代码
        └── tests.rs     # calculator 模块测试代码
```

**测试代码示例 `hello/tests.rs`：**

```rust
use super::*;

#[test]
fn calculate_score_handles_zero_input() {
    // Arrange
    let zero_input = 0;

    // Act
    let result = calculate_score(zero_input);

    // Assert
    assert!(matches!(result, Err(ref msg) if msg.contains("zero")));
}
```

**这种结构的优势：**
- 源代码和测试代码完全分离，便于维护
- 测试文件专注于测试逻辑，不干扰源代码阅读
- 保持模块的组织清晰
- 遵循 Rust 项目的标准组织方式

## 文档标准

### 行内文档

使用 `///` 记录公共 API 元素：

```rust
/// 处理输入字符串并将其转换为大写。
///
/// # 参数
///
/// * `input` - 要处理的字符串切片
///
/// # 返回值
///
/// 返回 `Ok(String)` 包含大写版本，如果处理失败则返回错误。
///
/// # 错误
///
/// 如果输入为空，返回 `LibraryError::InvalidConfig`。
///
/// # 示例
///
/// ```
/// use my_library::process_data;
///
/// let result = process_data("hello");
/// assert_eq!(result.unwrap(), "HELLO");
/// ```
pub fn process_data(input: &str) -> Result<String, LibraryError> {
    // implementation
}
```

### 模块文档

在 `lib.rs` 中添加模块级文档：

```rust
//! # 我的库
//!
//! 一个 Rust 库，用于演示库开发的最佳实践。
//!
//! ## 功能特性
//!
//! - 使用 `thiserror` 的简洁错误处理
//! - 全面的测试覆盖
//! - 文档完善的公共 API
//!
//! ## 使用方法
//!
//! ```rust
//! use my_library::process_data;
//!
//! let result = process_data("hello").unwrap();
//! assert_eq!(result, "HELLO");
//! ```
```

## 代码质量指南

### 格式化

使用 `rustfmt` 保持一致的代码格式：

```bash
cargo fmt
```

### Lint 检查

使用 `clippy` 进行额外的 lint 检查：

```bash
cargo clippy -- -D warnings
```

### 提交前检查

提交代码前，运行以下命令：

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps
```

## Cargo.toml 最佳实践

```toml
[package]
name = "your_library_name"
version = "0.1.0"
edition = "2021"
rust-version = "1.92"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
description = "库的简要描述"
repository = "https://github.com/yourusername/your_library"
keywords = ["keyword1", "keyword2"]
categories = ["category1"]
readme = "README.md"

[dependencies]
thiserror = "1.0"

[dev-dependencies]
# 在此添加仅用于测试的依赖

[features]
default = []
# 定义可选功能
```

## 发布检查清单

发布到 crates.io 之前：

1. [ ] 更新 `Cargo.toml` 中的版本号
2. [ ] 更新 `CHANGELOG.md`
3. [ ] 运行 `cargo fmt`
4. [ ] 运行 `cargo clippy -- -D warnings`
5. [ ] 运行 `cargo test --all-features`
6. [ ] 运行 `cargo doc --no-deps`（确保没有警告）
7. [ ] 验证 `cargo publish --dry-run` 成功
8. [ ] 使用 `cargo publish` 发布

## 贡献指南

1. 遵循 Rust 命名约定（变量使用 snake_case，类型使用 CamelCase）
2. 为所有新功能编写测试
3. 为所有公共 API 编写文档
4. 保持公共 API 稳定（使用 SemVer）
5. 提交 PR 前运行 `cargo test`
6. 确保所有 clippy 警告都已解决

## 更多资源

- [Rust 程序设计语言](https://doc.rust-lang.org/book/)
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/)
- [thiserror 文档](https://docs.rs/thiserror/)
- [Rust 测试指南](https://doc.rust-lang.org/book/ch11-00-testing.html)
