# 🚀 Fechatter 开发服务器使用指南

## 📱 启动方式

### 1. 标准方式 (推荐)
```bash
yarn dev
```
- ✅ **不会自动打开浏览器**
- ✅ 显示 URL 供手动访问
- ✅ 复制配置文件
- ✅ 启动 Vite 开发服务器

### 2. 使用自定义脚本
```bash
# 使用 bash 脚本 (更友好的输出)
./start-dev.sh

# 或使用 Node.js 脚本
node dev-start.js
```

### 3. 如果需要自动打开浏览器
```bash
# 在命令后添加 --open 参数
yarn dev --open

# 或者直接使用 Vite
yarn vite --open
```

## 🌐 访问地址

开发服务器启动后，您可以在浏览器中访问：

- **本地地址**: http://localhost:5173
- **网络地址**: http://192.168.x.x:5173 (具体IP请看控制台输出)

## 📝 Vite 配置修改

在 `vite.config.js` 中已修改：
```javascript
server: {
  port: 5173,
  open: false, // 🚀 不自动打开浏览器
  // ... 其他配置
}
```

## 🔧 其他开发命令

```bash
# 清理构建并启动
yarn dev:clean

# 启动时包含机器人服务
yarn dev:with-bot

# 仅启动机器人服务
yarn dev:bot-only

# 预览构建结果
yarn preview

# 构建生产版本
yarn build
```

## 💡 使用建议

1. **首次启动**: 使用 `yarn dev`，然后手动复制显示的 URL 到浏览器
2. **网络访问**: 使用 Vite 输出中的 Network 地址可以让同网络设备访问
3. **停止服务器**: 按 `Ctrl+C` 停止开发服务器

## 🔗 相关文件

- `vite.config.js` - Vite 配置文件
- `start-dev.sh` - Bash 启动脚本
- `dev-start.js` - Node.js 启动脚本 