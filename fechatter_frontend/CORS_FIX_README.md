# CORS Configuration Fix for Port 5173

## 问题
前端现在运行在 `localhost:5173`，但后端CORS配置仍允许 `localhost:1420`。

## 解决方案

### 1. 前端修复 ✅ 已完成
所有直接访问远程服务器的代码已修改为通过vite代理：
- `api.js` - 使用 `/api` 相对路径  
- `sse-minimal.js` - 使用 `/events` 相对路径
- `sse.js` - 使用 `/events` 相对路径
- 所有工具文件 - 使用相对路径

### 2. 后端CORS配置需要更新

#### 方式1: 更新后端代码 (推荐)
在后端CORS配置中添加新端口：

```rust
// 在 fechatter_server/src/main.rs 或相关配置文件中
allowed_origins: vec![
    "http://localhost:1420",  // 保留旧的
    "http://localhost:5173",  // 添加新的
    "http://127.0.0.1:1420", 
    "http://127.0.0.1:5173"
],
```

#### 方式2: 环境变量配置
```bash
# 设置允许的源
export CORS_ALLOWED_ORIGINS="http://localhost:1420,http://localhost:5173"
```

#### 方式3: 配置文件更新
如果使用配置文件，更新 CORS 设置：
```yaml
cors:
  allowed_origins:
    - "http://localhost:1420"
    - "http://localhost:5173"
    - "http://127.0.0.1:1420" 
    - "http://127.0.0.1:5173"
```

## 验证修复
1. 重启后端服务
2. 启动前端: `yarn dev`
3. 检查浏览器控制台无CORS错误
4. 测试登录和实时功能

## 临时解决方案
如果无法立即更新后端，可以临时使用：
```bash
# 启动前端在旧端口
yarn dev --port 1420
```

但建议使用新端口5173以保持一致性。 