-- Performance optimization indexes
-- These indexes significantly improve query performance for common operations
-- Note: Using regular CREATE INDEX instead of CONCURRENTLY to work within transaction

-- 1. Message query optimization
-- Speeds up message listing by chat with chronological ordering
CREATE INDEX IF NOT EXISTS idx_messages_chat_created_desc 
ON messages(chat_id, created_at DESC);

-- Optimize idempotency key lookups for duplicate detection
CREATE INDEX IF NOT EXISTS idx_messages_idempotency 
ON messages(idempotency_key) 
WHERE idempotency_key IS NOT NULL;

-- 2. Chat member query optimization
-- Speeds up "get user's active chats" queries
CREATE INDEX IF NOT EXISTS idx_chat_members_user_active 
ON chat_members(user_id, joined_at);

-- Speeds up "get chat members" queries
CREATE INDEX IF NOT EXISTS idx_chat_members_chat_active 
ON chat_members(chat_id, user_id);

-- 3. Message status optimization (if table exists)
-- Note: These indexes assume message_status table exists
-- Uncomment when message_status table is created

-- CREATE INDEX IF NOT EXISTS idx_message_status_unread 
-- ON message_status(user_id, chat_id, is_read) 
-- WHERE is_read = false;

-- CREATE INDEX IF NOT EXISTS idx_message_status_last_read 
-- ON message_status(user_id, chat_id, last_read_message_id);

-- 4. User workspace query optimization
-- Note: Users table has workspace_id, not a separate workspace_users table
CREATE INDEX IF NOT EXISTS idx_users_workspace 
ON users(workspace_id);

-- 5. Chat query optimization
-- Speeds up workspace chat listing
CREATE INDEX IF NOT EXISTS idx_chats_workspace_active
ON chats(workspace_id, created_at DESC);

-- 6. User query optimization
-- Speeds up user lookups by email (for login)
CREATE INDEX IF NOT EXISTS idx_users_email_lower
ON users(LOWER(email));

-- 7. Composite indexes for common JOIN operations
-- Optimize chat list with last message queries
CREATE INDEX IF NOT EXISTS idx_messages_chat_latest
ON messages(chat_id, id DESC);

-- Analyze tables to update statistics after index creation
ANALYZE messages;
ANALYZE chat_members;
ANALYZE chats;
ANALYZE users;
ANALYZE workspaces;