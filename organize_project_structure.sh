#!/bin/bash

# Fechatter 项目结构整理脚本
echo "🗂️ Fechatter 项目结构整理 - 删除临时文件并归类整理"
echo "============================================================="

# 检查干燥运行模式
DRY_RUN=false
if [ "$1" == "--dry-run" ] || [ "$1" == "-n" ]; then
    DRY_RUN=true
    echo "🔍 DRY RUN MODE - 不会实际修改文件"
    echo ""
fi

# 创建备份
backup_dir="backups/structure_cleanup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$backup_dir"
echo "📦 备份目录: $backup_dir"

# 统计
deleted_count=0
moved_count=0

# 安全删除函数
safe_delete() {
    local file="$1"
    local description="$2"
    
    if [ -f "$file" ] || [ -d "$file" ]; then
        if [ "$DRY_RUN" = true ]; then
            echo "  🔍 Would delete: $file ($description)"
        else
            cp -r "$file" "$backup_dir/" 2>/dev/null || true
            rm -rf "$file"
            echo "  ✅ Deleted: $file ($description)"
        fi
        deleted_count=$((deleted_count + 1))
    fi
}

# 安全移动函数
safe_move() {
    local src="$1"
    local dest="$2"
    local description="$3"
    
    if [ -f "$src" ]; then
        if [ "$DRY_RUN" = true ]; then
            echo "  🔍 Would move: $src → $dest ($description)"
        else
            mkdir -p "$(dirname "$dest")"
            cp "$src" "$backup_dir/" 2>/dev/null || true
            mv "$src" "$dest"
            echo "  ✅ Moved: $src → $dest ($description)"
        fi
        moved_count=$((moved_count + 1))
    fi
}

echo -e "\n🗑️ 第一阶段：删除临时和废弃文件"

# 1. 删除构建日志和临时文件
echo -e "\n📋 构建日志和临时文件:"
safe_delete "./build-complete-x86.log" "构建日志"
safe_delete "./build-x86.log" "构建日志"
safe_delete "./build.log" "构建日志"
safe_delete "./debug_logs.txt" "调试日志"
safe_delete "./test.txt" "测试文件"

# 2. 删除临时分析目录
echo -e "\n📊 临时分析目录:"
safe_delete "./analysis_20250630_000951" "废弃分析目录"

# 3. 删除我们创建的临时脚本
echo -e "\n🔧 临时脚本:"
safe_delete "./analyze_deprecated_assets.sh" "临时分析脚本"
safe_delete "./cleanup_deprecated_html.sh" "临时清理脚本"
safe_delete "./safe_cleanup_frontend.sh" "临时清理脚本"

# 4. 删除测试数据文件
echo -e "\n🧪 测试数据文件:"
safe_delete "./employee_ids.txt" "员工ID测试数据"
safe_delete "./europe_channel.json" "欧洲频道测试数据"
safe_delete "./japan_channel.json" "日本频道测试数据"
safe_delete "./group_chat.json" "群聊测试数据"
safe_delete "./private_channel.json" "私有频道测试数据"
safe_delete "./public_channel.json" "公共频道测试数据"
safe_delete "./single_chat.json" "单聊测试数据"
safe_delete "./super_user.json" "超级用户测试数据"

echo -e "\n📁 第二阶段：归类整理文件"

# 确保目录存在
if [ "$DRY_RUN" = false ]; then
    mkdir -p scripts/build
    mkdir -p scripts/deployment
    mkdir -p scripts/utils
    mkdir -p config/docker
    mkdir -p config/test-data
fi

# 5. 移动构建脚本
echo -e "\n🔨 构建脚本归类:"
safe_move "./build-cross.sh" "scripts/build/build-cross.sh" "跨平台构建脚本"
safe_move "./build-individual.sh" "scripts/build/build-individual.sh" "单独构建脚本"
safe_move "./build-local.sh" "scripts/build/build-local.sh" "本地构建脚本"
safe_move "./build-musl.sh" "scripts/build/build-musl.sh" "MUSL构建脚本"
safe_move "./manual-build-x86.sh" "scripts/build/manual-build-x86.sh" "手动X86构建"

# 6. 移动部署脚本
echo -e "\n🚀 部署脚本归类:"
safe_move "./deploy-fechatter-server.sh" "scripts/deployment/deploy-fechatter-server.sh" "服务器部署脚本"
safe_move "./global-health-check.sh" "scripts/deployment/global-health-check.sh" "健康检查脚本"

