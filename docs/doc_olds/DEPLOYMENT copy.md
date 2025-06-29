# Fechatter 部署指南

## 🏗️ 项目架构

Fechatter 采用现代化微服务架构，包含以下组件：

### 核心服务
- **API Gateway** (8080) - Pingora 网关，统一入口和路由管理
- **fechatter_server** (6688) - 主 API 服务，处理用户、聊天、文件等功能
- **notify_server** (6687) - 实时通知服务，处理 SSE 推送
- **bot_server** (6686) - AI 机器人服务，提供智能对话和代码索引
- **analytics_server** (6690) - 数据分析服务，处理使用统计

### 基础设施
- **PostgreSQL** - 主数据库，存储用户、消息等核心数据
- **Redis** - 缓存、会话存储和限流计数器
- **NATS JetStream** - 消息队列和事件总线
- **Meilisearch** - 全文搜索引擎
- **ClickHouse** - 分析数据库，存储事件和统计数据

### 前端
- **fechatter_frontend** - Vue 3 + Vite 构建的 Web 应用

## 🚀 快速开始

### 环境准备

1. 创建 `.env` 文件：
```bash
cp env.example .env
# 编辑 .env，设置必要的环境变量
```

必要的环境变量：
- `OPENAI_API_KEY` - OpenAI API 密钥
- `JWT_SECRET` - JWT 签名密钥（生产环境必须更改）
- `REDIS_PASSWORD` - Redis 密码
- `MEILI_MASTER_KEY` - Meilisearch 主密钥

2. 安装依赖：
- Docker & Docker Compose
- Rust toolchain
- Node.js & Yarn
- tmux (可选，用于本地开发)

### 使用 Docker Compose (推荐)

```bash
# 构建并启动所有服务
make up

# 查看日志
make logs

# 查看特定服务日志
make logs-fechatter-server

# 停止服务
make down
```

### 使用独立 Docker

```bash
# 构建所有镜像
make build-docker

# 或构建特定服务
make build-docker-gateway
make build-docker-fechatter-server

# 运行所有服务
make run-docker

# 查看运行状态
make ps

# 查看资源使用
make stats

# 停止所有服务
make kill-docker
```

### 本地开发

```bash
# 设置数据库
make db-setup

# 启动基础设施服务（使用 Docker）
docker-compose up postgres redis nats meilisearch -d

# 启动所有后端服务 (使用 tmux)
make dev

# 启动前端开发服务器
make frontend-dev

# 停止开发服务
make dev-stop
```

## 📝 服务访问地址

### 生产/Docker 环境
- **前端应用**: http://localhost:8080 (通过网关)
- **API 网关**: http://localhost:8080
- **健康检查**: http://localhost:8080/health
- **指标监控**: http://localhost:8080/metrics

### 开发环境（直接访问）
- **前端开发服务器**: http://localhost:5173
- **Fechatter API**: http://localhost:6688
- **通知服务 SSE**: http://localhost:6687/sse
- **Bot API**: http://localhost:6686
- **分析 API**: http://localhost:6690

### 基础设施服务
- **PostgreSQL**: localhost:5432
- **Redis**: localhost:6379
- **NATS**: localhost:4222 (客户端), localhost:8222 (监控)
- **Meilisearch**: http://localhost:7700
- **ClickHouse**: http://localhost:8123 (HTTP), localhost:9000 (Native)

## 🔧 配置文件

所有服务配置文件位于 `fixtures/` 目录：
- `gateway.yml` - API 网关配置
- `fechatter.yml` - 主服务配置
- `notify.yml` - 通知服务配置
- `bot.yml` - AI 服务配置
- `analytics.yml` - 分析服务配置

配置文件支持环境变量替换，格式：`${VAR_NAME:-default_value}`

## 🐳 Docker 构建优化

Dockerfile 采用多阶段构建，优化了：
1. **依赖缓存** - 单独构建依赖，加快后续构建
2. **最小镜像** - 使用 debian:bookworm-slim 作为运行时基础镜像
3. **安全性** - 使用非 root 用户运行服务

## 🌐 网络架构

所有服务通过 `fechatter-net` Docker 网络通信：
- 服务间使用内部主机名（如 `postgres`, `redis`）
- 只暴露必要的端口到主机
- API Gateway 作为统一入口点

## 📊 监控和日志

- 使用 `RUST_LOG` 环境变量控制日志级别
- 可通过 `make logs` 查看实时日志
- 使用 `make stats` 监控资源使用
- Prometheus 格式指标端点：`/metrics`

## 🚨 故障排查

1. **服务启动失败**
   ```bash
   # 检查容器状态
   make ps
   # 查看具体服务日志
   make logs-fechatter-server
   # 检查健康状态
   curl http://localhost:8080/health
   ```

2. **数据库连接问题**
   ```bash
   # 确保数据库运行
   docker ps | grep postgres
   # 检查连接
   docker exec -it fechatter-postgres psql -U postgres -d fechatter
   # 重置数据库
   make db-reset
   ```

3. **Redis 连接问题**
   ```bash
   # 测试 Redis 连接
   docker exec -it fechatter-redis redis-cli -a fechatter_redis_pass ping
   ```

4. **NATS 连接问题**
   ```bash
   # 查看 NATS 状态
   curl http://localhost:8222/varz
   ```

5. **清理环境**
   ```bash
   # 停止所有容器
   make kill-docker
   # 清理构建缓存
   make clean
   # 删除数据卷（谨慎操作）
   docker volume prune
   ```

## 🔒 安全建议

1. **生产环境必须更改的配置**：
   - `JWT_SECRET` - 使用强随机密钥
   - `REDIS_PASSWORD` - 设置强密码
   - `MEILI_MASTER_KEY` - 设置强密钥
   - 数据库密码

2. **网络安全**：
   - 使用 TLS/SSL 加密通信
   - 配置防火墙规则
   - 限制服务端口访问

3. **访问控制**：
   - 启用 API 限流
   - 配置 CORS 策略
   - 实施认证授权

## 📚 相关文档

- [微服务架构详解](./MICROSERVICES_ARCHITECTURE.md)
- [API 网关配置](./fechatter_gateway/README.md)
- [迁移指南](./MIGRATION_SUMMARY.md) 