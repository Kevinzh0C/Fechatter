-- Group and chat enhancements
-- Migration: 0009_group_enhancement.sql

-- Add group/chat specific features
ALTER TABLE chats 
ADD COLUMN IF NOT EXISTS max_members INTEGER DEFAULT 100,
ADD COLUMN IF NOT EXISTS is_public BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS invite_code VARCHAR(50) UNIQUE,
ADD COLUMN IF NOT EXISTS settings JSONB DEFAULT '{}';

-- Add chat member roles and permissions
CREATE TYPE chat_member_role AS ENUM ('owner', 'admin', 'moderator', 'member');

ALTER TABLE chat_members 
ADD COLUMN IF NOT EXISTS role chat_member_role DEFAULT 'member',
ADD COLUMN IF NOT EXISTS permissions JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS muted_until TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS is_banned BOOLEAN DEFAULT FALSE;

-- Add group-specific tables
CREATE TABLE IF NOT EXISTS chat_invites (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    inviter_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invitee_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    invite_code VARCHAR(50),
    expires_at TIMESTAMPTZ,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Chat announcements
CREATE TABLE IF NOT EXISTS chat_announcements (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    author_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    is_pinned BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_chats_is_public ON chats(is_public);
CREATE INDEX IF NOT EXISTS idx_chats_invite_code ON chats(invite_code);
CREATE INDEX IF NOT EXISTS idx_chat_members_role ON chat_members(role);
CREATE INDEX IF NOT EXISTS idx_chat_invites_chat_id ON chat_invites(chat_id);
CREATE INDEX IF NOT EXISTS idx_chat_invites_invite_code ON chat_invites(invite_code);
CREATE INDEX IF NOT EXISTS idx_chat_announcements_chat_id ON chat_announcements(chat_id);