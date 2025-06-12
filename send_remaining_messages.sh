#!/bin/bash

# Remaining messages to reach 60 total
messages=(
  "employee4:会议纪要已发送到群邮件，请查收"
  "employee5:系统监控报警已处理，是误报"
  "employee6:下月招聘计划：前端2人，后端3人"
  "employee7:知识库搜索功能优化完成"
  "employee8:建议使用Docker容器化部署"
  "employee9:数据备份策略需要更新，建议增加异地备份"
  "employee10:用户反馈：希望增加深色模式"
  "employee11:本月客服响应时间平均15分钟，达标"
  "employee12:采购申请已提交，等待审批"
  "employee13:新LOGO设计稿已上传到共享盘"
  "employee14:公司WIFI密码下周更新，请留意通知"
  "employee15:月度安全报告：无重大安全事件"
  "super:感谢大家的努力，本季度业绩超预期完成！"
  "admin:月底前请提交考勤记录"
  "employee1:下周市场推广活动已准备就绪"
  "employee2:生产环境部署计划：周六凌晨2点"
)

# Send messages
for msg in "${messages[@]}"; do
  IFS=':' read -r sender content <<< "$msg"
  
  if [ "$sender" = "super" ]; then
    TOKEN=$(cat super_token.txt)
  elif [ "$sender" = "admin" ]; then
    TOKEN=$(cat admin_token.txt)
  else
    TOKEN=$(cat ${sender}_token.txt)
  fi
  
  curl -s -X POST http://45.77.178.85:8080/api/chat/3/messages \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"content\": \"$content\", \"files\": null}" | jq -r '.data.id'
  
  echo "Sent: $content"
  sleep 0.3
done 