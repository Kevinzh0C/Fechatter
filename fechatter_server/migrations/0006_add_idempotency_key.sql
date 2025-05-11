-- Add idempotency_key to messages table
ALTER TABLE messages ADD COLUMN idempotency_key UUID UNIQUE;

-- Create index for fast lookups by idempotency_key
CREATE INDEX idx_message_idempotency ON messages(idempotency_key);

-- Add a trigger function to prevent duplicate messages
CREATE OR REPLACE FUNCTION prevent_duplicate_messages()
RETURNS TRIGGER AS $$
BEGIN
  -- If idempotency_key already exists for this chat_id and sender_id, don't insert new record
  IF EXISTS (
    SELECT 1 FROM messages 
    WHERE idempotency_key = NEW.idempotency_key 
    AND chat_id = NEW.chat_id 
    AND sender_id = NEW.sender_id
    AND id <> NEW.id
  ) THEN
    RAISE NOTICE 'Duplicate message with idempotency_key % detected, skipping insert', NEW.idempotency_key;
    
    -- Notify about the duplicate attempt
    PERFORM pg_notify('duplicate_message_attempted', json_build_object(
      'idempotency_key', NEW.idempotency_key,
      'chat_id', NEW.chat_id,
      'sender_id', NEW.sender_id
    )::text);
    
    RETURN NULL;  -- Skip this insert
  END IF;
  
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to run before insert
CREATE TRIGGER prevent_duplicate_messages_trigger
BEFORE INSERT ON messages
FOR EACH ROW
EXECUTE FUNCTION prevent_duplicate_messages(); 