\c fechatter_db

-- 查看当前工作区状态
SELECT * FROM workspaces;

-- 查看用户列表
SELECT id, fullname, email, workspace_id, status FROM users;

-- 确保工作区4有正确的拥有者
UPDATE workspaces SET owner_id = 7 WHERE id = 4;

-- 确保用户7的工作区ID正确
UPDATE users SET workspace_id = 4 WHERE id = 7; 