# Scripts Directory

## 📁 目录结构

### build/
构建相关脚本
- `build-cross.sh` - 跨平台构建
- `build-local.sh` - 本地构建
- `build-musl.sh` - MUSL静态链接构建

### deployment/
部署相关脚本
- `deploy-fechatter-server.sh` - 服务器部署
- `global-health-check.sh` - 健康检查

### utils/
工具脚本
- `bulk_search_sync.sh` - 批量搜索同步
- `filter_*.sh` - 各种过滤工具
- `fix_*.sh` - 修复工具

## 🚀 使用方法

所有脚本都应该从项目根目录执行：
```bash
# 构建
./scripts/build/build-local.sh

# 部署
./scripts/deployment/deploy-fechatter-server.sh

# 工具
./scripts/utils/bulk_search_sync.sh
```
