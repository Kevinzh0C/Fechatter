-- Fechatter Database Optimization Script
-- 一键优化数据库性能
-- 预期效果: 8秒 -> 1秒以内

-- ========================================
-- 1. 创建消息表索引（最重要）
-- ========================================

-- 检查现有索引
SELECT 'Current indexes on messages table:' as Info;
SHOW INDEXES FROM messages;

-- 创建复合索引（最优化查询性能）
CREATE INDEX IF NOT EXISTS idx_messages_chat_created 
ON messages(chat_id, created_at DESC);

-- 创建单列索引（备用）
CREATE INDEX IF NOT EXISTS idx_messages_chat_id 
ON messages(chat_id);

CREATE INDEX IF NOT EXISTS idx_messages_created_at 
ON messages(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_sender_id 
ON messages(sender_id);

-- ========================================
-- 2. 创建聊天表索引
-- ========================================

CREATE INDEX IF NOT EXISTS idx_chats_workspace_id 
ON chats(workspace_id);

CREATE INDEX IF NOT EXISTS idx_chats_updated_at 
ON chats(updated_at DESC);

-- ========================================
-- 3. 创建聊天成员表索引
-- ========================================

CREATE INDEX IF NOT EXISTS idx_chat_members_chat_user 
ON chat_members(chat_id, user_id);

CREATE INDEX IF NOT EXISTS idx_chat_members_chat_id 
ON chat_members(chat_id);

CREATE INDEX IF NOT EXISTS idx_chat_members_user_id 
ON chat_members(user_id);

-- ========================================
-- 4. 创建用户表索引（如果存在）
-- ========================================

-- 检查users表是否存在
SELECT COUNT(*) INTO @table_exists 
FROM information_schema.tables 
WHERE table_schema = DATABASE() 
AND table_name = 'users';

-- 如果存在则创建索引
SET @sql = IF(@table_exists > 0,
    'CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)',
    'SELECT "users table not found" as Info');
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- ========================================
-- 5. 更新表统计信息
-- ========================================

ANALYZE TABLE messages;
ANALYZE TABLE chats;
ANALYZE TABLE chat_members;

-- ========================================
-- 6. 验证优化效果
-- ========================================

SELECT '=== Optimization Complete ===' as Info;
SELECT 'New indexes on messages table:' as Info;
SHOW INDEXES FROM messages;

-- 测试查询性能
SELECT 'Query execution plan for fetching messages:' as Info;
EXPLAIN SELECT m.*, u.fullname as sender_name 
FROM messages m 
LEFT JOIN users u ON m.sender_id = u.id 
WHERE m.chat_id = 6 
ORDER BY m.created_at DESC 
LIMIT 15;

-- 显示表统计
SELECT 
    table_name,
    table_rows,
    avg_row_length,
    data_length/1024/1024 as data_size_mb,
    index_length/1024/1024 as index_size_mb
FROM information_schema.tables
WHERE table_schema = DATABASE()
AND table_name IN ('messages', 'chats', 'chat_members', 'users')
ORDER BY table_rows DESC;

-- ========================================
-- 7. 性能提升预期
-- ========================================

SELECT '=== Expected Performance Improvement ===' as Info;
SELECT 'Before: 8234ms' as 'Response Time';
SELECT 'After: ~500-1000ms (8-16x faster)' as 'Expected';
SELECT 'Note: Run this script during low traffic period' as Warning; 