-- Add left_at column to chat_members table
-- This column is used to track when a user leaves a chat.
ALTER TABLE chat_members
ADD COLUMN IF NOT EXISTS left_at TIMESTAMPTZ;

-- Create an index on left_at to speed up queries for active members
CREATE INDEX IF NOT EXISTS idx_chat_members_left_at ON chat_members (left_at); 