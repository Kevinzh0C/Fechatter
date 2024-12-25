-- Add migration script here

-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces (
  id bigserial PRIMARY KEY,
  name VARCHAR(32) NOT NULL UNIQUE,
  owner_id bigint NOT NULL REFERENCES users(id),
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- alert users table to add workspace_id
ALTER TABLE users 
  ADD COLUMN workspace_id bigint REFERENCES workspaces(id);

-- add super user 0 and workspace 0 
BEGIN;
INSERT INTO users (id, fullname, email, password_hash, status)
  VALUES (0, 'super user', 'super@none.org', 'superuser', 'active');
INSERT INTO workspaces(id, name, owner_id)
  VALUES (0, 'none', 0);
UPDATE 
  users
SET 
  workspace_id = 0
WHERE 
  id = 0;
COMMIT;

-- alter users table to set workspace_id not null
ALTER TABLE users 
  ALTER COLUMN workspace_id SET NOT NULL;

SELECT version();

DO $$
BEGIN
EXCEPTION WHEN OTHERS THEN
  RAISE EXCEPTION 'Migration failed: %', SQLERRM;
END $$;





