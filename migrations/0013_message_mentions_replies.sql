-- Add support for @mentions and message replies
-- Optimized for 200DAU enterprise chat platform

-- Add message reply support
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS reply_to BIGINT REFERENCES messages(id) ON DELETE SET NULL;

-- Add support for @mentioned users list
ALTER TABLE messages
ADD COLUMN IF NOT EXISTS mentions BIGINT[] DEFAULT '{}';

-- Create indexes to optimize query performance
CREATE INDEX IF NOT EXISTS idx_messages_reply_to ON messages(reply_to) WHERE reply_to IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_messages_mentions ON messages USING GIN(mentions) WHERE array_length(mentions, 1) > 0;

-- Create mention records table for efficient querying of who was mentioned
CREATE TABLE IF NOT EXISTS message_mentions (
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    mentioned_user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (message_id, mentioned_user_id)
);

-- Add indexes to optimize mention queries
CREATE INDEX IF NOT EXISTS idx_message_mentions_user ON message_mentions(mentioned_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_message_mentions_message ON message_mentions(message_id);

-- Create trigger to automatically maintain the message_mentions table
CREATE OR REPLACE FUNCTION update_message_mentions()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete old mention records
    DELETE FROM message_mentions WHERE message_id = NEW.id;
    
    -- If there are new mentions, insert records
    IF NEW.mentions IS NOT NULL AND array_length(NEW.mentions, 1) > 0 THEN
        INSERT INTO message_mentions (message_id, mentioned_user_id)
        SELECT NEW.id, unnest(NEW.mentions);
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger
DROP TRIGGER IF EXISTS message_mentions_trigger ON messages;
CREATE TRIGGER message_mentions_trigger
    AFTER INSERT OR UPDATE OF mentions ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_message_mentions();

-- Add function for querying reply chains
CREATE OR REPLACE FUNCTION get_message_thread(p_message_id BIGINT)
RETURNS TABLE(
    id BIGINT,
    content TEXT,
    sender_id BIGINT,
    created_at TIMESTAMPTZ,
    level INTEGER
) AS $$
BEGIN
    RETURN QUERY
    WITH RECURSIVE message_thread AS (
        -- Find root message
        SELECT 
            m.id,
            m.content,
            m.sender_id,
            m.created_at,
            0 as level
        FROM messages m
        WHERE m.id = p_message_id
        
        UNION ALL
        
        -- Recursively find replies
        SELECT 
            m.id,
            m.content,
            m.sender_id,
            m.created_at,
            mt.level + 1
        FROM messages m
        INNER JOIN message_thread mt ON m.reply_to = mt.id
        WHERE mt.level < 10 -- Limit recursion depth to prevent infinite loops
    )
    SELECT * FROM message_thread ORDER BY level, created_at;
END;
$$ LANGUAGE plpgsql;