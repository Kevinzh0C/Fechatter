#!/bin/bash

# API endpoint
API_BASE="http://45.77.178.85:8080/api"

# 登录函数
login_user() {
  local email=$1
  local user_id=$2
  
  echo -n "登录 $user_id ($email) ... "
  
  # 登录
  RESPONSE=$(curl -s -X POST "$API_BASE/auth/signin" \
    -H "Content-Type: application/json" \
    -d "{
      \"email\": \"$email\",
      \"password\": \"password123\"
    }")
  
  # 提取token
  TOKEN=$(echo "$RESPONSE" | jq -r '.data.access_token')
  
  if [ "$TOKEN" != "null" ] && [ -n "$TOKEN" ]; then
    echo "$TOKEN" > "${user_id}_token.txt"
    echo "✅ 成功"
  else
    echo "❌ 失败: $(echo "$RESPONSE" | jq -r '.message // .error // "未知错误"')"
  fi
  
  sleep 0.2
}

echo "========================================="
echo "刷新所有用户的Token"
echo "========================================="

# 管理员账号
login_user "super@test.com" "super"
login_user "admin@test.com" "admin"
login_user "developer@test.com" "developer"

# 员工账号
for i in {1..15}; do
  login_user "employee${i}@test.com" "employee${i}"
done

# 开发人员账号
for i in {1..5}; do
  login_user "dev${i}@test.com" "dev${i}"
done

echo "========================================="
echo "Token刷新完成！"
echo "=========================================" 