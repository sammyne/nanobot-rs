#!/bin/bash

# 统计 crates 目录下除测试代码外的代码行数
# 测试代码：独立的 tests.rs 文件

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$SCRIPT_DIR/crates"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "代码行数统计（排除 tests.rs）"
echo "=========================================="

# 统计所有 .rs 文件的总行数
total_lines=$(find "$SOURCE_DIR" -name "*.rs" -exec cat {} \; | wc -l)

# 统计 tests.rs 文件的行数
test_lines=$(find "$SOURCE_DIR" -name "tests.rs" -exec cat {} \; 2>/dev/null | wc -l || echo 0)

# 计算非测试代码行数
non_test_lines=$((total_lines - test_lines))

echo ""
echo -e "总代码行数:     ${YELLOW}$total_lines${NC}"
echo -e "tests.rs 行数:  ${YELLOW}$test_lines${NC}"
echo ""
echo "=========================================="
echo -e "非测试代码行数: ${GREEN}$non_test_lines${NC}"
echo "=========================================="

# 显示各 crate 的统计
echo ""
echo "各 crate 代码行数统计："
for crate_dir in "$SOURCE_DIR"/*/; do
    [ -d "$crate_dir" ] || continue
    crate_name=$(basename "$crate_dir")
    crate_total=$(find "$crate_dir" -name "*.rs" -exec cat {} \; 2>/dev/null | wc -l || echo 0)
    crate_tests=$(find "$crate_dir" -name "tests.rs" -exec cat {} \; 2>/dev/null | wc -l || echo 0)
    crate_non_test=$((crate_total - crate_tests))
    printf "  %-15s: %5d 行\n" "$crate_name" "$crate_non_test"
done
