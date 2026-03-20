#!/bin/bash

# 统计 crates 目录下除测试代码函数外的代码行数
# 排除 channels、cli 和 providers 三个 crate

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$SCRIPT_DIR/crates"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "代码行数统计（排除测试函数）"
echo "=========================================="

# 排除的 crates
EXCLUDED_CRATES="channels|nanobot|provider"

# 统计非测试代码行数的函数
count_non_test_lines() {
    local dir="$1"
    local total=0
    
    # 查找所有 .rs 文件，排除指定 crate、tests.rs 文件和 tests 文件夹
    while IFS= read -r -d '' file; do
        # 读取文件并统计非测试代码行
        local in_test_block=0
        while IFS= read -r line; do
            # 检测 #[test] 或 #[cfg(test)]
            if [[ "$line" =~ ^#\[test\] ]] || [[ "$line" =~ ^#\[cfg\(test\)\] ]]; then
                in_test_block=1
                continue
            fi
            
            # 检测测试模块结束 (通过遇到非空且非注释的行来判断)
            if [ $in_test_block -eq 1 ]; then
                if [[ ! "$line" =~ ^[[:space:]]*$ ]] && [[ ! "$line" =~ ^[[:space:]]*// ]] && [[ ! "$line" =~ ^[[:space:]]*(pub\s+)?(mod|fn|struct|enum|impl|trait|type) ]]; then
                    # 如果是其他非空、非注释的代码行，可能还在测试块中
                    # 只有遇到新的声明才认为测试块结束
                    if [[ "$line" =~ ^[[:space:]]*(pub\s+)?(mod|fn|struct|enum|impl|trait|type) ]] && [[ ! "$line" =~ test ]]; then
                        in_test_block=0
                    else
                        continue
                    fi
                elif [[ "$line" =~ ^[[:space:]]*(pub\s+)?(mod|fn|struct|enum|impl|trait|type) ]] && [[ ! "$line" =~ test ]]; then
                    in_test_block=0
                else
                    continue
                fi
            fi
            
            # 如果不在测试块中，统计行数
            if [ $in_test_block -eq 0 ]; then
                ((total++))
            fi
        done < "$file"
    done < <(find "$dir" -name "*.rs" -type f | grep -vE "crates/($EXCLUDED_CRATES)" | grep -v "tests.rs" | grep -v "tests/" | tr '\n' '\0')
    
    echo "$total"
}

# 统计所有 crate 的非测试代码行数
total_non_test_lines=$(count_non_test_lines "$SOURCE_DIR")

echo ""
echo -e "非测试代码行数（排除 channels、nanobot、provider）: ${GREEN}$total_non_test_lines${NC}"
echo "=========================================="

# 显示各 crate 的统计（排除指定 crate）
echo ""
echo "各 crate 代码行数统计（排除测试函数）："
for crate_dir in "$SOURCE_DIR"/*/; do
    [ -d "$crate_dir" ] || continue
    crate_name=$(basename "$crate_dir")
    
    # 跳过排除的 crates
    if [[ "$crate_name" =~ ^($EXCLUDED_CRATES)$ ]]; then
        printf "  %-15s: %5s (已排除)\n" "$crate_name" "---"
        continue
    fi
    
    crate_lines=$(count_non_test_lines "$crate_dir")
    printf "  %-15s: %5d 行\n" "$crate_name" "$crate_lines"
done
