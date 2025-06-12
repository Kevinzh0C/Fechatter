-- Remove default time values and clean up timestamps
-- Migration: 0011_remove_times_defaults.sql

-- Remove any problematic default timestamp values that might cause issues
-- Update existing records with NULL timestamps to use proper defaults

-- Clean up messages table timestamps
UPDATE messages 
SET created_at = NOW() 
WHERE created_at IS NULL;

UPDATE messages 
SET updated_at = created_at 
WHERE updated_at IS NULL;

-- Clean up chats table timestamps  
UPDATE chats 
SET created_at = NOW() 
WHERE created_at IS NULL;

UPDATE chats 
SET updated_at = created_at 
WHERE updated_at IS NULL;

-- Clean up users table timestamps
UPDATE users 
SET created_at = NOW() 
WHERE created_at IS NULL;

UPDATE users 
SET updated_at = created_at 
WHERE updated_at IS NULL;

-- Clean up chat_members table timestamps
UPDATE chat_members 
SET joined_at = NOW() 
WHERE joined_at IS NULL;

-- Ensure all timestamp columns are properly NOT NULL after cleanup
ALTER TABLE messages 
ALTER COLUMN created_at SET NOT NULL,
ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE chats 
ALTER COLUMN created_at SET NOT NULL,
ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE users 
ALTER COLUMN created_at SET NOT NULL,
ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE chat_members 
ALTER COLUMN joined_at SET NOT NULL;

-- Add triggers to automatically update updated_at columns
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply the trigger to relevant tables
DROP TRIGGER IF EXISTS update_messages_updated_at ON messages;
CREATE TRIGGER update_messages_updated_at
    BEFORE UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_chats_updated_at ON chats;
CREATE TRIGGER update_chats_updated_at
    BEFORE UPDATE ON chats
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();