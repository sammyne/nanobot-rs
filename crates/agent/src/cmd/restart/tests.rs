// RestartCmd 的实际行为（spawn 新进程 + exit）不适合在单元测试中验证。
// 命令注册和分发的测试在 loop/tests.rs 中覆盖。
