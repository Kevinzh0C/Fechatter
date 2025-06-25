#!/bin/bash

# Test Current Backend Performance
# 测试当前后端性能

echo "🚀 Testing Fechatter Backend Performance"
echo "======================================"
echo ""
echo "Server: 45.77.178.85:8080"
echo "Time: $(date)"
echo ""

# Test health endpoint (baseline)
echo "1️⃣ Testing Health Endpoint (baseline):"
echo "-----------------------------------"
for i in {1..3}; do
  echo -n "Test $i: "
  curl -s -o /dev/null -w "%{time_total}s\n" \
    http://45.77.178.85:8080/api/health
done

echo ""
echo "2️⃣ Testing Message API (main issue):"
echo "---------------------------------"
echo "⚠️  This might take 8+ seconds..."
echo ""

# Test message endpoint
for i in {1..3}; do
  echo -n "Test $i: "
  START=$(date +%s.%N)
  curl -s -o /dev/null -w "Total: %{time_total}s (Connect: %{time_connect}s, Transfer: %{time_starttransfer}s)\n" \
    http://45.77.178.85:8080/api/chat/6/messages
  END=$(date +%s.%N)
  DIFF=$(echo "$END - $START" | bc)
  echo "       Real time: ${DIFF}s"
  sleep 2
done

echo ""
echo "3️⃣ Quick Performance Check:"
echo "------------------------"
echo ""

# One-liner performance test
echo "Running 5 quick tests..."
for i in {1..5}; do
  printf "Test %d: " $i
  time=$(curl -s -o /dev/null -w "%{time_total}" http://45.77.178.85:8080/api/chat/6/messages)
  echo "${time}s"
  sleep 1
done

echo ""
echo "======================================"
echo "📊 ANALYSIS:"
echo "======================================"
echo ""
echo "If message API takes 8+ seconds:"
echo "  ❌ Database indexes NOT deployed"
echo "  👉 Run the optimization scripts"
echo ""
echo "If message API takes <1 second:"
echo "  ✅ Database indexes ARE deployed"
echo "  👉 Performance already optimized"
echo ""
echo "Expected after optimization: <500ms"
echo "" 