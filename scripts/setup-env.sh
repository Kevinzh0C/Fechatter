#!/bin/bash
# 环境配置安全设置脚本

set -e

echo "🔒 Fechatter 环境配置向导"
echo "========================"

# 检查 .env 文件是否已存在
if [ -f .env ]; then
    echo "⚠️  .env 文件已存在。是否覆盖？(y/N)"
    read -r response
    if [[ ! "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        echo "保留现有配置。"
        exit 0
    fi
fi

# 复制示例文件
cp env.example .env

echo ""
echo "📝 请配置以下敏感信息："
echo ""

# JWT Secret
echo "1. JWT Secret (用于认证，建议使用强密码):"
echo "   建议: $(openssl rand -base64 32)"
read -r -p "   输入 JWT_SECRET: " jwt_secret
sed -i '' "s/JWT_SECRET=.*/JWT_SECRET=$jwt_secret/" .env

# Redis Password
echo ""
echo "2. Redis 密码:"
echo "   建议: $(openssl rand -base64 16)"
read -r -p "   输入 REDIS_PASSWORD: " redis_password
sed -i '' "s/REDIS_PASSWORD=.*/REDIS_PASSWORD=$redis_password/" .env

# Meilisearch Master Key
echo ""
echo "3. Meilisearch Master Key:"
echo "   建议: $(openssl rand -base64 32)"
read -r -p "   输入 MEILI_MASTER_KEY: " meili_key
sed -i '' "s/MEILI_MASTER_KEY=.*/MEILI_MASTER_KEY=$meili_key/" .env

# OpenAI API Key
echo ""
echo "4. OpenAI API Key (可选，用于 AI 功能):"
echo "   ⚠️  请访问 https://platform.openai.com/api-keys 获取"
echo "   留空跳过..."
read -r -p "   输入 OPENAI_API_KEY: " openai_key
if [ ! -z "$openai_key" ]; then
    sed -i '' "s/OPENAI_API_KEY=.*/OPENAI_API_KEY=$openai_key/" .env
fi

# 设置文件权限
chmod 600 .env

echo ""
echo "✅ 环境配置完成！"
echo ""
echo "📋 配置摘要："
echo "   - JWT_SECRET: [已设置]"
echo "   - REDIS_PASSWORD: [已设置]"
echo "   - MEILI_MASTER_KEY: [已设置]"
if [ ! -z "$openai_key" ]; then
    echo "   - OPENAI_API_KEY: [已设置]"
else
    echo "   - OPENAI_API_KEY: [未设置]"
fi
echo ""
echo "🔐 .env 文件权限已设置为 600 (仅所有者可读写)"
echo ""
echo "⚠️  重要提醒："
echo "   1. 不要将 .env 文件提交到 Git"
echo "   2. 定期轮换密钥和密码"
echo "   3. 在生产环境使用更强的密码"
echo ""