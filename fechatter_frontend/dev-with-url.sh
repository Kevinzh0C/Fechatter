#!/bin/bash

echo ""
echo "🚀 启动 Fechatter 开发服务器..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 复制配置文件
node scripts/copy-configs.js

echo ""
echo "💡 开发服务器启动中..."
echo "📱 请在浏览器中访问以下地址:"
echo ""
echo "   🌐 本地地址: http://localhost:5173"
echo "   🔗 网络地址: http://192.168.x.x:5173 (具体IP请看下方输出)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 按 Ctrl+C 停止服务器"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 启动Vite开发服务器
vite 