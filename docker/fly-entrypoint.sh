#!/bin/sh
# Fly.io 启动脚本

set -e

echo "🚀 Starting Fechatter Demo for Fly.io..."

# 检查数据目录
if [ ! -d "/data" ]; then
    mkdir -p /data
fi

# 初始化 SQLite 数据库（如果不存在）
if [ ! -f "/data/fechatter.db" ]; then
    echo "📊 Initializing demo database..."
    sqlite3 /data/fechatter.db < /app/demo-setup.sql
    echo "✅ Demo database initialized with sample data"
fi

# 设置环境变量
export DATABASE_URL="sqlite:///data/fechatter.db"
export REDIS_URL="memory://"
export ENVIRONMENT="demo"
export RUST_LOG="info"

# 健康检查
echo "🔍 Running health checks..."

# 检查数据库
if sqlite3 /data/fechatter.db "SELECT COUNT(*) FROM users;" > /dev/null 2>&1; then
    echo "✅ Database is ready"
else
    echo "❌ Database check failed"
    exit 1
fi

# 启动应用
echo "🎯 Starting Fechatter demo application..."
echo "🌐 Demo will be available on port 8080"
echo "👥 Demo users: demo_admin, alice_dev, bob_designer, charlie_pm, diana_qa"
echo "🔑 Default password for all demo users: demo123"

exec /app/fechatter