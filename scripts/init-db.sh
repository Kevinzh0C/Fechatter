#!/bin/bash
# init-db.sh - 数据库初始化和迁移脚本

set -e

echo "🗄️  开始数据库初始化..."

# 设置数据库连接
export DATABASE_URL=${DATABASE_URL:-"postgres://postgres:postgres@postgres:5432/fechatter"}

# 等待 PostgreSQL 就绪
echo "等待 PostgreSQL 连接..."
until pg_isready -d "$DATABASE_URL"; do
  echo "等待数据库连接 - 休眠"
  sleep 2
done

echo "✅ PostgreSQL 连接成功"

# 检查是否需要运行迁移
echo "检查数据库迁移状态..."

# 尝试运行迁移
if command -v sqlx >/dev/null 2>&1; then
  echo "使用 sqlx 运行迁移..."
  sqlx migrate run --database-url "$DATABASE_URL"
  echo "✅ 数据库迁移完成"
else
  echo "⚠️  sqlx 不可用，跳过自动迁移"
  echo "请确保在应用启动时运行迁移"
fi

# 验证关键表是否存在
echo "验证数据库表..."
psql "$DATABASE_URL" -c "\dt" | grep -E "(users|chats|messages)" && echo "✅ 核心表存在" || echo "⚠️  核心表可能缺失"

echo "🎉 数据库初始化完成"