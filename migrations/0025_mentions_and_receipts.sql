-- Add @mentions and enhanced read receipts
-- Migration: 0025_mentions_and_receipts.sql

-- Add mentions table to track @mentions in messages
CREATE TABLE IF NOT EXISTS message_mentions (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    mentioned_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mention_type VARCHAR(20) NOT NULL DEFAULT 'user' CHECK (mention_type IN ('user', 'everyone', 'here')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(message_id, mentioned_user_id, mention_type)
);

-- Create indexes for mentions
CREATE INDEX IF NOT EXISTS idx_message_mentions_message_id ON message_mentions(message_id);
CREATE INDEX IF NOT EXISTS idx_message_mentions_user_id ON message_mentions(mentioned_user_id);
CREATE INDEX IF NOT EXISTS idx_message_mentions_type ON message_mentions(mention_type);

-- Add mention_count to chat_members for unread mentions tracking
ALTER TABLE chat_members 
ADD COLUMN IF NOT EXISTS unread_mentions_count INTEGER DEFAULT 0;

-- Function to extract and store mentions from message content
CREATE OR REPLACE FUNCTION extract_and_store_mentions(
    p_message_id BIGINT,
    p_content TEXT,
    p_chat_id BIGINT
) RETURNS INTEGER AS $$
DECLARE
    mention_count INTEGER := 0;
    mention_pattern TEXT := '@(\w+)';
    username_match TEXT;
    mentioned_user_id BIGINT;
BEGIN
    -- Clear existing mentions for this message
    DELETE FROM message_mentions WHERE message_id = p_message_id;
    
    -- Handle @everyone mention
    IF p_content ~* '@everyone' THEN
        INSERT INTO message_mentions (message_id, mentioned_user_id, mention_type)
        SELECT p_message_id, cm.user_id, 'everyone'
        FROM chat_members cm
        WHERE cm.chat_id = p_chat_id
        AND cm.left_at IS NULL
        ON CONFLICT (message_id, mentioned_user_id, mention_type) DO NOTHING;
        
        GET DIAGNOSTICS mention_count = ROW_COUNT;
    END IF;
    
    -- Handle @here mention (online users)
    IF p_content ~* '@here' THEN
        INSERT INTO message_mentions (message_id, mentioned_user_id, mention_type)
        SELECT p_message_id, cm.user_id, 'here'
        FROM chat_members cm
        JOIN users u ON u.id = cm.user_id
        WHERE cm.chat_id = p_chat_id
        AND cm.left_at IS NULL
        AND u.status = 'online'  -- Assuming we track online status
        ON CONFLICT (message_id, mentioned_user_id, mention_type) DO NOTHING;
        
        GET DIAGNOSTICS mention_count = mention_count + ROW_COUNT;
    END IF;
    
    -- Handle individual @username mentions
    FOR username_match IN 
        SELECT regexp_matches(p_content, mention_pattern, 'g')[1]
    LOOP
        -- Find user by username in the same workspace
        SELECT u.id INTO mentioned_user_id
        FROM users u
        JOIN chat_members cm ON cm.user_id = u.id
        WHERE u.username = username_match
        AND cm.chat_id = p_chat_id
        AND cm.left_at IS NULL
        LIMIT 1;
        
        IF mentioned_user_id IS NOT NULL THEN
            INSERT INTO message_mentions (message_id, mentioned_user_id, mention_type)
            VALUES (p_message_id, mentioned_user_id, 'user')
            ON CONFLICT (message_id, mentioned_user_id, mention_type) DO NOTHING;
            
            mention_count := mention_count + 1;
        END IF;
    END LOOP;
    
    RETURN mention_count;
END;
$$ LANGUAGE plpgsql;

-- Function to get unread mentions count for a user in a chat
CREATE OR REPLACE FUNCTION get_unread_mentions_count(
    p_user_id BIGINT,
    p_chat_id BIGINT
) RETURNS INTEGER AS $$
DECLARE
    v_last_read_id BIGINT;
    v_mentions_count INTEGER;
BEGIN
    -- Get the last read message ID for this user in this chat
    SELECT last_read_message_id INTO v_last_read_id
    FROM chat_members
    WHERE user_id = p_user_id AND chat_id = p_chat_id;
    
    -- Count unread mentions
    IF v_last_read_id IS NULL THEN
        -- Count all mentions if no messages read yet
        SELECT COUNT(*) INTO v_mentions_count
        FROM message_mentions mm
        JOIN messages m ON m.id = mm.message_id
        WHERE mm.mentioned_user_id = p_user_id
        AND m.chat_id = p_chat_id;
    ELSE
        -- Count mentions after last read message
        SELECT COUNT(*) INTO v_mentions_count
        FROM message_mentions mm
        JOIN messages m ON m.id = mm.message_id
        WHERE mm.mentioned_user_id = p_user_id
        AND m.chat_id = p_chat_id
        AND m.id > v_last_read_id;
    END IF;
    
    RETURN COALESCE(v_mentions_count, 0);
END;
$$ LANGUAGE plpgsql;

-- Enhanced function to update last read message with mentions
CREATE OR REPLACE FUNCTION update_last_read_message_with_mentions(
    p_user_id BIGINT, 
    p_chat_id BIGINT, 
    p_message_id BIGINT
) RETURNS VOID AS $$
BEGIN
    -- Update last read message
    UPDATE chat_members
    SET last_read_message_id = p_message_id,
        last_read_at = NOW(),
        unread_mentions_count = get_unread_mentions_count(p_user_id, p_chat_id)
    WHERE user_id = p_user_id 
    AND chat_id = p_chat_id
    AND (last_read_message_id IS NULL OR last_read_message_id < p_message_id);
    
    -- Update message receipts
    INSERT INTO message_receipts (message_id, user_id, status, timestamp)
    VALUES (p_message_id, p_user_id, 'read', NOW())
    ON CONFLICT (message_id, user_id, status) 
    DO UPDATE SET timestamp = NOW();
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically extract mentions when a message is inserted
CREATE OR REPLACE FUNCTION trigger_extract_mentions() RETURNS TRIGGER AS $$
BEGIN
    PERFORM extract_and_store_mentions(NEW.id, NEW.content, NEW.chat_id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic mention extraction
DROP TRIGGER IF EXISTS auto_extract_mentions ON messages;
CREATE TRIGGER auto_extract_mentions
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION trigger_extract_mentions();