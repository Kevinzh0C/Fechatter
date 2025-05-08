SELECT * FROM workspaces;
SELECT * FROM users;

-- 更新工作区拥有者（如果工作区存在且owner_id为0）
UPDATE workspaces SET owner_id = (SELECT id FROM users WHERE email = 'test3@example.com' LIMIT 1) 
WHERE owner_id = 0;

-- 查看更新后的工作区
SELECT * FROM workspaces; 