-- Add migration script here
-- if chat is updated, notify with chat_updated
CREATE OR REPLACE FUNCTION add_to_chat() 
RETURNS TRIGGER AS $$
BEGIN
  RAISE NOTICE 'Adding user to chat: %', NEW;
  PERFORM pg_notify('chat_updated', json_build_object(
    'op', TG_OP,
    'old', OLD,
    'new', NEW
  )::text);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_to_chat_trigger
  AFTER INSERT OR UPDATE OR DELETE 
  ON chat_members
  FOR EACH ROW 
  EXECUTE FUNCTION add_to_chat();

-- if new message added to a chat, notify with message data
CREATE OR REPLACE FUNCTION add_message_to_chat() 
RETURNS TRIGGER AS $$
DECLARE
  USERS BIGINT[];
BEGIN
  IF TG_OP = 'INSERT' THEN
    RAISE NOTICE 'Adding message to chat: %', NEW;
    -- select chat with chat_id in NEW
    SELECT 
      chat_members
    INTO
      USERS
    FROM
      chats
    WHERE
      id = NEW.chat_id;
    PERFORM pg_notify('message_created', json_build_object(
      'message', jsonb_build_object(
        'id', NEW.id,
        'chat_id', NEW.chat_id,
        'sender_id', NEW.sender_id, 
        'content', NEW.content,
        'files', NEW.files,
        'created_at', to_char(NEW.created_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS'),
        'idempotency_key', NEW.idempotency_key
      ), 
      'chat_members', USERS
    )::text);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_message_to_chat_trigger
  AFTER INSERT
  ON messages
  FOR EACH ROW 
  EXECUTE FUNCTION add_message_to_chat();