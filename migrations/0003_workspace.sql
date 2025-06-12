-- Add migration script here

-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(32) NOT NULL UNIQUE,
  owner_id BIGINT NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- alert users table to add workspace_id
ALTER TABLE users 
  ADD COLUMN workspace_id BIGINT REFERENCES workspaces(id);

-- Add workspace_id column to chats table
ALTER TABLE chats ADD COLUMN workspace_id BIGINT NOT NULL DEFAULT 1;

-- Add foreign key constraint
ALTER TABLE chats ADD CONSTRAINT fk_chats_workspace
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
    ON DELETE CASCADE; 

-- add super user 0 and workspace 0 
BEGIN;
-- First, temporarily allow NULL in workspace_id for initial setup
ALTER TABLE users ALTER COLUMN workspace_id DROP NOT NULL;

-- Insert system user first (without workspace_id)
INSERT INTO users (id, fullname, email, password_hash, status)
  VALUES (0, 'super user', 'super@none.org', 'superuser', 'active')
  ON CONFLICT (id) DO NOTHING;

-- Now create the system workspace
INSERT INTO workspaces(id, name, owner_id)
  VALUES (0, 'none', 0)
  ON CONFLICT (id) DO NOTHING;

-- Create default workspace for regular users
INSERT INTO workspaces(id, name, owner_id)
  VALUES (1, 'default', 0)
  ON CONFLICT (id) DO NOTHING;

-- Update the system user to have workspace_id
UPDATE users SET workspace_id = 0 WHERE id = 0;

-- Update any existing users without workspace to use default workspace
UPDATE users SET workspace_id = 1 WHERE workspace_id IS NULL AND id != 0;

-- Update sequence to ensure next ID is correct
SELECT setval('users_id_seq', GREATEST((SELECT MAX(id) FROM users), 1));
SELECT setval('workspaces_id_seq', GREATEST((SELECT MAX(id) FROM workspaces), 1));
COMMIT;

-- alter users table to set workspace_id not null
ALTER TABLE users 
  ALTER COLUMN workspace_id SET NOT NULL;





