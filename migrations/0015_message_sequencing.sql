-- Add sequence number support for ordered message delivery

-- Add sequence_number to messages table
ALTER TABLE messages
ADD COLUMN sequence_number BIGINT;

-- Create index for efficient sequence-based queries
CREATE INDEX idx_messages_chat_sequence 
ON messages(chat_id, sequence_number DESC)
WHERE sequence_number IS NOT NULL;

-- Create unique constraint for sequence numbers within a chat
CREATE UNIQUE INDEX idx_messages_chat_sequence_unique
ON messages(chat_id, sequence_number)
WHERE sequence_number IS NOT NULL;

-- Create sequence tracking table for gap detection
CREATE TABLE IF NOT EXISTS chat_sequences (
    chat_id BIGINT PRIMARY KEY REFERENCES chats(id) ON DELETE CASCADE,
    last_sequence BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create message acknowledgments table (replacing message_receipts if needed)
CREATE TABLE IF NOT EXISTS message_acknowledgments (
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    delivered_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    PRIMARY KEY (message_id, user_id)
);

-- Create partial index for undelivered messages
CREATE INDEX idx_acks_user_delivered 
ON message_acknowledgments(user_id, delivered_at)
WHERE delivered_at IS NOT NULL;

-- Create index for read receipts
CREATE INDEX idx_acks_user_read
ON message_acknowledgments(user_id, read_at)
WHERE read_at IS NOT NULL;

-- Function to get next sequence number
CREATE OR REPLACE FUNCTION next_message_sequence(p_chat_id BIGINT)
RETURNS BIGINT AS $$
DECLARE
    v_sequence BIGINT;
BEGIN
    INSERT INTO chat_sequences (chat_id, last_sequence, updated_at)
    VALUES (p_chat_id, 1, NOW())
    ON CONFLICT (chat_id) DO UPDATE
    SET last_sequence = chat_sequences.last_sequence + 1,
        updated_at = NOW()
    RETURNING last_sequence INTO v_sequence;
    
    RETURN v_sequence;
END;
$$ LANGUAGE plpgsql;

-- Backfill existing messages with sequence numbers
-- Backfill existing messages with sequence numbers using window functions
DO $$
DECLARE
    r RECORD;
    max_seq BIGINT;
BEGIN
    -- First, update all messages with sequence numbers using window function
    WITH numbered_messages AS (
        SELECT 
            id,
            chat_id,
            ROW_NUMBER() OVER (PARTITION BY chat_id ORDER BY created_at, id) AS row_num
        FROM messages
        WHERE sequence_number IS NULL
    )
    UPDATE messages m
    SET sequence_number = nm.row_num
    FROM numbered_messages nm
    WHERE m.id = nm.id;
    
    -- Then update chat_sequences table with max sequence per chat
    FOR r IN (
        SELECT DISTINCT chat_id 
        FROM messages 
        WHERE sequence_number IS NOT NULL
        ORDER BY chat_id
    ) LOOP
        -- Get the maximum sequence number for this chat
        SELECT MAX(sequence_number) INTO max_seq
        FROM messages
        WHERE chat_id = r.chat_id;
        
        -- Update chat_sequences with the max sequence
        INSERT INTO chat_sequences (chat_id, last_sequence, updated_at)
        VALUES (r.chat_id, max_seq, NOW())
        ON CONFLICT (chat_id) DO UPDATE
        SET last_sequence = GREATEST(chat_sequences.last_sequence, max_seq),
            updated_at = NOW();
    END LOOP;
END;
$$;

-- Make sequence_number NOT NULL after backfill
ALTER TABLE messages
ALTER COLUMN sequence_number SET NOT NULL; 