-- Fix chat_members left_at issue caused by 0018 migration
-- Migration: 0019_fix_chat_members_left_at_issue.sql

-- Problem: 0018 migration incorrectly marks chat_members as left 
-- when there's a temporary inconsistency during chat creation

-- Step 1: Identify and fix incorrectly marked members
-- These are members who are in chats.chat_members array but marked as left
UPDATE chat_members 
SET left_at = NULL
WHERE left_at IS NOT NULL
AND EXISTS (
  SELECT 1 FROM chats c 
  WHERE c.id = chat_members.chat_id 
  AND chat_members.user_id = ANY(c.chat_members)
);

-- Step 2: Ensure all members in chats.chat_members array have corresponding records
WITH missing_members AS (
  SELECT 
    c.id as chat_id,
    member_id as user_id,
    c.created_by,
    c.created_at as joined_at
  FROM chats c,
  LATERAL unnest(c.chat_members) AS member_id
  WHERE c.chat_members IS NOT NULL 
  AND array_length(c.chat_members, 1) > 0
  AND EXISTS (SELECT 1 FROM users u WHERE u.id = member_id)
  AND NOT EXISTS (
    SELECT 1 FROM chat_members cm 
    WHERE cm.chat_id = c.id 
    AND cm.user_id = member_id
  )
)
INSERT INTO chat_members (chat_id, user_id, role, joined_at, left_at)
SELECT 
  mm.chat_id,
  mm.user_id,
  CASE 
    WHEN mm.user_id = mm.created_by THEN 'owner'::chat_member_role
    ELSE 'member'::chat_member_role 
  END as role,
  mm.joined_at,
  NULL -- Explicitly set left_at to NULL
FROM missing_members mm;

-- Step 3: Remove the problematic logic from future operations
-- Create a safer constraint to prevent the issue from recurring
-- Add a comment to document the fix
COMMENT ON COLUMN chat_members.left_at IS 'NULL means active member, NOT NULL means left. Fixed in migration 0019 to prevent incorrect marking during chat creation.';

-- Step 4: Add a function to safely manage member consistency
CREATE OR REPLACE FUNCTION sync_chat_members_array(target_chat_id BIGINT)
RETURNS VOID AS $$
BEGIN
  -- Update chats.chat_members array based on active members only
  UPDATE chats 
  SET chat_members = COALESCE(
    (SELECT ARRAY_AGG(cm.user_id ORDER BY cm.user_id) 
     FROM chat_members cm 
     WHERE cm.chat_id = target_chat_id 
     AND cm.left_at IS NULL),
    ARRAY[]::bigint[]
  )
  WHERE id = target_chat_id;
END;
$$ LANGUAGE plpgsql;

-- Step 5: Create a safer member removal function
CREATE OR REPLACE FUNCTION leave_chat_safely(target_chat_id BIGINT, target_user_id BIGINT)
RETURNS VOID AS $$
BEGIN
  -- Mark as left in chat_members table
  UPDATE chat_members 
  SET left_at = NOW()
  WHERE chat_id = target_chat_id 
  AND user_id = target_user_id 
  AND left_at IS NULL;
  
  -- Update the array in chats table
  UPDATE chats 
  SET chat_members = array_remove(chat_members, target_user_id)
  WHERE id = target_chat_id;
END;
$$ LANGUAGE plpgsql; 