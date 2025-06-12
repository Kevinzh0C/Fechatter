-- Add unread message tracking
-- Migration: 0024_add_unread_tracking.sql

-- Add last_read_message_id to chat_members table
ALTER TABLE chat_members 
ADD COLUMN IF NOT EXISTS last_read_message_id BIGINT REFERENCES messages(id);

-- Add last_read_at timestamp
ALTER TABLE chat_members 
ADD COLUMN IF NOT EXISTS last_read_at TIMESTAMPTZ;

-- Create index for efficient unread count queries
CREATE INDEX IF NOT EXISTS idx_chat_members_last_read 
ON chat_members(chat_id, user_id, last_read_message_id);

-- Function to get unread count for a user in a chat
CREATE OR REPLACE FUNCTION get_unread_count(p_user_id BIGINT, p_chat_id BIGINT)
RETURNS INTEGER AS $$
DECLARE
    v_last_read_id BIGINT;
    v_unread_count INTEGER;
BEGIN
    -- Get the last read message ID for this user in this chat
    SELECT last_read_message_id INTO v_last_read_id
    FROM chat_members
    WHERE user_id = p_user_id AND chat_id = p_chat_id;
    
    -- If no last read message, count all messages
    IF v_last_read_id IS NULL THEN
        SELECT COUNT(*) INTO v_unread_count
        FROM messages
        WHERE chat_id = p_chat_id
        AND sender_id != p_user_id;
    ELSE
        -- Count messages after the last read message
        SELECT COUNT(*) INTO v_unread_count
        FROM messages
        WHERE chat_id = p_chat_id
        AND id > v_last_read_id
        AND sender_id != p_user_id;
    END IF;
    
    RETURN COALESCE(v_unread_count, 0);
END;
$$ LANGUAGE plpgsql;

-- Function to update last read message
CREATE OR REPLACE FUNCTION update_last_read_message(
    p_user_id BIGINT, 
    p_chat_id BIGINT, 
    p_message_id BIGINT
)
RETURNS VOID AS $$
BEGIN
    UPDATE chat_members
    SET last_read_message_id = p_message_id,
        last_read_at = NOW()
    WHERE user_id = p_user_id 
    AND chat_id = p_chat_id
    AND (last_read_message_id IS NULL OR last_read_message_id < p_message_id);
END;
$$ LANGUAGE plpgsql;