#!/bin/bash

# API endpoint
API_BASE="http://45.77.178.85:8080/api"

# 统计变量
total=0
success=0
failed=0
skipped=0

echo "========================================="
echo "开始批量更新用户真实姓名"
echo "========================================="

# 更新函数
update_user() {
  local user_id=$1
  local fullname=$2
  
  ((total++))
  echo -n "[$total] 更新 $user_id -> $fullname ... "
  
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
}

# 更新管理员账号
update_user "super" "超级管理员"
update_user "admin" "系统管理员"
update_user "developer" "产品开发者"

# 更新员工账号
update_user "employee1" "张伟"
update_user "employee2" "李娜"
update_user "employee3" "王强"
update_user "employee4" "刘洋"
update_user "employee5" "陈静"
update_user "employee6" "杨帆"
update_user "employee7" "赵磊"
update_user "employee8" "黄丽"
update_user "employee9" "周军"
update_user "employee10" "吴敏"
update_user "employee11" "徐涛"
update_user "employee12" "孙芳"
update_user "employee13" "马超"
update_user "employee14" "朱婷"
update_user "employee15" "郭明"

# 更新开发人员账号
update_user "dev1" "王建国"
update_user "dev2" "李晓明"
update_user "dev3" "张鹏飞"
update_user "dev4" "刘思雨"
update_user "dev5" "陈晓东"

echo ""
echo "========================================="
echo "更新完成！统计信息："
echo "总计: $total 个用户"
echo "成功: $success 个"
echo "失败: $failed 个"
echo "跳过: $skipped 个"
echo "========================================="

# 显示用户名映射表
echo ""
echo "用户名映射表："
echo "----------------------------------------"
printf "%-15s | %s\n" "用户ID" "真实姓名"
echo "----------------------------------------"
printf "%-15s | %s\n" "super" "超级管理员"
printf "%-15s | %s\n" "admin" "系统管理员"
printf "%-15s | %s\n" "developer" "产品开发者"
echo "--- 员工账号 ---"
printf "%-15s | %s\n" "employee1" "张伟"
printf "%-15s | %s\n" "employee2" "李娜"
printf "%-15s | %s\n" "employee3" "王强"
printf "%-15s | %s\n" "employee4" "刘洋"
printf "%-15s | %s\n" "employee5" "陈静"
printf "%-15s | %s\n" "employee6" "杨帆"
printf "%-15s | %s\n" "employee7" "赵磊"
printf "%-15s | %s\n" "employee8" "黄丽"
printf "%-15s | %s\n" "employee9" "周军"
printf "%-15s | %s\n" "employee10" "吴敏"
printf "%-15s | %s\n" "employee11" "徐涛"
printf "%-15s | %s\n" "employee12" "孙芳"
printf "%-15s | %s\n" "employee13" "马超"
printf "%-15s | %s\n" "employee14" "朱婷"
printf "%-15s | %s\n" "employee15" "郭明"
echo "--- 开发人员账号 ---"
printf "%-15s | %s\n" "dev1" "王建国"
printf "%-15s | %s\n" "dev2" "李晓明"
printf "%-15s | %s\n" "dev3" "张鹏飞"
printf "%-15s | %s\n" "dev4" "刘思雨"
printf "%-15s | %s\n" "dev5" "陈晓东"
echo "----------------------------------------" 