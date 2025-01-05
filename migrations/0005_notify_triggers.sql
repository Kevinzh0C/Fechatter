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
  ON chat_members_relation
  FOR EACH ROW 
  EXECUTE FUNCTION add_to_chat();

-- if new message added to a chat, notify with message data
CREATE OR REPLACE FUNCTION add_message_to_chat() 
RETURNS TRIGGER AS $$
BEGIN
  RAISE NOTICE 'Adding message to chat: %', NEW;
  IF TG_OP = 'INSERT' THEN
    PERFORM pg_notify('message_created', row_to_json(NEW)::text);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_message_to_chat_trigger
  AFTER INSERT OR UPDATE OR DELETE 
  ON messages
  FOR EACH ROW 
  EXECUTE FUNCTION add_message_to_chat();
