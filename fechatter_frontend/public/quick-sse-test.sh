#!/bin/bash

echo "🔍 Quick SSE Issue Test - Frontend vs Backend"
echo "=========================================="

# 获取token
TOKEN=$(grep -o '"auth_token":"[^"]*"' ~/.config/fechatter/storage.json 2>/dev/null | cut -d'"' -f4)
if [ -z "$TOKEN" ]; then
    TOKEN=$(grep -o 'auth_token[^,]*' ~/.config/Cursor/User/globalStorage/state.vscdb 2>/dev/null)
fi

if [ -z "$TOKEN" ]; then
    echo "❌ No token found, please check localStorage manually"
    echo "Open browser dev tools and run: localStorage.getItem('auth_token')"
    exit 1
fi

echo "✅ Found token (${#TOKEN} chars)"

# 测试远程SSE端点
echo ""
echo "🌐 Testing remote SSE endpoint..."
timeout 5s curl -v "http://45.77.178.85:8080/events?access_token=$TOKEN" 2>&1 | \
    if grep -q "Connected"; then
        echo "✅ Connection established"
    else
        echo "❌ Connection failed - checking reasons..."
        echo "Response details:"
        timeout 3s curl -I "http://45.77.178.85:8080/events?access_token=$TOKEN" 2>&1
    fi

echo ""
echo "🎯 Quick Analysis:"
echo "- If connection established then immediately closed: BACKEND ISSUE"
echo "- If connection refused or timeout: BACKEND SERVICE DOWN"  
echo "- If 401 Unauthorized: TOKEN ISSUE"
echo "- If connection works but no events: BACKEND MESSAGE ROUTING"

echo ""
echo "📋 Next steps:"
echo "1. Visit: http://localhost:5173/remote-server-diagnostic.html"
echo "2. SSH to 45.77.178.85 and check logs"
echo "3. Compare results with backend-log-analysis-guide.md"

