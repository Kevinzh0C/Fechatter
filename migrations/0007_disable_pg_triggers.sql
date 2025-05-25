-- Disable PostgreSQL triggers when using NATS for event publishing
-- This migration can be run when NATS messaging is enabled

-- 禁用消息创建触发器
DROP TRIGGER IF EXISTS add_message_to_chat_trigger ON messages;

-- 禁用聊天成员变更触发器
DROP TRIGGER IF EXISTS add_to_chat_trigger ON chat_members;

-- 禁用重复消息尝试触发器
DROP TRIGGER IF EXISTS prevent_duplicate_messages_trigger ON messages;

-- 保留触发器函数，以备需要回滚到PostgreSQL NOTIFY机制
-- 如果需要重新启用PostgreSQL触发器，可以运行以下命令：

-- CREATE TRIGGER add_message_to_chat_trigger
--   AFTER INSERT ON messages
--   FOR EACH ROW 
--   EXECUTE FUNCTION add_message_to_chat();

-- CREATE TRIGGER add_to_chat_trigger
--   AFTER INSERT OR UPDATE OR DELETE 
--   ON chat_members
--   FOR EACH ROW 
--   EXECUTE FUNCTION add_to_chat();

-- CREATE TRIGGER prevent_duplicate_messages_trigger
--   BEFORE INSERT ON messages
--   FOR EACH ROW
--   EXECUTE FUNCTION prevent_duplicate_messages(); 