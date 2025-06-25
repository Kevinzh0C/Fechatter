#!/bin/bash

echo ""
echo "🚀 启动 Fechatter 开发服务器..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 复制配置文件
echo "📋 复制配置文件中..."
node scripts/copy-configs.js

echo ""
echo "💡 开发服务器启动中..."
echo "📱 请在浏览器中手动访问以下地址:"
echo ""
echo "   🌐 本地地址: http://localhost:5173"
echo "   🔗 网络地址: 请看下方 Vite 输出中的 Network 地址"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 按 Ctrl+C 停止服务器"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 启动Vite开发服务器
yarn vite
