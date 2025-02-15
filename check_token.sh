#!/bin/bash

# 检查token是否有效
USER_ID="super"

if [ -f "${USER_ID}_token.txt" ]; then
  TOKEN=$(cat "${USER_ID}_token.txt")
  
  echo "检查 $USER_ID 的token是否有效..."
  echo ""
  
  # 尝试获取当前用户信息
  echo "1. 获取当前用户信息:"
  curl -s -X GET "http://45.77.178.85:8080/api/users/me" \
    -H "Authorization: Bearer $TOKEN" | jq '.' 2>/dev/null || echo "请求失败"
  
  echo ""
  echo "2. 获取用户列表:"
  curl -s -X GET "http://45.77.178.85:8080/api/users" \
    -H "Authorization: Bearer $TOKEN" | jq '.' 2>/dev/null || echo "请求失败"
  
else
  echo "未找到 ${USER_ID}_token.txt"
fi 