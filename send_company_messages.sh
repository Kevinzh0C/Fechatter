#!/bin/bash

# Message array with sender and content
messages=(
  "admin:今天会讨论Q4的销售目标，销售部的同事请准备相关数据"
  "employee2:昨天的服务器升级已经完成，目前运行正常"
  "employee3:市场部需要技术支持做新产品的demo视频"
  "super:@employee3 这个可以安排，下午我们详聊需求"
  "employee4:财务报表已经更新，请各部门负责人查收邮件"
  "employee5:客户反馈系统有个bug，正在修复中"
  "employee6:新员工培训材料已经准备好了，HR可以安排时间"
  "admin:下周三有重要客户来访，请大家保持办公环境整洁"
  "employee7:产品文档更新完毕，已同步到知识库"
  "employee8:建议增加每周的技术分享会，提升团队技术水平"
  "super:@employee8 好建议！可以从下周开始，每周五下午"
  "employee9:服务器监控显示CPU使用率偏高，需要优化"
  "employee10:新版本APP已提交应用商店审核"
  "employee11:客户满意度调查结果出来了，整体评分8.5/10"
  "employee12:仓库库存系统需要升级，老版本不太稳定"
  "admin:请各部门提交下个月的预算申请"
  "employee13:设计稿已完成，请产品经理review"
  "employee14:建议公司订购一些技术书籍，充实图书角"
  "employee15:最新的安全补丁已经部署到所有服务器"
  "super:本周五下午4点有全员大会，请勿安排其他会议"
  "employee1:营销活动的数据分析报告已完成"
  "employee2:代码审查发现几个潜在问题，已创建issue"
  "employee3:客户要求增加导出Excel功能，技术上可行"
  "admin:IT部门下周会进行网络维护，可能短暂断网"
  "employee4:新的报销流程已经上线，请查看使用指南"
  "employee5:bug已修复，正在测试环境验证"
  "employee6:三位新同事下周一入职，请各部门做好准备"
  "super:产品路线图已更新，请产品团队关注"
  "employee7:API文档需要更新，这周会完成"
  "employee8:技术分享会第一期主题：微服务架构实践"
  "employee9:数据库优化完成，查询速度提升30%"
  "employee10:iOS版本审核通过，Android还在等待"
  "admin:年度体检安排在下个月，请关注通知"
  "employee11:建议增加客服团队人手，咨询量增长明显"
  "employee12:供应商付款已处理，请财务确认"
  "employee13:UI改版方案已发送，请查看邮件"
  "employee14:团建活动投票结果：郊游获胜"
  "employee15:安全扫描发现2个低危漏洞，已修复"
  "super:明天的产品评审会议改到3楼会议室"
  "employee1:竞品分析报告完成，有几个值得借鉴的点"
  "employee2:持续集成系统升级成功，构建速度提升"
  "employee3:客户demo准备完毕，明天上午演示"
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