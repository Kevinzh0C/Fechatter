#!/bin/bash

# 真实的员工名字映射
declare -A employee_names=(
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

echo "开始更新员工的真实姓名..."

# 更新每个员工的profile
for emp_id in "${!employee_names[@]}"; do
  fullname="${employee_names[$emp_id]}"
  
  # 检查是否有token文件
  if [ -f "${emp_id}_token.txt" ]; then
    TOKEN=$(cat "${emp_id}_token.txt")
    
    echo "更新 $emp_id 的姓名为: $fullname"
    
    # 更新用户profile
    RESPONSE=$(curl -s -X PUT "$API_BASE/users/profile" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d "{
        \"fullname\": \"$fullname\"
      }")
    
    # 检查响应
    SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
    
    if [ "$SUCCESS" = "true" ]; then
      echo "✅ $emp_id 更新成功: $fullname"
    else
      echo "❌ $emp_id 更新失败: $RESPONSE"
    fi
  else
    echo "⚠️  未找到 ${emp_id}_token.txt，跳过"
  fi
  
  sleep 0.3
done

echo "员工姓名更新完成！" 