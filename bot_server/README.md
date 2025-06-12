# Bot Server - AI Chat Bot Service

一个基于 NATS 的智能聊天机器人服务，支持 OpenAI 和 Ollama 模型。

## 🚀 快速开始

### 1. 配置文件

复制并编辑配置文件：
```bash
cp bot.yml.example bot.yml
```

### 2. 环境变量

```bash
export OPENAI_API_KEY="your_openai_api_key"
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/fechatter"
```

### 3. 启动服务

```bash
# 启动 bot server
cargo run --bin bot

# 或使用环境变量
OPENAI_API_KEY=your_key cargo run --bin bot
```

## 🏥 健康检查

Bot Server 提供了完整的健康检查端点：

### 健康检查端点

- **完整健康检查**: `GET http://localhost:6686/health`
- **就绪检查**: `GET http://localhost:6686/ready`  
- **存活检查**: `GET http://localhost:6686/live`

### 使用测试脚本

```bash
# 运行完整的健康检查
./scripts/test-bot.sh

# 快速检查
curl http://localhost:6686/health | jq
```

## 🔧 如何检查 Bot 能否正常工作

### 方法 1: 使用健康检查脚本（推荐）

```bash
chmod +x scripts/test-bot.sh
./scripts/test-bot.sh
```

这个脚本会检查：
- ✅ PostgreSQL 数据库连接
- ✅ NATS 消息队列连接  
- ✅ OpenAI API 连接
- ✅ Analytics 服务连接
- ✅ AI SDK 功能测试
- ✅ 配置文件验证
- ✅ 端到端消息测试

### 方法 2: 手动检查步骤

#### 1. 检查依赖服务
```bash
# PostgreSQL
psql $DATABASE_URL -c "SELECT 1;"

# NATS
nc -z localhost 4222

# Analytics Server
curl http://localhost:6690/health
```

#### 2. 检查 Bot Server 健康状态
```bash
curl http://localhost:6686/health | jq
```

#### 3. 测试 AI 功能
```bash
cd ai_sdk
cargo run --example test_bot
```

#### 4. 发送测试消息
```bash
# 使用 nats CLI 发送测试消息
echo '{
  "msg": {
    "id": 999999,
    "chat_id": 1,
    "sender_id": 1,
    "content": "Hello bot!",
    "created_at": "'$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)'"
  },
  "members": [1, 2]
}' | nats pub fechatter.messages.created --stdin
```

### 方法 3: 通过应用端到端测试

1. 在数据库中创建机器人用户：
```sql
INSERT INTO users (email, fullname, is_bot) 
VALUES ('bot@fechatter.com', 'AI Assistant', true);
```

2. 通过 Fechatter 应用向机器人发送消息
3. 检查机器人是否回复
4. 查看 analytics 数据确认事件被记录

## 📊 监控和日志

### 查看日志
```bash
# Bot server 日志
RUST_LOG=debug cargo run --bin bot

# 特定模块日志
RUST_LOG=bot_server::notif=debug cargo run --bin bot
```

### 健康检查响应示例

```json
{
  "status": "healthy",
  "service": "bot_server", 
  "version": "0.1.0",
  "checks": {
    "database": {
      "status": "healthy",
      "message": "Database connection successful",
      "latency_ms": 5
    },
    "nats": {
      "status": "healthy", 
      "message": "NATS connection successful",
      "latency_ms": 2
    },
    "openai": {
      "status": "healthy",
      "message": "OpenAI API connection successful", 
      "latency_ms": 150
    },
    "analytics": {
      "status": "healthy",
      "message": "Analytics service connection successful",
      "latency_ms": 8
    }
  },
  "timestamp": 1703123456789
}
```

## 🛠️ 故障排除

### 常见问题

#### 1. OpenAI API 错误
```bash
# 检查 API Key
echo $OPENAI_API_KEY

# 测试 API 连接
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/models
```

#### 2. 数据库连接失败
```bash
# 检查数据库连接
psql $DATABASE_URL -c "\conninfo"

# 检查 bot 用户
psql $DATABASE_URL -c "SELECT * FROM users WHERE is_bot = TRUE;"
```

#### 3. NATS 连接问题
```bash
# 检查 NATS 状态
nats server info

# 测试发布消息
echo "test" | nats pub test.topic --stdin
```

#### 4. Analytics 服务不可用
```bash
# 检查 analytics server
curl http://localhost:6690/health

# 启动 analytics server
cd analytics_server && cargo run
```

### 日志级别设置

```bash
# 详细日志
RUST_LOG=trace cargo run --bin bot

# 特定模块
RUST_LOG=bot_server=debug,swiftide=info cargo run --bin bot

# 生产环境
RUST_LOG=info cargo run --bin bot
```

### 性能调优

```yaml
# bot.yml 配置
bot:
  response_delay_ms: 1000  # 响应延迟
  max_response_length: 2000  # 最大响应长度
  
messaging:
  nats:
    url: "nats://localhost:4222"
    subscription_subjects:
    - "fechatter.messages.created"
```

## 🧪 开发和测试

### 运行测试
```bash
# 单元测试
cargo test

# AI SDK 测试
cd ai_sdk && cargo run --example test_bot

# 集成测试
./scripts/test-bot.sh
```

### 开发模式
```bash
# 监听文件变化自动重启
cargo install cargo-watch
cargo watch -x "run --bin bot"
```

## 📈 生产部署

### Docker 部署
```bash
# 构建镜像
docker build -t bot_server .

# 运行容器
docker run -p 6686:6686 \
  -e OPENAI_API_KEY=your_key \
  -e DATABASE_URL=postgres://... \
  bot_server
```

### Kubernetes 部署
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bot-server
spec:
  replicas: 2
  selector:
    matchLabels:
      app: bot-server
  template:
    metadata:
      labels:
        app: bot-server
    spec:
      containers:
      - name: bot-server
        image: bot_server:latest
        ports:
        - containerPort: 6686
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: openai-secret
              key: api-key
        livenessProbe:
          httpGet:
            path: /live
            port: 6686
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready  
            port: 6686
          initialDelaySeconds: 5
          periodSeconds: 5
```

## 📚 API 文档

### NATS 主题

- `fechatter.messages.created` - 新消息通知
- `fechatter.chats.member.joined` - 成员加入聊天
- `fechatter.analytics.bot.*` - Bot 分析事件

### 配置参数

详见 `bot.yml` 配置文件中的注释说明。

## 🤝 贡献

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 发起 Pull Request

## �� 许可证

MIT License 