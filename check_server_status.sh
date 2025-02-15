#!/bin/bash

API_BASE="http://45.77.178.85:8080"

echo "检查服务器状态..."
echo ""

# 1. 检查健康状态
echo "1. 健康检查:"
curl -s "$API_BASE/health" | jq '.' 2>/dev/null || echo "健康检查失败"

echo ""
echo "2. 检查API版本:"
curl -s "$API_BASE/api/version" | jq '.' 2>/dev/null || echo "版本检查失败"

echo ""
echo "3. 测试注册新用户:"
RESPONSE=$(curl -s -X POST "$API_BASE/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "fullname": "Test User",
    "email": "test_'$(date +%s)'@test.com",
    "password": "password123",
    "confirm_password": "password123"
  }')

echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE" 