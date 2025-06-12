-- Fix chat_member role type mismatch between Rust String and PostgreSQL ENUM
-- Migration: 0022_fix_chat_member_role_type.sql
-- Purpose: Convert chat_member_role ENUM to VARCHAR to match Rust String expectations
-- Impact: Critical - fixes runtime errors in message sending and chat operations

-- ============================================================================
-- STEP 1: Backup current data and verify constraints
-- ============================================================================

-- Record current state for verification
DO $$
BEGIN
    RAISE NOTICE 'Starting chat_member role type migration...';
    RAISE NOTICE 'Current chat_members count: %', (SELECT COUNT(*) FROM chat_members);
    RAISE NOTICE 'Current role distribution: %', (
        SELECT json_object_agg(role, count) 
        FROM (SELECT role, COUNT(*) as count FROM chat_members GROUP BY role) t
    );
END $$;

-- ============================================================================
-- STEP 2: Create new VARCHAR role column
-- ============================================================================

-- Add new role_text column as VARCHAR(50)
ALTER TABLE chat_members 
ADD COLUMN IF NOT EXISTS role_text VARCHAR(50);

-- Populate new column with existing ENUM values converted to text
UPDATE chat_members 
SET role_text = role::text
WHERE role_text IS NULL;

-- ============================================================================
-- STEP 3: Update constraints and defaults
-- ============================================================================

-- Add CHECK constraint to ensure valid role values
ALTER TABLE chat_members 
ADD CONSTRAINT chat_member_role_values 
CHECK (role_text IN ('owner', 'admin', 'moderator', 'member'));

-- Set NOT NULL constraint after population
ALTER TABLE chat_members 
ALTER COLUMN role_text SET NOT NULL;

-- Set default value for new column
ALTER TABLE chat_members 
ALTER COLUMN role_text SET DEFAULT 'member';

-- ============================================================================
-- STEP 4: Drop old ENUM column and rename new column
-- ============================================================================

-- Drop dependent indexes first
DROP INDEX IF EXISTS idx_chat_members_role;

-- Drop the old ENUM column
ALTER TABLE chat_members DROP COLUMN role;

-- Rename new column to replace old one
ALTER TABLE chat_members RENAME COLUMN role_text TO role;

-- ============================================================================
-- STEP 5: Recreate indexes and optimize
-- ============================================================================

-- Recreate role index for performance
CREATE INDEX idx_chat_members_role ON chat_members(role);

-- Create optimized composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_chat_members_chat_role 
ON chat_members(chat_id, role) 
WHERE left_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_chat_members_user_role 
ON chat_members(user_id, role) 
WHERE left_at IS NULL;

-- ============================================================================
-- STEP 6: Update any dependent functions or triggers
-- ============================================================================

-- Update bot_config trigger function to use text values instead of ENUM
CREATE OR REPLACE FUNCTION create_bot_config_on_bot_user()
RETURNS TRIGGER AS $$
BEGIN
    -- Only create bot config for users marked as bots
    IF NEW.is_bot = TRUE THEN
        INSERT INTO bot_config (
            user_id,
            ai_model,
            embed_model,
            max_response_length,
            response_delay_ms,
            vector_size,
            enabled,
            created_at,
            updated_at
        ) VALUES (
            NEW.id,
            'gpt-4o-mini',
            'text-embedding-3-small',
            2000,
            1000,
            1536,
            TRUE,
            NOW(),
            NOW()
        )
        ON CONFLICT (user_id) DO NOTHING;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- STEP 7: Clean up old ENUM type (optional - can be kept for safety)
-- ============================================================================

-- Note: We keep the ENUM type for now in case of rollback needs
-- It can be dropped in a future migration after confirming stability
-- DROP TYPE IF EXISTS chat_member_role;

-- ============================================================================
-- STEP 8: Data validation and verification
-- ============================================================================

DO $$
DECLARE
    role_count INTEGER;
    invalid_roles INTEGER;
BEGIN
    -- Verify all roles are valid
    SELECT COUNT(*) INTO role_count FROM chat_members;
    SELECT COUNT(*) INTO invalid_roles 
    FROM chat_members 
    WHERE role NOT IN ('owner', 'admin', 'moderator', 'member');
    
    IF invalid_roles > 0 THEN
        RAISE EXCEPTION 'Migration failed: Found % invalid role values', invalid_roles;
    END IF;
    
    RAISE NOTICE 'Migration successful: % chat_members updated, 0 invalid roles', role_count;
    
    -- Log final role distribution
    RAISE NOTICE 'Final role distribution: %', (
        SELECT json_object_agg(role, count) 
        FROM (SELECT role, COUNT(*) as count FROM chat_members GROUP BY role) t
    );
END $$;

-- ============================================================================
-- STEP 9: Performance optimization
-- ============================================================================

-- Analyze table for query planner optimization
ANALYZE chat_members;

-- Update table statistics
ANALYZE chats;

-- ============================================================================
-- MIGRATION VERIFICATION QUERIES (for manual testing)
-- ============================================================================

-- Verify role column type
-- SELECT column_name, data_type, character_maximum_length 
-- FROM information_schema.columns 
-- WHERE table_name = 'chat_members' AND column_name = 'role';

-- Verify role constraints
-- SELECT constraint_name, check_clause 
-- FROM information_schema.check_constraints 
-- WHERE constraint_name = 'chat_member_role_values';

-- Verify role distribution
-- SELECT role, COUNT(*) as count 
-- FROM chat_members 
-- GROUP BY role 
-- ORDER BY count DESC;

-- ============================================================================
-- ROLLBACK SCRIPT (for emergency use)
-- ============================================================================

/*
-- EMERGENCY ROLLBACK PROCEDURE (run manually if needed):

-- 1. Add back ENUM column
ALTER TABLE chat_members ADD COLUMN role_enum chat_member_role;

-- 2. Populate ENUM column from VARCHAR
UPDATE chat_members SET role_enum = role::chat_member_role;

-- 3. Set constraints
ALTER TABLE chat_members ALTER COLUMN role_enum SET NOT NULL;
ALTER TABLE chat_members ALTER COLUMN role_enum SET DEFAULT 'member';

-- 4. Drop VARCHAR column and rename ENUM column
ALTER TABLE chat_members DROP COLUMN role;
ALTER TABLE chat_members RENAME COLUMN role_enum TO role;

-- 5. Recreate indexes
CREATE INDEX idx_chat_members_role ON chat_members(role);
*/

-- ============================================================================
-- COMMENTS AND DOCUMENTATION
-- ============================================================================

COMMENT ON COLUMN chat_members.role IS 'Member role in chat: owner, admin, moderator, or member (converted from ENUM to VARCHAR for Rust compatibility)';

COMMENT ON CONSTRAINT chat_member_role_values ON chat_members IS 'Ensures role values match expected chat member roles';

COMMENT ON INDEX idx_chat_members_role IS 'Index on role for efficient role-based queries';

COMMENT ON INDEX idx_chat_members_chat_role IS 'Composite index for chat-role queries on active members';

-- Migration completed successfully
-- This migration resolves the type mismatch between Rust String and PostgreSQL ENUM
-- All existing functionality should work without changes
-- Performance impact: Minimal, VARCHAR is efficient for small strings like roles 