# 🚀 Fechatter 简单命令指南

## 最简单的使用方式

项目默认使用 **Podman**，如果你想使用 Docker，可以通过环境变量覆盖。

### 🎯 一键启动（推荐）
```bash
make quick-start
```
这个命令会：
- 构建所有服务
- 启动所有容器
- 显示服务地址
- 给出后续操作提示

### 🔧 常用命令

#### 构建
```bash
# 构建所有服务（并行构建，最快）
make build-all

# 或使用 compose
podman-compose build
```

#### 启动/停止
```bash
# 启动服务
make up
# 或
podman-compose up -d

# 停止服务
make down
# 或
podman-compose down
```

#### 查看状态
```bash
# 查看运行状态
make ps

# 查看日志
make logs

# 查看特定服务日志
make logs-fechatter-server
```

#### 重建
```bash
# 完全重建（停止 → 构建 → 启动）
make rebuild
```

### 🔄 切换容器工具

#### 使用 Docker（临时）
```bash
# 临时使用 Docker
make DOCKER=docker DOCKER_COMPOSE=docker-compose quick-start

# 或设置环境变量
export DOCKER=docker
export DOCKER_COMPOSE=docker-compose
make quick-start
```

#### 使用 Docker（永久）
在你的 shell 配置文件中添加：
```bash
# ~/.zshrc 或 ~/.bashrc
export DOCKER=docker
export DOCKER_COMPOSE=docker-compose
```

### 🛠️ 开发模式

如果你想在本地开发而不用容器：
```bash
# 启动开发服务（使用 tmux）
make dev

# 停止开发服务
make dev-stop
```

### 📊 监控

```bash
# 查看容器资源使用
make stats

# 查看镜像大小
podman images | grep fechatter
```

## 🎯 最常用的工作流

### 第一次使用
```bash
# 1. 克隆项目后，一键启动
make quick-start

# 2. 查看日志确认启动成功
make logs
```

### 日常开发
```bash
# 修改代码后重建
make rebuild

# 或者只重建特定服务
podman-compose build fechatter-server
podman-compose up -d fechatter-server
```

### 清理
```bash
# 停止所有服务
make down

# 清理构建缓存
make clean
```

## 🔧 环境配置

确保你有 `.env` 文件：
```bash
cp env.example .env
# 编辑 .env 文件，设置必要的环境变量
```

必须设置的变量：
- `OPENAI_API_KEY` - OpenAI API 密钥
- `JWT_SECRET` - JWT 签名密钥

## 📝 总结

**最简单的使用方式就是：**
1. `make quick-start` - 一键启动所有服务（默认使用 Podman）
2. `make logs` - 查看日志
3. `make down` - 停止服务

**容器工具选择：**
- 默认：Podman（更安全，无需 daemon）
- 可选：Docker（通过环境变量切换）
- 灵活：支持临时切换或永久配置 