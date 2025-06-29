#!/bin/bash

# Fechatter 废弃资源智能分析工具
echo "🔍 Fechatter 废弃资源分析 - 安全识别JS/CSS/SH文件"
echo "=================================================================="

# 分析结果目录
ANALYSIS_DIR="analysis_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$ANALYSIS_DIR"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "📁 分析结果将保存到: $ANALYSIS_DIR"

# 1. 查找疑似废弃文件
echo -e "\n${BLUE}=== 第一阶段：基于文件名模式识别 ===${NC}"

find_suspicious_files() {
    local extension="$1"
    local description="$2"
    
    echo -e "\n🔍 查找疑似废弃的 $description 文件:"
    
    find . -name "*.$extension" -type f \
        -not -path "./target/*" \
        -not -path "./node_modules/*" \
        -not -path "./.venv/*" \
        -not -path "./fechatter_frontend/node_modules/*" \
        | grep -E "(test|debug|fix|backup|old|temp|unused|deprecated|mock|demo)" \
        > "$ANALYSIS_DIR/suspicious_${extension}_files.txt"
    
    if [ -s "$ANALYSIS_DIR/suspicious_${extension}_files.txt" ]; then
        local count=$(wc -l < "$ANALYSIS_DIR/suspicious_${extension}_files.txt")
        echo -e "${YELLOW}  发现 $count 个疑似文件${NC}"
        
        # 显示前10个
        echo "  预览:"
        head -10 "$ANALYSIS_DIR/suspicious_${extension}_files.txt" | sed 's/^/    /'
        
        if [ $count -gt 10 ]; then
            echo "    ... 还有 $((count - 10)) 个文件"
        fi
    else
        echo -e "${GREEN}  未发现疑似文件${NC}"
    fi
}

# 分别查找各类文件
find_suspicious_files "js" "JavaScript"
find_suspicious_files "css" "CSS"
find_suspicious_files "sh" "Shell脚本"

# 2. 深度内容分析
echo -e "\n${BLUE}=== 第二阶段：内容特征分析 ===${NC}"

analyze_js_content() {
    echo -e "\n📊 JavaScript 内容分析:"
    
    # 查找包含测试/调试标记的JS文件
    find . -name "*.js" -type f \
        -not -path "./target/*" \
        -not -path "./node_modules/*" \
        -not -path "./.venv/*" \
        -not -path "./fechatter_frontend/node_modules/*" \
        -exec grep -l -E "(console\.log.*DEBUG|TODO.*remove|FIXME|// TEST|\/\* DEBUG|\/\/ TEMP)" {} \; \
        > "$ANALYSIS_DIR/js_with_debug_content.txt"
    
    if [ -s "$ANALYSIS_DIR/js_with_debug_content.txt" ]; then
        local count=$(wc -l < "$ANALYSIS_DIR/js_with_debug_content.txt")
        echo -e "${YELLOW}  发现 $count 个包含调试标记的JS文件${NC}"
        cat "$ANALYSIS_DIR/js_with_debug_content.txt" | sed 's/^/    /'
    fi
    
    # 查找只有简单测试代码的JS文件
    find . -name "*.js" -type f \
        -not -path "./target/*" \
        -not -path "./node_modules/*" \
        -not -path "./.venv/*" \
        -not -path "./fechatter_frontend/node_modules/*" \
        | while read file; do
            lines=$(wc -l < "$file")
            if [ $lines -lt 50 ]; then
                if grep -q -E "(alert\(|confirm\(|window\.open\(.*test)" "$file"; then
                    echo "$file (${lines} lines)" >> "$ANALYSIS_DIR/simple_test_js.txt"
                fi
            fi
        done
    
    if [ -s "$ANALYSIS_DIR/simple_test_js.txt" ]; then
        local count=$(wc -l < "$ANALYSIS_DIR/simple_test_js.txt")
        echo -e "${YELLOW}  发现 $count 个简单测试JS文件${NC}"
        cat "$ANALYSIS_DIR/simple_test_js.txt" | sed 's/^/    /'
    fi
}

analyze_css_content() {
    echo -e "\n🎨 CSS 内容分析:"
    
    # 查找包含测试标记的CSS文件
    find . -name "*.css" -type f \
        -not -path "./target/*" \
        -not -path "./node_modules/*" \
        -not -path "./.venv/*" \
        -not -path "./fechatter_frontend/node_modules/*" \
        -exec grep -l -E "(\/\* TEST|\/\* DEBUG|\/\* TEMP|\.test-|\.debug-)" {} \; \
        > "$ANALYSIS_DIR/css_with_debug_content.txt"
    
    if [ -s "$ANALYSIS_DIR/css_with_debug_content.txt" ]; then
        local count=$(wc -l < "$ANALYSIS_DIR/css_with_debug_content.txt")
        echo -e "${YELLOW}  发现 $count 个包含调试标记的CSS文件${NC}"
        cat "$ANALYSIS_DIR/css_with_debug_content.txt" | sed 's/^/    /'
    fi
    
    # 查找空的或几乎空的CSS文件
    find . -name "*.css" -type f \
        -not -path "./target/*" \
        -not -path "./node_modules/*" \
        -not -path "./.venv/*" \
        -not -path "./fechatter_frontend/node_modules/*" \
        | while read file; do
            lines=$(wc -l < "$file")
            if [ $lines -lt 10 ]; then
                echo "$file (${lines} lines)" >> "$ANALYSIS_DIR/empty_css.txt"
            fi
        done
    
    if [ -s "$ANALYSIS_DIR/empty_css.txt" ]; then
        local count=$(wc -l < "$ANALYSIS_DIR/empty_css.txt")
        echo -e "${YELLOW}  发现 $count 个空的或很小的CSS文件${NC}"
        cat "$ANALYSIS_DIR/empty_css.txt" | sed 's/^/    /'
    fi
}

