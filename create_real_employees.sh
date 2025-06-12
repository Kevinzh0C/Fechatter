#!/bin/bash

# 真实的员工名字列表
declare -A employees=(
  ["employee1"]="张伟"
  ["employee2"]="李娜"
  ["employee3"]="王强"
  ["employee4"]="刘洋"
  ["employee5"]="陈静"
  ["employee6"]="杨帆"
  ["employee7"]="赵磊"
  ["employee8"]="黄丽"
  ["employee9"]="周军"
  ["employee10"]="吴敏"
  ["employee11"]="徐涛"
  ["employee12"]="孙芳"
  ["employee13"]="马超"
  ["employee14"]="朱婷"
  ["employee15"]="郭明"
)

# API endpoint
API_BASE="http://45.77.178.85:8080/api"

echo "开始创建真实姓名的员工账号..."

# 创建员工账号
for emp_id in "${!employees[@]}"; do
  fullname="${employees[$emp_id]}"
  email="${emp_id}@test.com"
  
  echo "创建账号: $fullname ($email)"
  
  # 创建用户
  RESPONSE=$(curl -s -X POST "$API_BASE/auth/register" \
    -H "Content-Type: application/json" \
    -d "{
      \"fullname\": \"$fullname\",
      \"email\": \"$email\",
      \"password\": \"password123\",
      \"confirm_password\": \"password123\"
    }")
  
  # 提取token
  TOKEN=$(echo "$RESPONSE" | jq -r '.data.access_token')
  
  if [ "$TOKEN" != "null" ] && [ -n "$TOKEN" ]; then
    echo "$TOKEN" > "${emp_id}_token.txt"
    echo "✅ $fullname 创建成功"
  else
    echo "❌ $fullname 创建失败: $RESPONSE"
  fi
  
  sleep 0.5
done

echo "员工账号创建完成！" 