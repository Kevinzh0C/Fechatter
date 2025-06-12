#!/bin/bash

# 所有用户的真实姓名映射
declare -A all_users=(
  # 管理员账号
  ["super"]="超级管理员"
  ["admin"]="系统管理员"
  ["developer"]="产品开发者"
  
  # 员工账号
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
  
  # 开发人员账号
  ["dev1"]="王建国"
  ["dev2"]="李晓明"
  ["dev3"]="张鹏飞"
  ["dev4"]="刘思雨"
  ["dev5"]="陈晓东"
)

# API endpoint
API_BASE="http://45.77.178.85:8080/api"

# 统计变量
total=0
success=0
failed=0
skipped=0

echo "========================================="
echo "开始批量更新用户真实姓名"
echo "总共需要更新: ${#all_users[@]} 个用户"
echo "========================================="

# 更新每个用户的profile
for user_id in "${!all_users[@]}"; do
  fullname="${all_users[$user_id]}"
  ((total++))
  
  echo -n "[$total/${#all_users[@]}] 更新 $user_id -> $fullname ... "
  
  # 检查是否有token文件
  if [ -f "${user_id}_token.txt" ]; then
    TOKEN=$(cat "${user_id}_token.txt")
    
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
      echo "✅ 成功"
      ((success++))
    else
      echo "❌ 失败"
      echo "  错误信息: $(echo "$RESPONSE" | jq -r '.message // .error // "未知错误"')"
      ((failed++))
    fi
  else
    echo "⚠️  跳过 (未找到token文件)"
    ((skipped++))
  fi
  
  # 避免请求过快
  sleep 0.2
done

echo ""
echo "========================================="
echo "更新完成！统计信息："
echo "总计: $total 个用户"
echo "成功: $success 个"
echo "失败: $failed 个"
echo "跳过: $skipped 个"
echo "========================================="

# 显示当前用户名映射表
echo ""
echo "用户名映射表："
echo "----------------------------------------"
printf "%-15s | %s\n" "用户ID" "真实姓名"
echo "----------------------------------------"
for user_id in "${!all_users[@]}"; do
  printf "%-15s | %s\n" "$user_id" "${all_users[$user_id]}"
done | sort
echo "----------------------------------------" 