analyze_sh_content() {
    echo -e "\n🐚 Shell脚本分析:"
    
    # 查找包含临时/测试标记的Shell脚本
    find . -name "*.sh" -type f \
        -not -path "./target/*" \
        -not -path "./node_modules/*" \
        -not -path "./.venv/*" \
        -not -path "./fechatter_frontend/node_modules/*" \
        -exec grep -l -E "(# TEST|# DEBUG|# TEMP|echo.*test|TODO.*remove)" {} \; \
        > "$ANALYSIS_DIR/sh_with_debug_content.txt"
    
    if [ -s "$ANALYSIS_DIR/sh_with_debug_content.txt" ]; then
        local count=$(wc -l < "$ANALYSIS_DIR/sh_with_debug_content.txt")
        echo -e "${YELLOW}  发现 $count 个包含调试标记的Shell脚本${NC}"
        cat "$ANALYSIS_DIR/sh_with_debug_content.txt" | sed 's/^/    /'
    fi
}

# 执行内容分析
analyze_js_content
analyze_css_content  
analyze_sh_content

# 3. 依赖关系分析
echo -e "\n${BLUE}=== 第三阶段：依赖关系分析 ===${NC}"

analyze_dependencies() {
    echo -e "\n🔗 分析文件引用关系:"
    
    # 创建引用分析文件
    > "$ANALYSIS_DIR/file_references.txt"
    
    # 分析每个疑似文件是否被引用
    for ext in js css sh; do
        if [ -s "$ANALYSIS_DIR/suspicious_${ext}_files.txt" ]; then
            echo "=== $ext 文件引用分析 ===" >> "$ANALYSIS_DIR/file_references.txt"
            
            while read file; do
                filename=$(basename "$file")
                filename_no_ext="${filename%.*}"
                
                # 搜索引用
                references=$(find . -type f \
                    -not -path "./target/*" \
                    -not -path "./node_modules/*" \
                    -not -path "./.venv/*" \
                    -not -path "./fechatter_frontend/node_modules/*" \
                    -name "*.html" -o -name "*.vue" -o -name "*.js" -o -name "*.ts" \
                    -exec grep -l "$filename\|$filename_no_ext" {} \; 2>/dev/null | wc -l)
                
                echo "$file: $references 个引用" >> "$ANALYSIS_DIR/file_references.txt"
                
                if [ $references -eq 0 ]; then
                    echo "$file" >> "$ANALYSIS_DIR/unreferenced_files.txt"
                fi
                
            done < "$ANALYSIS_DIR/suspicious_${ext}_files.txt"
        fi
    done
    
    if [ -s "$ANALYSIS_DIR/unreferenced_files.txt" ]; then
        local count=$(wc -l < "$ANALYSIS_DIR/unreferenced_files.txt")
        echo -e "${RED}  发现 $count 个未被引用的疑似废弃文件${NC}"
        echo "  详细列表已保存到: $ANALYSIS_DIR/unreferenced_files.txt"
    else
        echo -e "${GREEN}  所有疑似文件都有引用关系${NC}"
    fi
}

analyze_dependencies

# 4. 生成详细报告
echo -e "\n${BLUE}=== 第四阶段：生成分析报告 ===${NC}"

