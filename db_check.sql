\c fechatter_db

-- 检查和修复工作区
SELECT * FROM workspaces;
SELECT * FROM users;

-- 如果工作区为空或owner_id为0，那么更新或添加工作区
INSERT INTO workspaces (name, owner_id) 
VALUES ('Default Workspace', (SELECT id FROM users WHERE email = 'test5@example.com' LIMIT 1))
ON CONFLICT (name) DO UPDATE 
SET owner_id = (SELECT id FROM users WHERE email = 'test5@example.com' LIMIT 1)
WHERE workspaces.owner_id = 0;

-- 确保用户有工作区ID
UPDATE users SET workspace_id = (SELECT id FROM workspaces WHERE name = 'Default Workspace')
WHERE email = 'test5@example.com';

-- 查看更新后的表
SELECT * FROM workspaces;
SELECT * FROM users WHERE email = 'test5@example.com'; 