# 7. 移动工具脚本
echo -e "\n🛠️ 工具脚本归类:"
safe_move "./bulk_search_sync.sh" "scripts/utils/bulk_search_sync.sh" "批量搜索同步"
safe_move "./filter_dates_fixed.sh" "scripts/utils/filter_dates_fixed.sh" "日期过滤脚本"
safe_move "./filter_dates.sh" "scripts/utils/filter_dates.sh" "日期过滤脚本"
safe_move "./filter_env.sh" "scripts/utils/filter_env.sh" "环境过滤脚本"
safe_move "./fix_search_routing.sh" "scripts/utils/fix_search_routing.sh" "搜索路由修复"
safe_move "./final-sse-complete-test.sh" "scripts/utils/final-sse-complete-test.sh" "SSE完整测试"
safe_move "./fix-sse-notify-config.sh" "scripts/utils/fix-sse-notify-config.sh" "SSE通知配置修复"

# 8. 移动Docker配置文件
echo -e "\n🐳 Docker配置归类:"
safe_move "./docker-compose.local.yml" "config/docker/docker-compose.local.yml" "本地Docker配置"
safe_move "./docker-compose.prod.yml" "config/docker/docker-compose.prod.yml" "生产Docker配置"
safe_move "./docker-compose.vcr.yml" "config/docker/docker-compose.vcr.yml" "VCR Docker配置"
safe_move "./gateway-docker-fixed.yml" "config/docker/gateway-docker-fixed.yml" "网关Docker配置"
safe_move "./gateway-vcr.yml" "config/docker/gateway-vcr.yml" "网关VCR配置"

# 9. 移动服务配置文件
echo -e "\n⚙️ 服务配置归类:"
safe_move "./chat_fixed.yml" "config/chat_fixed.yml" "修复的聊天配置"
safe_move "./chat.yml" "config/chat.yml" "聊天服务配置"
safe_move "./notify.yml" "config/notify.yml" "通知服务配置"

# 10. 清理旧备份
echo -e "\n🧹 清理旧备份:"
if [ -d "./backups" ]; then
    old_backups=$(find ./backups -name "*cleanup*" -mtime +7 -type d 2>/dev/null | wc -l | tr -d ' ')
    if [ "$old_backups" -gt 0 ]; then
        if [ "$DRY_RUN" = true ]; then
            echo "  🔍 Would clean $old_backups old backup directories"
        else
            find ./backups -name "*cleanup*" -mtime +7 -type d -exec rm -rf {} \; 2>/dev/null || true
            echo "  ✅ Cleaned $old_backups old backup directories"
        fi
    else
        echo "  ℹ️  No old backups to clean"
    fi
fi

# 11. 创建目录说明文件
if [ "$DRY_RUN" = false ]; then
    cat > scripts/README.md << 'EOF'
# Scripts Directory

## 📁 目录结构

### build/
构建相关脚本
- `build-cross.sh` - 跨平台构建
- `build-local.sh` - 本地构建
- `build-musl.sh` - MUSL静态链接构建

### deployment/
部署相关脚本
- `deploy-fechatter-server.sh` - 服务器部署
- `global-health-check.sh` - 健康检查

### utils/
工具脚本
- `bulk_search_sync.sh` - 批量搜索同步
- `filter_*.sh` - 各种过滤工具
- `fix_*.sh` - 修复工具

## 🚀 使用方法

所有脚本都应该从项目根目录执行：
```bash
# 构建
./scripts/build/build-local.sh

# 部署
./scripts/deployment/deploy-fechatter-server.sh

# 工具
./scripts/utils/bulk_search_sync.sh
```
EOF

    cat > config/README.md << 'EOF'
# Configuration Directory

## 📁 目录结构

### 服务配置
- `chat.yml` - 聊天服务配置
- `notify.yml` - 通知服务配置
- `chat_fixed.yml` - 修复版聊天配置

### docker/
Docker相关配置
- `docker-compose.*.yml` - 各环境Docker编排
- `gateway-*.yml` - 网关配置

## ⚙️ 使用说明

配置文件按环境和服务分类，方便管理和部署。
EOF

    echo "  📄 Created directory documentation"
fi

# 总结
echo -e "\n🎉 整理完成！"
echo "============================================================="
if [ "$DRY_RUN" = true ]; then
    echo "📊 预计删除文件: $deleted_count"
    echo "📊 预计移动文件: $moved_count"
    echo "💡 运行 ./organize_project_structure.sh 执行实际整理"
else
    echo "📊 已删除文件: $deleted_count"
    echo "📊 已移动文件: $moved_count"
    echo "📦 备份位置: $backup_dir"
fi

echo -e "\n📁 新的目录结构:"
echo "├── scripts/"
echo "│   ├── build/       (构建脚本)"
echo "│   ├── deployment/  (部署脚本)"
echo "│   └── utils/       (工具脚本)"
echo "├── config/"
echo "│   ├── docker/      (Docker配置)"
echo "│   └── *.yml        (服务配置)"
echo "└── backups/         (备份文件)"

if [ "$DRY_RUN" = false ]; then
    echo -e "\n💡 下一步建议:"
    echo "1. 测试项目功能确保正常"
    echo "2. 提交到Git: git add . && git commit -m 'Organize project structure'"
    echo "3. 更新相关文档中的脚本路径"
fi 