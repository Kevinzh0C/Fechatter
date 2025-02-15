#!/bin/bash

# Check and Deploy Database Indexes
# 检查并部署数据库索引

echo "🔍 Fechatter Database Index Check & Deploy Script"
echo "================================================"
echo ""

# Configuration
SERVER_IP="45.77.178.85"
DB_NAME="fechatter"  # 修改为实际数据库名
DB_USER="root"       # 修改为实际用户名

echo "📍 Target Server: $SERVER_IP"
echo "📊 Database: $DB_NAME"
echo ""

# ========================================
# 1. CHECK CURRENT INDEXES
# ========================================
echo "1️⃣ CHECKING CURRENT INDEXES"
echo "==========================="
echo ""
echo "To check if indexes exist, run this SQL:"
echo ""

cat << 'EOF'
-- Connect to database first
USE fechatter;  -- Replace with your database name

-- Check messages table indexes
SELECT 
    INDEX_NAME,
    COLUMN_NAME,
    SEQ_IN_INDEX,
    CARDINALITY
FROM INFORMATION_SCHEMA.STATISTICS
WHERE TABLE_SCHEMA = DATABASE()
AND TABLE_NAME = 'messages'
ORDER BY INDEX_NAME, SEQ_IN_INDEX;

-- Check if our optimization indexes exist
SELECT 
    CASE 
        WHEN COUNT(*) > 0 THEN '✅ Index exists'
        ELSE '❌ Index missing'
    END as Status,
    'idx_messages_chat_created' as Index_Name
FROM INFORMATION_SCHEMA.STATISTICS
WHERE TABLE_SCHEMA = DATABASE()
AND TABLE_NAME = 'messages'
AND INDEX_NAME = 'idx_messages_chat_created';

-- Check all critical indexes
SELECT 
    'messages' as Table_Name,
    INDEX_NAME,
    GROUP_CONCAT(COLUMN_NAME ORDER BY SEQ_IN_INDEX) as Columns
FROM INFORMATION_SCHEMA.STATISTICS
WHERE TABLE_SCHEMA = DATABASE()
AND TABLE_NAME = 'messages'
AND INDEX_NAME LIKE 'idx_%'
GROUP BY INDEX_NAME;
EOF

echo ""
echo "========================================="
echo ""

# ========================================
# 2. QUICK DEPLOYMENT COMMAND
# ========================================
echo "2️⃣ QUICK DEPLOYMENT (One Command)"
echo "================================"
echo ""
echo "SSH to server and run:"
echo ""

cat << 'EOF'
# One-liner to create all indexes
mysql -u root -p fechatter -e "
CREATE INDEX IF NOT EXISTS idx_messages_chat_created ON messages(chat_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages(sender_id);
ANALYZE TABLE messages;
"
EOF

echo ""
echo "========================================="
echo ""

# ========================================
# 3. VERIFY DEPLOYMENT
# ========================================
echo "3️⃣ VERIFY DEPLOYMENT"
echo "==================="
echo ""

cat << 'EOF'
-- After creating indexes, verify performance
EXPLAIN SELECT * FROM messages 
WHERE chat_id = 6 
ORDER BY created_at DESC 
LIMIT 15;

-- Look for "Using index" in Extra column
-- key column should show idx_messages_chat_created

-- Test actual query time
SET profiling = 1;
SELECT * FROM messages WHERE chat_id = 6 ORDER BY created_at DESC LIMIT 15;
SHOW PROFILES;
SET profiling = 0;
EOF

echo ""
echo "========================================="
echo ""

# ========================================
# 4. PERFORMANCE TEST
# ========================================
echo "4️⃣ PERFORMANCE TEST COMMANDS"
echo "==========================="
echo ""

cat << 'EOF'
# Test API response time BEFORE indexes
echo "Testing current performance..."
time curl -s -o /dev/null -w "Response time: %{time_total}s\n" \
  http://45.77.178.85:8080/api/chat/6/messages

# After applying indexes, test again
echo "Testing after optimization..."
time curl -s -o /dev/null -w "Response time: %{time_total}s\n" \
  http://45.77.178.85:8080/api/chat/6/messages

# Continuous monitoring
for i in {1..5}; do
  echo -n "Test $i: "
  curl -s -o /dev/null -w "%{time_total}s\n" \
    http://45.77.178.85:8080/api/chat/6/messages
  sleep 1
done
EOF

echo ""
echo "========================================="
echo ""

# ========================================
# 5. FULL DEPLOYMENT SCRIPT
# ========================================
echo "5️⃣ FULL DEPLOYMENT SCRIPT"
echo "======================="
echo ""
echo "Save this as deploy-indexes.sql and run:"
echo ""

cat << 'EOF'
-- Fechatter Index Deployment
-- Run this to optimize performance

USE fechatter;  -- Change to your database name

-- Show before state
SELECT 'BEFORE: Current indexes' as Status;
SHOW INDEXES FROM messages;

-- Create all optimization indexes
CREATE INDEX IF NOT EXISTS idx_messages_chat_created 
ON messages(chat_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_chat_id 
ON messages(chat_id);

CREATE INDEX IF NOT EXISTS idx_messages_created_at 
ON messages(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_sender_id 
ON messages(sender_id);

-- Update statistics
ANALYZE TABLE messages;

-- Show after state
SELECT 'AFTER: New indexes' as Status;
SHOW INDEXES FROM messages;

-- Test query plan
EXPLAIN SELECT * FROM messages 
WHERE chat_id = 6 
ORDER BY created_at DESC 
LIMIT 15;

SELECT 'Deployment complete! Expected speedup: 8x-16x' as Result;
EOF

echo ""
echo "========================================="
echo ""

# ========================================
# 6. ROLLBACK COMMANDS (if needed)
# ========================================
echo "6️⃣ ROLLBACK COMMANDS (if needed)"
echo "==============================="
echo ""

cat << 'EOF'
-- If you need to remove indexes
DROP INDEX idx_messages_chat_created ON messages;
DROP INDEX idx_messages_chat_id ON messages;
DROP INDEX idx_messages_created_at ON messages;
DROP INDEX idx_messages_sender_id ON messages;
EOF

echo ""
echo "========================================="
echo "📊 SUMMARY"
echo "========================================="
echo ""
echo "1. First, CHECK if indexes exist (Section 1)"
echo "2. If not, DEPLOY using quick command (Section 2)"
echo "3. VERIFY the deployment worked (Section 3)"
echo "4. TEST performance improvement (Section 4)"
echo ""
echo "Expected improvement: 8234ms → 500ms (16x faster)"
echo ""
echo "⚠️  IMPORTANT: Replace database name and credentials!"
echo "" 