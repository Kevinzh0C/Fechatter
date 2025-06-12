#!/bin/bash

# 测试更新单个用户的profile
USER_ID="super"
FULLNAME="超级管理员"

if [ -f "${USER_ID}_token.txt" ]; then
  TOKEN=$(cat "${USER_ID}_token.txt")
  
  echo "测试更新 $USER_ID 的姓名为: $FULLNAME"
  echo "使用的Token: ${TOKEN:0:20}..."
  echo ""
  
  # 发送请求并显示完整响应
  echo "发送请求..."
  RESPONSE=$(curl -s -X PUT "http://45.77.178.85:8080/api/users/profile" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"fullname\": \"$FULLNAME\"
    }" -w "\nHTTP_STATUS:%{http_code}")
  
  # 提取HTTP状态码
  HTTP_STATUS=$(echo "$RESPONSE" | tail -n 1 | cut -d: -f2)
  BODY=$(echo "$RESPONSE" | sed '$d')
  
  echo "HTTP状态码: $HTTP_STATUS"
  echo "响应内容:"
  echo "$BODY" | jq '.' 2>/dev/null || echo "$BODY"
else
  echo "未找到 ${USER_ID}_token.txt"
fi 