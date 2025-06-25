-- Fix all database issues discovered during testing

-- 1. Fix refresh_tokens table structure
-- Drop existing table if it has wrong structure
DROP TABLE IF EXISTS refresh_tokens CASCADE;

-- Recreate refresh_tokens table with correct structure
CREATE TABLE refresh_tokens (
    id BIGSERIAL PRIMARY KEY,                    -- Added: missing primary key
    user_id BIGINT NOT NULL,
    token VARCHAR(255),                          -- Optional: raw token (can be null)
    token_hash VARCHAR(255),                     -- Added: missing field for hashed token
    expires_at TIMESTAMPTZ NOT NULL,
    absolute_expires_at TIMESTAMPTZ,             -- Added: missing field
    issued_at TIMESTAMPTZ DEFAULT NOW(),         -- Added: missing field
    revoked BOOLEAN DEFAULT FALSE,               -- Added: missing field
    replaced_by VARCHAR(255),                    -- Added: missing field
    user_agent TEXT,
    ip_address TEXT,                             -- Fixed: was INET, now TEXT
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes on refresh_tokens table
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);

-- 2. Fix chats table field names to match Rust code expectations
-- Rename 'name' to 'chat_name' if it exists
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'chats' AND column_name = 'name') THEN
        ALTER TABLE chats RENAME COLUMN name TO chat_name;
    END IF;
END $$;

-- Rename 'chat_type' to 'type' if it exists
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'chats' AND column_name = 'chat_type') THEN
        ALTER TABLE chats RENAME COLUMN chat_type TO type;
    END IF;
END $$;

-- Add chat_members array field if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'chats' AND column_name = 'chat_members') THEN
        ALTER TABLE chats ADD COLUMN chat_members BIGINT[] DEFAULT '{}';
    END IF;
END $$;

-- 3. Ensure messages table has all required fields
-- Add idempotency_key if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'messages' AND column_name = 'idempotency_key') THEN
        ALTER TABLE messages ADD COLUMN idempotency_key UUID DEFAULT gen_random_uuid();
        CREATE INDEX idx_messages_idempotency_key ON messages(idempotency_key);
    END IF;
END $$;

-- Add read_by_users if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'messages' AND column_name = 'read_by_users') THEN
        ALTER TABLE messages ADD COLUMN read_by_users BIGINT[] DEFAULT '{}';
    END IF;
END $$;

-- Add mentions if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'messages' AND column_name = 'mentions') THEN
        ALTER TABLE messages ADD COLUMN mentions BIGINT[] DEFAULT '{}';
    END IF;
END $$;

-- Add reply_to if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'messages' AND column_name = 'reply_to') THEN
        ALTER TABLE messages ADD COLUMN reply_to BIGINT;
        ALTER TABLE messages ADD CONSTRAINT fk_reply_to 
            FOREIGN KEY (reply_to) REFERENCES messages(id) ON DELETE SET NULL;
    END IF;
END $$;

-- 4. Fix user status field to use VARCHAR instead of enum for compatibility
-- Update users table status column to VARCHAR if it's using enum
DO $$
BEGIN
    -- Check if status column is using enum type
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'users' AND column_name = 'status' 
               AND data_type = 'USER-DEFINED') THEN
        -- Convert enum to VARCHAR
        ALTER TABLE users ALTER COLUMN status TYPE VARCHAR(50) USING status::text;
        -- Set default value
        ALTER TABLE users ALTER COLUMN status SET DEFAULT 'Active';
    END IF;
END $$;

-- 5. Fix chat type field to use VARCHAR instead of enum for compatibility
-- Update chats table type column to VARCHAR if it's using enum
DO $$
BEGIN
    -- Check if type column is using enum type
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'chats' AND column_name = 'type' 
               AND data_type = 'USER-DEFINED') THEN
        -- Convert enum to VARCHAR
        ALTER TABLE chats ALTER COLUMN type TYPE VARCHAR(50) USING type::text;
    END IF;
END $$;

-- 6. Ensure all necessary indexes exist
-- Create missing indexes if they don't exist
CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chats_workspace_id ON chats(workspace_id);
CREATE INDEX IF NOT EXISTS idx_chats_created_by ON chats(created_by);
CREATE INDEX IF NOT EXISTS idx_chat_members_user_id ON chat_members(user_id);

-- 7. Ensure NATS notification functions exist
-- Create notification functions if they don't exist
CREATE OR REPLACE FUNCTION notify_new_message()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('new_message', json_build_object(
        'message_id', NEW.id,
        'chat_id', NEW.chat_id,
        'sender_id', NEW.sender_id,
        'content', NEW.content,
        'created_at', NEW.created_at
    )::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION notify_duplicate_message()
RETURNS TRIGGER AS $$
BEGIN
    -- This function is called when a duplicate idempotency_key is detected
    PERFORM pg_notify('duplicate_message', json_build_object(
        'idempotency_key', NEW.idempotency_key,
        'original_message_id', NEW.id,
        'chat_id', NEW.chat_id
    )::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION add_message_to_chat()
RETURNS TRIGGER AS $$
BEGIN
    -- Update chat_members array in chats table when new message is sent
    UPDATE chats 
    SET chat_members = array_append(chat_members, NEW.sender_id)
    WHERE id = NEW.chat_id 
    AND NOT (NEW.sender_id = ANY(chat_members));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers if they don't exist
DROP TRIGGER IF EXISTS trigger_notify_new_message ON messages;
CREATE TRIGGER trigger_notify_new_message
    AFTER INSERT ON messages
    FOR EACH ROW EXECUTE FUNCTION notify_new_message();

DROP TRIGGER IF EXISTS trigger_notify_duplicate_message ON messages;
CREATE TRIGGER trigger_notify_duplicate_message
    AFTER INSERT ON messages
    FOR EACH ROW EXECUTE FUNCTION notify_duplicate_message();

DROP TRIGGER IF EXISTS add_message_to_chat_trigger ON messages;
CREATE TRIGGER add_message_to_chat_trigger
    AFTER INSERT ON messages
    FOR EACH ROW EXECUTE FUNCTION add_message_to_chat(); 