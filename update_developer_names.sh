#!/bin/bash

# 开发人员的真实名字映射
declare -A dev_names=(
  ["dev1"]="王建国"
  ["dev2"]="李晓明"
  ["dev3"]="张鹏飞"
  ["dev4"]="刘思雨"
  ["dev5"]="陈晓东"
)

# API endpoint
API_BASE="http://45.77.178.85:8080/api"

echo "开始更新开发人员的真实姓名..."

# 更新每个开发人员的profile
for dev_id in "${!dev_names[@]}"; do
  fullname="${dev_names[$dev_id]}"
  
  # 检查是否有token文件
  if [ -f "${dev_id}_token.txt" ]; then
    TOKEN=$(cat "${dev_id}_token.txt")
    
    echo "更新 $dev_id 的姓名为: $fullname"
    
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
      echo "✅ $dev_id 更新成功: $fullname"
    else
      echo "❌ $dev_id 更新失败: $RESPONSE"
    fi
  else
    echo "⚠️  未找到 ${dev_id}_token.txt，跳过"
  fi
  
  sleep 0.3
done

echo "开发人员姓名更新完成！" 