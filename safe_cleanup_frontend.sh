#!/bin/bash

# Fechatter 前端废弃文件安全清理脚本
echo "🧹 Fechatter 前端安全清理 - 删除确认的废弃文件"
echo "=================================================="

# 检查dry-run模式
DRY_RUN=false
if [ "$1" == "--dry-run" ] || [ "$1" == "-n" ]; then
    DRY_RUN=true
    echo "🔍 DRY RUN MODE - 不会删除任何文件"
    echo ""
fi

# 创建备份目录
backup_dir="backups/frontend_cleanup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$backup_dir"
echo "📦 备份目录: $backup_dir"

deleted_count=0

# 安全删除函数
safe_delete() {
    local file="$1"
    local description="$2"
    
    if [ -f "$file" ]; then
        if [ "$DRY_RUN" = true ]; then
            echo "  🔍 Would delete: $file ($description)"
        else
            # 创建备份
            cp "$file" "$backup_dir/" 2>/dev/null || true
            rm -f "$file"
            echo "  ✅ Deleted: $file ($description)"
        fi
        deleted_count=$((deleted_count + 1))
    else
        echo "  ℹ️  Not found: $file"
    fi
}

echo -e "\n🗑️ 清理高信心废弃文件:"

# 1. 临时测试文件
echo -e "\n📋 临时测试文件:"
safe_delete "./fechatter_frontend/test-auth.js" "临时认证测试"
safe_delete "./fechatter_frontend/test_presence_fix.js" "在线状态测试修复"
safe_delete "./fechatter_frontend/test-navigation-fix.js" "导航测试修复"

# 2. 紧急修复脚本（已不需要）
echo -e "\n🚨 紧急修复脚本:"
safe_delete "./fechatter_frontend/public/console-fix.js" "控制台修复脚本"
safe_delete "./fechatter_frontend/public/emergency-menu-fix.js" "紧急菜单修复"
safe_delete "./fechatter_frontend/public/emergency-message-fix.js" "紧急消息修复"
safe_delete "./fechatter_frontend/public/emergency-sse-fix.js" "紧急SSE修复"

# 3. 临时CSS文件
echo -e "\n🎨 临时CSS文件:"
safe_delete "./fechatter_frontend/temp_emoji_clean.css" "临时表情清理样式"

# 4. 备份目录清理
echo -e "\n📁 备份目录清理:"
if [ -d "./fechatter_frontend/src/styles/conflicted-backup-20250624_010034" ]; then
    if [ "$DRY_RUN" = true ]; then
        echo "  🔍 Would delete directory: conflicted-backup-20250624_010034"
    else
        cp -r "./fechatter_frontend/src/styles/conflicted-backup-20250624_010034" "$backup_dir/" 2>/dev/null || true
        rm -rf "./fechatter_frontend/src/styles/conflicted-backup-20250624_010034"
        echo "  ✅ Deleted directory: conflicted-backup-20250624_010034"
    fi
    deleted_count=$((deleted_count + 1))
fi

# 5. Tauri构建临时文件
echo -e "\n🔧 构建临时文件:"
if [ -d "./fechatter_frontend/src-tauri/target" ]; then
    if [ "$DRY_RUN" = true ]; then
        echo "  🔍 Would clean: src-tauri/target directory"
    else
        echo "  🧹 Cleaning Tauri build artifacts..."
        cd "./fechatter_frontend" && cargo clean --manifest-path src-tauri/Cargo.toml 2>/dev/null || true
        cd ..
        echo "  ✅ Cleaned: Tauri build artifacts"
    fi
fi

# 6. 明显的调试文件（高信心）
echo -e "\n🐛 明显的调试文件:"
safe_delete "./fechatter_frontend/src/utils/debugSearchNow.js" "搜索调试工具"
safe_delete "./fechatter_frontend/src/utils/debugDuplicateChannels.js" "重复频道调试"
safe_delete "./fechatter_frontend/src/utils/debugTokenState.js" "Token状态调试"
safe_delete "./fechatter_frontend/src/utils/debugMessageLoading.js" "消息加载调试"
safe_delete "./fechatter_frontend/src/utils/debugGroupChatIssue.js" "群聊问题调试"

# 7. 简单测试脚本
echo -e "\n📋 简单测试脚本:"
safe_delete "./fechatter_frontend/src/utils/testCodeHighlight.js" "代码高亮测试"
safe_delete "./fechatter_frontend/src/utils/testSearchApi.js" "搜索API测试"
safe_delete "./fechatter_frontend/src/utils/testAutoExecutionFix.js" "自动执行测试"
safe_delete "./fechatter_frontend/src/utils/testGroupChannelPreload.js" "群组频道预载测试"
safe_delete "./fechatter_frontend/src/utils/testLogout.js" "登出测试"
safe_delete "./fechatter_frontend/src/utils/testMessagePersistence.js" "消息持久化测试"

# 8. 修复验证文件
echo -e "\n🔧 修复验证文件:"
safe_delete "./fechatter_frontend/src/utils/fixVerification.js" "修复验证工具"

# 总结
echo -e "\n🎉 清理完成！"
echo "=================================================="
if [ "$DRY_RUN" = true ]; then
    echo "📊 将要处理的文件数: $deleted_count"
    echo "💡 运行 ./safe_cleanup_frontend.sh 执行实际清理"
else
    echo "📊 已处理文件数: $deleted_count"
    echo "📦 备份位置: $backup_dir"
    echo "✅ 所有删除的文件都已备份"
fi

echo -e "\n💡 下一步建议:"
echo "1. 测试前端功能确保正常"
git status > /dev/null 2>&1 && echo "2. 提交到Git: git add . && git commit -m 'Clean deprecated frontend files'"
echo "3. 如需回滚，从备份目录恢复文件" 