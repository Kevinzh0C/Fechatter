#!/bin/bash
# deploy-fly.sh - Fly.io 快速部署脚本 (HR演示)

set -e

echo "✈️ 开始部署 Fechatter 到 Fly.io (HR演示环境)..."

# 检查 flyctl
if ! command -v flyctl &> /dev/null; then
    echo "❌ flyctl 未安装"
    echo "请安装: curl -L https://fly.io/install.sh | sh"
    exit 1
fi

# 检查登录状态
if ! flyctl auth whoami &> /dev/null; then
    echo "❌ 未登录 Fly.io"
    echo "请登录: flyctl auth login"
    exit 1
fi

echo "✅ Fly.io 认证成功"

# 应用配置
APP_NAME="fechatter-demo"
REGION="nrt"  # Tokyo

echo "📋 部署配置:"
echo "  应用名: $APP_NAME"
echo "  区域: $REGION (Tokyo)"
echo "  目标: HR技术演示"

# 检查应用是否存在
if flyctl apps list | grep -q "$APP_NAME"; then
    echo "📱 应用 $APP_NAME 已存在，将进行更新部署"
else
    echo "🆕 创建新应用 $APP_NAME"
    flyctl apps create "$APP_NAME" --org personal
fi

# 创建存储卷（如果不存在）
if ! flyctl volumes list -a "$APP_NAME" | grep -q "fechatter_data"; then
    echo "💾 创建持久化存储卷..."
    flyctl volumes create fechatter_data \
        --region "$REGION" \
        --size 1 \
        --app "$APP_NAME"
fi

# 设置密钥
echo "🔐 配置应用密钥..."
flyctl secrets set \
    JWT_SECRET="demo-jwt-secret-for-hr-presentation-$(date +%s)" \
    ENVIRONMENT="demo" \
    --app "$APP_NAME" > /dev/null 2>&1

# 部署应用
echo "🚀 开始部署应用..."
flyctl deploy --config fly.toml --app "$APP_NAME"

# 检查部署状态
echo "🔍 检查部署状态..."
flyctl status --app "$APP_NAME"

# 获取应用URL
APP_URL=$(flyctl info --app "$APP_NAME" | grep "Hostname" | awk '{print $2}')

if [ -n "$APP_URL" ]; then
    echo ""
    echo "🎉 HR演示环境部署成功！"
    echo ""
    echo "📋 部署信息:"
    echo "  应用名: $APP_NAME"
    echo "  区域: $REGION (Tokyo)"
    echo "  URL: https://$APP_URL"
    echo ""
    echo "👥 演示账户:"
    echo "  demo_admin / demo123 (管理员)"
    echo "  alice_dev / demo123 (开发)"
    echo "  bob_designer / demo123 (设计)"
    echo "  charlie_pm / demo123 (产品)"
    echo "  diana_qa / demo123 (测试)"
    echo ""
    echo "🔗 演示链接:"
    echo "  主页: https://$APP_URL"
    echo "  API演示: https://$APP_URL/api/demo"
    echo "  用户列表: https://$APP_URL/api/users"
    echo "  健康检查: https://$APP_URL/health"
    echo ""
    echo "💡 HR展示要点:"
    echo "  ✅ 现代化技术栈 (Rust + Vue.js)"
    echo "  ✅ 云原生部署 (Docker + Fly.io)"
    echo "  ✅ 实时通信能力 (WebSocket)"
    echo "  ✅ RESTful API 设计"
    echo "  ✅ 自动化CI/CD"
    echo ""
    echo "🎯 成本优化:"
    echo "  自动休眠: 不使用时成本接近$0"
    echo "  按需扩容: 访问时自动启动"
    echo "  预计月成本: $0-5"
else
    echo "⚠️ 无法获取应用URL，请检查部署状态"
fi

# 显示日志（最后几行）
echo ""
echo "📊 最近日志:"
flyctl logs --app "$APP_NAME" -n 10

echo ""
echo "🔧 管理命令:"
echo "  查看状态: flyctl status -a $APP_NAME"
echo "  查看日志: flyctl logs -a $APP_NAME"
echo "  打开应用: flyctl open -a $APP_NAME"
echo "  SSH连接: flyctl ssh console -a $APP_NAME"
echo ""
echo "✨ 演示环境已就绪，可以向HR展示了！"