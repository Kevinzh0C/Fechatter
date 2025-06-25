-- Message enhancements: replies, mentions, priority, scheduling
-- Migration: 0012_message_enhancements.sql

-- Add message reply support
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS reply_to BIGINT REFERENCES messages(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS thread_id BIGINT REFERENCES messages(id) ON DELETE SET NULL;

-- Add message mentions support
ALTER TABLE messages
ADD COLUMN IF NOT EXISTS mentions BIGINT[] DEFAULT '{}';

-- Add message priority and importance levels
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS priority VARCHAR(20) DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
ADD COLUMN IF NOT EXISTS is_important BOOLEAN DEFAULT FALSE;

-- Add message scheduling functionality
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS scheduled_for TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS is_scheduled BOOLEAN DEFAULT FALSE;

-- Add message editing tracking
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS is_edited BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS edit_count INTEGER DEFAULT 0;

-- Create message_mentions table for detailed mention tracking
CREATE TABLE IF NOT EXISTS message_mentions (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    mentioned_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mention_type VARCHAR(20) DEFAULT 'user' CHECK (mention_type IN ('user', 'channel', 'everyone')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(message_id, mentioned_user_id)
);

-- Create message_reactions table for emoji reactions
CREATE TABLE IF NOT EXISTS message_reactions (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(message_id, user_id, emoji)
);

-- Create message_edits table to track edit history
CREATE TABLE IF NOT EXISTS message_edits (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    previous_content TEXT NOT NULL,
    edited_by BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    edit_reason VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create message_threads table for thread management
CREATE TABLE IF NOT EXISTS message_threads (
    id BIGSERIAL PRIMARY KEY,
    parent_message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    chat_id BIGINT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    reply_count INTEGER DEFAULT 0,
    last_reply_at TIMESTAMPTZ,
    last_reply_user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(parent_message_id)
);

-- Create scheduled_messages table for better tracking
CREATE TABLE IF NOT EXISTS scheduled_messages (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    scheduled_by BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scheduled_for TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'cancelled', 'failed')),
    retry_count INTEGER DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for message enhancements
CREATE INDEX IF NOT EXISTS idx_messages_reply_to ON messages(reply_to) WHERE reply_to IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_messages_thread_id ON messages(thread_id);
CREATE INDEX IF NOT EXISTS idx_messages_mentions ON messages USING GIN(mentions) WHERE array_length(mentions, 1) > 0;
CREATE INDEX IF NOT EXISTS idx_messages_priority ON messages(priority);
CREATE INDEX IF NOT EXISTS idx_messages_is_important ON messages(is_important);
CREATE INDEX IF NOT EXISTS idx_messages_scheduled_for ON messages(scheduled_for);

-- Create indexes for related tables
CREATE INDEX IF NOT EXISTS idx_message_mentions_message_id ON message_mentions(message_id);
CREATE INDEX IF NOT EXISTS idx_message_mentions_user_id ON message_mentions(mentioned_user_id);
CREATE INDEX IF NOT EXISTS idx_message_reactions_message_id ON message_reactions(message_id);
CREATE INDEX IF NOT EXISTS idx_message_reactions_user_id ON message_reactions(user_id);
CREATE INDEX IF NOT EXISTS idx_message_edits_message_id ON message_edits(message_id);
CREATE INDEX IF NOT EXISTS idx_message_threads_parent_id ON message_threads(parent_message_id);
CREATE INDEX IF NOT EXISTS idx_message_threads_chat_id ON message_threads(chat_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_messages_status ON scheduled_messages(status);
CREATE INDEX IF NOT EXISTS idx_scheduled_messages_scheduled_for ON scheduled_messages(scheduled_for);

-- Function to update thread reply count
CREATE OR REPLACE FUNCTION update_thread_reply_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.reply_to IS NOT NULL THEN
        -- Update reply count and last reply info
        UPDATE message_threads 
        SET 
            reply_count = reply_count + 1,
            last_reply_at = NEW.created_at,
            last_reply_user_id = NEW.sender_id,
            updated_at = NEW.created_at
        WHERE parent_message_id = NEW.reply_to;
        
        -- If thread doesn't exist, create it
        INSERT INTO message_threads (parent_message_id, chat_id, reply_count, last_reply_at, last_reply_user_id)
        VALUES (NEW.reply_to, NEW.chat_id, 1, NEW.created_at, NEW.sender_id)
        ON CONFLICT (parent_message_id) DO NOTHING;
        
    ELSIF TG_OP = 'DELETE' AND OLD.reply_to IS NOT NULL THEN
        -- Decrease reply count
        UPDATE message_threads 
        SET reply_count = GREATEST(reply_count - 1, 0)
        WHERE parent_message_id = OLD.reply_to;
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create trigger for thread management
DROP TRIGGER IF EXISTS manage_thread_replies ON messages;
CREATE TRIGGER manage_thread_replies
    AFTER INSERT OR DELETE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_thread_reply_count();