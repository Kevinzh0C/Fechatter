#!/bin/bash

# Developer standup messages with code and technical discussions
messages=(
  'super:大家早上好，开始今天的standup，先说说昨天的进度'
  'dev1:昨天完成了用户认证模块的重构，使用JWT替换了session'
  'dev1:代码示例：\n```typescript\nconst generateToken = (user: User): string => {\n  return jwt.sign(\n    { id: user.id, email: user.email },\n    process.env.JWT_SECRET,\n    { expiresIn: "24h" }\n  );\n};\n```'
  'dev2:我这边完成了支付接口的集成，支持支付宝和微信'
  'dev2:遇到了回调验证的问题，已经解决了。这是流程图：[payment_flow_diagram.png]'
  'dev3:数据库优化完成，查询速度提升了40%'
  'dev3:主要是给订单表加了复合索引：\n```sql\nCREATE INDEX idx_orders_user_status_date \nON orders(user_id, status, created_at);\n```'
  'super:很好！支付模块的安全性测试做了吗？'
  'dev2:已经做了，包括签名验证、金额校验、重放攻击防护'
  'dev4:前端这边完成了商品列表的虚拟滚动优化'
  'dev4:性能对比图：[performance_comparison.png] 内存占用减少70%'
  'dev5:移动端适配完成，所有页面都支持响应式了'
  'dev5:这是不同设备的截图对比：[mobile_responsive_screenshots.png]'
  'super:今天的计划是什么？'
  'dev1:继续完善权限系统，实现RBAC'
  'dev1:设计文档已经写好：\n```yaml\nroles:\n  - admin: full_access\n  - editor: content_management\n  - viewer: read_only\n```'
  'dev2:集成物流查询API，预计下午完成'
  'dev3:开始做数据迁移脚本，从老系统迁移用户数据'
  'dev3:迁移策略：\n```python\ndef migrate_users(batch_size=1000):\n    offset = 0\n    while True:\n        users = fetch_old_users(offset, batch_size)\n        if not users:\n            break\n        insert_new_users(users)\n        offset += batch_size\n```'
  'dev4:实现图片懒加载和WebP格式支持'
  'dev5:开始做离线缓存功能，使用Service Worker'
  'super:有什么阻塞问题吗？'
  'dev1:需要确认一下权限继承的规则'
  'dev2:物流商的API文档不太清楚，可能需要联系他们'
  'dev3:老系统的数据有些不一致，需要清洗'
  'dev4:CDN配置需要运维协助'
  'dev5:iOS的推送证书需要更新'
  'super:我来协调这些问题。大家记得更新JIRA状态'
  'super:代码review安排：下午3点review支付模块'
  'dev2:OK，我准备一下代码和设计文档'
  'dev1:建议增加单元测试覆盖率要求，目前只有60%'
  'super:同意，目标定在80%以上。先从核心模块开始'
  'dev3:数据库备份策略也需要更新，建议用增量备份'
  'dev4:前端构建优化的方案：[build_optimization_plan.png]'
  'dev5:App的崩溃率降到0.1%了，这是统计图表：[crash_rate_chart.png]'
  'super:很好！保持这个势头。今天的standup就到这里'
)

# Send messages
i=0
for msg in "${messages[@]}"; do
  # Parse sender and content
  sender=$(echo "$msg" | cut -d: -f1)
  content=$(echo "$msg" | cut -d: -f2-)
  
  # Get appropriate token
  if [ "$sender" = "super" ]; then
    TOKEN=$(cat super_token.txt)
  else
    TOKEN=$(cat ${sender}_token.txt)
  fi
  
  # Send message
  curl -s -X POST http://45.77.178.85:8080/api/chat/4/messages \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"content\": \"$content\", \"files\": null}" | jq -r '.data.id'
  
  i=$((i + 1))
  echo "[$i] Sent from $sender"
  sleep 0.4
done 