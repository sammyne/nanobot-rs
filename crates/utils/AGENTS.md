# utils crate

通用工具函数（字符串处理等）。

## 关键类型

纯函数 API，无结构体：

- `strings::truncate(s, max_chars) -> Option<&str>` -- 在字符边界安全截断字符串
- `strings::redact(s) -> String` -- 遮蔽敏感字符串（如 API key）用于日志输出
- `strings::estimate_tokens(text) -> usize` -- 基于字节长度估算 token 数（1 token ≈ 4 字节）

## 内部依赖

无（叶子 crate）
