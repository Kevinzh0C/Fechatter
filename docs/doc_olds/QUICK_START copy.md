# Fechatter 快速启动指南

## 🚀 一键启动所有服务

```bash
# 1. 克隆项目
git clone https://github.com/your-org/fechatter.git
cd fechatter

# 2. 复制环境变量配置
cp env.example .env

# 3. 编辑 .env 文件，至少设置 OPENAI_API_KEY
# nano .env

# 4. 启动所有服务
docker-compose up -d

# 5. 等待服务启动完成（约30秒）
sleep 30

# 6. 检查服务状态
docker-compose ps

# 7. 访问应用
# 打开浏览器访问 http://localhost:8080
```

## 📋 服务启动顺序

1. **基础设施服务**（自动按依赖顺序启动）
   - PostgreSQL → Redis → NATS → Meilisearch → ClickHouse

2. **应用服务**
   - Fechatter Server → Notify Server → Bot Server → Analytics Server

3. **API 网关**
   - Gateway（最后启动，依赖所有其他服务）

## 🔍 验证服务状态

### 检查所有服务
```bash
# 查看容器状态
docker-compose ps

# 查看服务日志
docker-compose logs -f --tail=50
```

### 健康检查
```bash
# API 网关健康检查
curl http://localhost:8080/health

# 直接访问各服务（开发模式）
curl http://localhost:6688/health  # Fechatter Server
curl http://localhost:6687/health  # Notify Server
curl http://localhost:6686/health  # Bot Server
curl http://localhost:6690/health  # Analytics Server
```

### 基础设施服务检查
```bash
# PostgreSQL
docker exec -it fechatter-postgres psql -U postgres -c "SELECT 1"

# Redis
docker exec -it fechatter-redis redis-cli -a fechatter_redis_pass ping

# NATS
curl http://localhost:8222/varz | jq .

# Meilisearch
curl http://localhost:7700/health

# ClickHouse
curl http://localhost:8123/ping
```

## 🛠️ 常见问题

### 1. 端口冲突
如果端口已被占用，修改 `docker-compose.yml` 中的端口映射：
```yaml
ports:
  - "18080:8080"  # 改为其他端口
```

### 2. OpenAI API 错误
确保在 `.env` 文件中设置了有效的 `OPENAI_API_KEY`

### 3. 数据库连接失败
```bash
# 重启数据库
docker-compose restart postgres

# 查看数据库日志
docker-compose logs postgres
```

### 4. 内存不足
增加 Docker 内存限制或减少服务：
```bash
# 只启动核心服务
docker-compose up -d postgres redis fechatter-server notify-server gateway
```

## 📝 开发模式快速启动

如果你想在本地开发：

```bash
# 1. 只启动基础设施
docker-compose up -d postgres redis nats meilisearch clickhouse

# 2. 启动后端服务（新终端）
make dev

# 3. 启动前端（新终端）
cd fechatter_frontend
yarn install
yarn dev

# 访问 http://localhost:5173
```

## 🔄 更新和重启

```bash
# 拉取最新代码
git pull

# 重新构建镜像
docker-compose build

# 重启服务
docker-compose down
docker-compose up -d
```

## 📊 监控服务

```bash
# 实时查看资源使用
docker stats

# 查看特定服务日志
docker-compose logs -f fechatter-server

# 使用 tmux 监控所有服务
tmux new-session -d -s monitor "docker-compose logs -f"
tmux attach -t monitor
```

## 🧹 清理

```bash
# 停止所有服务
docker-compose down

# 清理数据（谨慎！）
docker-compose down -v

# 清理所有 Docker 资源
docker system prune -a
```

## 📚 下一步

- 查看 [部署文档](./DEPLOYMENT.md) 了解详细配置
- 查看 [微服务架构](./MICROSERVICES_ARCHITECTURE.md) 了解系统设计
- 查看 [API 文档](http://localhost:8080/docs) (服务启动后) 