-- Fix chat members consistency between chats.chat_members array and chat_members table
-- Migration: 0018_fix_chat_members_consistency.sql

-- Step 0: Clean up invalid user IDs from chats.chat_members array
UPDATE chats 
SET chat_members = COALESCE(
  (SELECT ARRAY_AGG(member_id ORDER BY member_id)
   FROM LATERAL unnest(chat_members) AS member_id
   WHERE EXISTS (SELECT 1 FROM users u WHERE u.id = member_id)),
  ARRAY[]::bigint[]
)
WHERE chat_members IS NOT NULL 
AND array_length(chat_members, 1) > 0;

-- Step 1: Update chats.chat_members array to only include active members (left_at IS NULL)
UPDATE chats 
SET chat_members = COALESCE(
  (SELECT ARRAY_AGG(cm.user_id ORDER BY cm.user_id) 
   FROM chat_members cm 
   WHERE cm.chat_id = chats.id 
   AND cm.left_at IS NULL),
  ARRAY[]::bigint[]
)
WHERE EXISTS (
  SELECT 1 FROM chat_members cm 
  WHERE cm.chat_id = chats.id
);

-- Step 2: For any chat_members records that exist but are not in chats.chat_members, 
-- mark them as left (this handles orphaned records)
UPDATE chat_members 
SET left_at = NOW()
WHERE left_at IS NULL 
AND NOT EXISTS (
  SELECT 1 FROM chats c 
  WHERE c.id = chat_members.chat_id 
  AND chat_members.user_id = ANY(c.chat_members)
);

-- Step 3: Add any missing chat_members records for users in chats.chat_members array
-- Use a subquery approach to avoid CASE with set-returning functions
-- Only process users that actually exist in the users table
WITH chat_member_data AS (
  SELECT 
    c.id as chat_id,
    member_id as user_id,
    c.created_by,
    c.created_at as joined_at
  FROM chats c,
  LATERAL unnest(c.chat_members) AS member_id
  WHERE c.chat_members IS NOT NULL 
  AND array_length(c.chat_members, 1) > 0
  AND EXISTS (SELECT 1 FROM users u WHERE u.id = member_id) -- Ensure user exists
)
INSERT INTO chat_members (chat_id, user_id, role, joined_at)
SELECT 
  cmd.chat_id,
  cmd.user_id,
  CASE 
    WHEN cmd.user_id = cmd.created_by THEN 'owner'::chat_member_role
    ELSE 'member'::chat_member_role 
  END as role,
  cmd.joined_at
FROM chat_member_data cmd
ON CONFLICT (chat_id, user_id) 
DO UPDATE SET 
  left_at = NULL,
  role = CASE 
    WHEN EXCLUDED.user_id = (SELECT created_by FROM chats WHERE id = EXCLUDED.chat_id) 
    THEN 'owner'::chat_member_role
    ELSE 'member'::chat_member_role 
  END
WHERE chat_members.left_at IS NOT NULL;

-- Step 4: Final consistency check - update chats.chat_members again to ensure perfect sync
UPDATE chats 
SET chat_members = COALESCE(
  (SELECT ARRAY_AGG(cm.user_id ORDER BY cm.user_id) 
   FROM chat_members cm 
   WHERE cm.chat_id = chats.id 
   AND cm.left_at IS NULL),
  ARRAY[]::bigint[]
);

-- Add helpful indexes for performance
CREATE INDEX IF NOT EXISTS idx_chat_members_active 
ON chat_members (chat_id, user_id) 
WHERE left_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_chat_members_left_at_null 
ON chat_members (chat_id) 
WHERE left_at IS NULL; 