#!/bin/bash

# European market discussion messages (30 messages total)
# Members: super, admin, employee2,4,6,8,10,12,14,15
messages=(
  'super:Good morning team! Lets discuss our European expansion strategy for Q4.'
  'admin:Our German operations are exceeding expectations with 35% growth MoM.'
  'employee2:The Berlin office is fully operational now. We have 25 staff members.'
  'employee4:France market shows strong potential. Paris and Lyon are our target cities.'
  'employee4:Local regulations are complex but we have good legal support.'
  'super:What about the UK market post-Brexit adjustments?'
  'employee6:UK operations remain stable. We adapted to new import regulations smoothly.'
  'employee8:Customer acquisition cost in Europe is 40% lower than anticipated.'
  'employee10:Our multilingual support team now covers 8 European languages.'
  'admin:Excellent progress. What about the supply chain optimization?'
  'employee12:We established distribution centers in Amsterdam and Milan.'
  'employee12:Delivery times reduced by 60% across Western Europe.'
  'employee14:Marketing campaigns in Spain and Portugal launching next week.'
  'super:How is our partnership with local retailers progressing?'
  'employee15:Signed agreements with 3 major chains in Germany and 2 in France.'
  'employee2:Digital marketing ROI in Europe is 250%, much higher than other regions.'
  'admin:We need to focus on Nordic countries next. High purchasing power there.'
  'employee4:Already initiated market research for Sweden and Norway.'
  'employee6:GDPR compliance is fully implemented across all systems.'
  'employee6:Data protection officer appointed for each country.'
  'super:Competition analysis shows we have a 6-month advantage.'
  'employee8:Our pricing strategy needs adjustment for Eastern Europe.'
  'employee10:Poland and Czech Republic showing interest in our products.'
  'admin:Lets allocate more resources to European expansion.'
  'employee12:Logistics costs can be reduced by 20% with better routing.'
  'employee14:Influencer partnerships in Italy generating great results.'
  'employee15:Customer retention rate in Europe is 85%, above global average.'
  'super:Schedule a meeting with European country managers next Tuesday.'
  'admin:Budget approved for hiring 50 more staff across Europe.'
  'super:Great work everyone! Europe will be our key growth driver.'
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
  elif [ "$sender" = "admin" ]; then
    TOKEN=$(cat admin_token.txt)
  else
    TOKEN=$(cat ${sender}_token.txt)
  fi
  
  # Send message
  curl -s -X POST http://45.77.178.85:8080/api/chat/6/messages \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"content\": \"$content\", \"files\": null}" | jq -r '.data.id'
  
  i=$((i + 1))
  echo "[$i/30] Europe Market: $sender"
  sleep 0.4
done

echo "European market channel discussion completed." 