#!/bin/bash

# 🎯 Fechatter SSE修复脚本
# 自动修复notify-server NATS订阅配置

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 远程服务器信息
REMOTE_HOST="45.77.178.85"
REMOTE_USER="root"
CONFIG_FILE="/root/fechatter/docker/configs/notify-ip.yml"
CONTAINER_NAME="notify-server-vcr"

echo -e "${BLUE}🎯 Fechatter SSE修复脚本${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# 步骤1: 检查SSH连接
echo -e "${YELLOW}步骤1: 检查SSH连接到 ${REMOTE_HOST}...${NC}"
if ssh -o ConnectTimeout=5 ${REMOTE_USER}@${REMOTE_HOST} 'echo "SSH连接成功"' 2>/dev/null; then
    echo -e "${GREEN}✅ SSH连接正常${NC}"
else
    echo -e "${RED}❌ SSH连接失败，请检查网络和密钥配置${NC}"
    exit 1
fi

# 步骤2: 检查远程文件
echo -e "${YELLOW}步骤2: 检查远程配置文件...${NC}"
if ssh ${REMOTE_USER}@${REMOTE_HOST} "test -f ${CONFIG_FILE}"; then
    echo -e "${GREEN}✅ 配置文件存在: ${CONFIG_FILE}${NC}"
else
    echo -e "${RED}❌ 配置文件不存在: ${CONFIG_FILE}${NC}"
    exit 1
fi

# 步骤3: 备份原配置
echo -e "${YELLOW}步骤3: 备份原配置文件...${NC}"
BACKUP_FILE="${CONFIG_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
ssh ${REMOTE_USER}@${REMOTE_HOST} "cp ${CONFIG_FILE} ${BACKUP_FILE}"
echo -e "${GREEN}✅ 已备份到: ${BACKUP_FILE}${NC}"

# 步骤4: 检查当前配置
echo -e "${YELLOW}步骤4: 检查当前NATS订阅配置...${NC}"
CURRENT_CONFIG=$(ssh ${REMOTE_USER}@${REMOTE_HOST} "grep -A 10 'subscription_subjects:' ${CONFIG_FILE}" || true)
echo "当前配置:"
echo "${CURRENT_CONFIG}"

# 检查是否已经包含realtime订阅
if echo "${CURRENT_CONFIG}" | grep -q "fechatter.realtime"; then
    echo -e "${GREEN}✅ 配置已包含fechatter.realtime.*订阅${NC}"
    echo -e "${BLUE}ℹ️  可能已经修复过了，继续验证...${NC}"
else
    echo -e "${RED}❌ 配置缺少fechatter.realtime.*订阅${NC}"
    
    # 步骤5: 修复配置
    echo -e "${YELLOW}步骤5: 修复NATS订阅配置...${NC}"
    
    # 创建修复脚本
    cat > /tmp/fix_notify_config.sh << 'EOF'
#!/bin/bash
CONFIG_FILE="$1"

# 使用sed在subscription_subjects部分添加realtime订阅
# 找到subscription_subjects:行，然后在其后的第一个缩进行前插入新行
sed -i '/subscription_subjects:/,/^[[:space:]]*-/ {
    /^[[:space:]]*-[[:space:]]*"fechatter\.message\.events"/ a\
- "fechatter.realtime.*"
}' "$CONFIG_FILE"

echo "配置修复完成"
EOF

    # 复制脚本到远程服务器并执行
    scp /tmp/fix_notify_config.sh ${REMOTE_USER}@${REMOTE_HOST}:/tmp/
    ssh ${REMOTE_USER}@${REMOTE_HOST} "chmod +x /tmp/fix_notify_config.sh && /tmp/fix_notify_config.sh ${CONFIG_FILE}"
    
    echo -e "${GREEN}✅ 配置修复完成${NC}"
fi

# 步骤6: 验证修复后的配置
echo -e "${YELLOW}步骤6: 验证修复后的配置...${NC}"
FIXED_CONFIG=$(ssh ${REMOTE_USER}@${REMOTE_HOST} "grep -A 10 'subscription_subjects:' ${CONFIG_FILE}")
echo "修复后配置:"
echo "${FIXED_CONFIG}"

if echo "${FIXED_CONFIG}" | grep -q "fechatter.realtime"; then
    echo -e "${GREEN}✅ 配置验证成功，已包含fechatter.realtime.*订阅${NC}"
else
    echo -e "${RED}❌ 配置验证失败，请手动检查${NC}"
    exit 1
fi

# 步骤7: 重启notify-server容器
echo -e "${YELLOW}步骤7: 重启notify-server容器...${NC}"
ssh ${REMOTE_USER}@${REMOTE_HOST} "docker restart ${CONTAINER_NAME}"
echo -e "${GREEN}✅ notify-server容器已重启${NC}"

# 步骤8: 验证容器状态
echo -e "${YELLOW}步骤8: 验证容器重启状态...${NC}"
sleep 3  # 等待容器启动
CONTAINER_STATUS=$(ssh ${REMOTE_USER}@${REMOTE_HOST} "docker ps --filter name=${CONTAINER_NAME} --format 'table {{.Names}}\t{{.Status}}'")
echo "${CONTAINER_STATUS}"

if echo "${CONTAINER_STATUS}" | grep -q "Up"; then
    echo -e "${GREEN}✅ notify-server容器运行正常${NC}"
else
    echo -e "${RED}❌ notify-server容器状态异常${NC}"
    echo "检查容器日志:"
    ssh ${REMOTE_USER}@${REMOTE_HOST} "docker logs --tail 20 ${CONTAINER_NAME}"
    exit 1
fi

# 步骤9: 检查日志验证修复
echo -e "${YELLOW}步骤9: 检查启动日志验证修复...${NC}"
echo "最新的notify-server日志:"
ssh ${REMOTE_USER}@${REMOTE_HOST} "docker logs --tail 15 ${CONTAINER_NAME} 2>/dev/null || docker logs --tail 15 ${CONTAINER_NAME}"

echo ""
echo -e "${GREEN}🎉 SSE修复完成！${NC}"
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}✅ 修复总结:${NC}"
echo -e "${GREEN}   • 已备份原配置文件${NC}"
echo -e "${GREEN}   • 已添加fechatter.realtime.*订阅${NC}"
echo -e "${GREEN}   • 已重启notify-server容器${NC}"
echo -e "${GREEN}   • 容器运行状态正常${NC}"
echo ""
echo -e "${BLUE}📋 下一步测试:${NC}"
echo -e "${BLUE}   1. 访问 http://localhost:5173${NC}"
echo -e "${BLUE}   2. 登录系统 (super@test.com/password)${NC}"
echo -e "${BLUE}   3. 发送测试消息${NC}"
echo -e "${BLUE}   4. 检查消息状态是否从⏰变为✅${NC}"
echo ""
echo -e "${YELLOW}⚡ 预期效果: 消息确认成功率从0%提升到95%+${NC}"

# 清理临时文件
rm -f /tmp/fix_notify_config.sh
ssh ${REMOTE_USER}@${REMOTE_HOST} "rm -f /tmp/fix_notify_config.sh" 2>/dev/null || true

echo ""
echo -e "${BLUE}🔍 如果问题依然存在，请运行故障排除脚本检查详细状态${NC}" 