generate_report() {
    local report_file="$ANALYSIS_DIR/废弃文件分析报告.md"
    
    cat > "$report_file" << EOF
# Fechatter 废弃文件分析报告

生成时间: $(date)

## 📊 总览

EOF

    # 统计各类文件
    for ext in js css sh; do
        if [ -s "$ANALYSIS_DIR/suspicious_${ext}_files.txt" ]; then
            count=$(wc -l < "$ANALYSIS_DIR/suspicious_${ext}_files.txt")
            echo "- 疑似废弃的 ${ext^^} 文件: $count 个" >> "$report_file"
        fi
    done
    
    echo "" >> "$report_file"
    
    # 高风险文件（未被引用）
    if [ -s "$ANALYSIS_DIR/unreferenced_files.txt" ]; then
        echo "## 🚨 高风险废弃文件（建议优先清理）" >> "$report_file"
        echo "" >> "$report_file"
        echo "以下文件未找到任何引用，很可能是废弃文件：" >> "$report_file"
        echo "" >> "$report_file"
        while read file; do
            echo "- \`$file\`" >> "$report_file"
        done < "$ANALYSIS_DIR/unreferenced_files.txt"
        echo "" >> "$report_file"
    fi
    
    # 中风险文件
    if [ -s "$ANALYSIS_DIR/js_with_debug_content.txt" ] || [ -s "$ANALYSIS_DIR/css_with_debug_content.txt" ] || [ -s "$ANALYSIS_DIR/sh_with_debug_content.txt" ]; then
        echo "## ⚠️ 中风险文件（包含调试内容）" >> "$report_file"
        echo "" >> "$report_file"
        
        for content_file in js_with_debug_content css_with_debug_content sh_with_debug_content; do
            if [ -s "$ANALYSIS_DIR/${content_file}.txt" ]; then
                echo "### ${content_file%_*} 文件" >> "$report_file"
                while read file; do
                    echo "- \`$file\`" >> "$report_file"
                done < "$ANALYSIS_DIR/${content_file}.txt"
                echo "" >> "$report_file"
            fi
        done
    fi
    
    # 安全清理建议
    cat >> "$report_file" << EOF

## 🛡️ 安全清理建议

### 立即可删除（高信心）
1. 未被引用的test/debug/fix文件
2. 包含明确TODO remove标记的文件
3. 临时备份文件（.backup, .old后缀）

### 需要手动确认（中等信心）
1. 包含调试代码但可能有用的文件
2. 简单测试文件（可能用于快速验证）
3. 工具脚本（可能偶尔使用）

### 建议保留
1. 有引用关系的文件
2. 构建和部署相关脚本
3. 核心业务逻辑文件

## 📝 使用说明

1. 首先备份整个项目
\`\`\`bash
cp -r . ../fechatter_backup_\$(date +%Y%m%d)
\`\`\`

2. 从高风险文件开始清理
3. 每删除一批文件后测试项目功能
4. 保持Git提交记录以便回滚

EOF
    
    echo -e "${GREEN}📄 详细报告已生成: $report_file${NC}"
}

generate_report

# 5. 生成清理脚本
echo -e "\n${BLUE}=== 第五阶段：生成安全清理脚本 ===${NC}"

if [ -s "$ANALYSIS_DIR/unreferenced_files.txt" ]; then
    cat > "$ANALYSIS_DIR/safe_cleanup.sh" << 'EOF'
#!/bin/bash

# 自动生成的安全清理脚本
echo "🧹 Fechatter 安全清理 - 删除高信心废弃文件"
echo "================================================"

DRY_RUN=false
if [ "$1" == "--dry-run" ] || [ "$1" == "-n" ]; then
    DRY_RUN=true
    echo "🔍 DRY RUN MODE - 不会删除任何文件"
fi

# 创建备份
backup_dir="backups/safe_cleanup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$backup_dir"

deleted_count=0

EOF
    
    # 添加文件删除逻辑
    echo 'echo -e "\n🗑️ 删除未引用的废弃文件:"' >> "$ANALYSIS_DIR/safe_cleanup.sh"
    
    while read file; do
        echo "if [ -f \"$file\" ]; then" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "    if [ \"\$DRY_RUN\" = true ]; then" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "        echo \"  🔍 Would delete: $file\"" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "    else" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "        cp \"$file\" \"\$backup_dir/\" 2>/dev/null || true" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "        rm -f \"$file\"" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "        echo \"  ✅ Deleted: $file\"" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "    fi" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "    deleted_count=\$((deleted_count + 1))" >> "$ANALYSIS_DIR/safe_cleanup.sh"
        echo "fi" >> "$ANALYSIS_DIR/safe_cleanup.sh"
    done < "$ANALYSIS_DIR/unreferenced_files.txt"
    
    cat >> "$ANALYSIS_DIR/safe_cleanup.sh" << 'EOF'

echo -e "\n✅ 清理完成!"
if [ "$DRY_RUN" = true ]; then
    echo "📊 将要删除的文件数: $deleted_count"
    echo "💡 运行 ./safe_cleanup.sh 执行实际清理"
else
    echo "📊 已删除文件数: $deleted_count"
    echo "📦 备份位置: $backup_dir"
fi
EOF
    
    chmod +x "$ANALYSIS_DIR/safe_cleanup.sh"
    echo -e "${GREEN}🔧 安全清理脚本已生成: $ANALYSIS_DIR/safe_cleanup.sh${NC}"
    echo "   使用方法:"
    echo "   - 预览模式: ./$ANALYSIS_DIR/safe_cleanup.sh --dry-run"
    echo "   - 实际清理: ./$ANALYSIS_DIR/safe_cleanup.sh"
fi

echo -e "\n${GREEN}🎉 分析完成！${NC}"
echo "📁 所有分析结果保存在: $ANALYSIS_DIR"
echo "📖 请查看详细报告: $ANALYSIS_DIR/废弃文件分析报告.md" 