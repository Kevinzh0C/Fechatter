# Fechatter Services Fix Guide

## 🔧 修复的问题

### 1. fechatter-server 配置问题
**问题**: YAML配置文件解析错误 - `features: missing field 'message_service'`

**修复**:
- ✅ 在 `fechatter_server/chat.yml` 的 `features` 部分添加了 `message_service` 配置
- ✅ 添加了 `rate_limiting` 配置字段
- ✅ 配置现在完全兼容代码期望的结构

### 2. notify-server NATS 连接问题
**问题**: 
- `missed idle heartbeat` - JetStream 消费者心跳超时
- `no responders` - NATS 服务器无响应者
- `consumer subject filters cannot overlap` - 消费者主题过滤器重叠

**修复**:
- ✅ 调整了 JetStream 消费者配置，增加了心跳间隔
- ✅ 添加了 NATS 连接重试逻辑和连接选项
- ✅ 增强了错误处理和自动恢复机制
- ✅ 优化了消息处理循环的稳定性
- ✅ 修复了重叠的主题过滤器，确保正确的主题匹配

### 3. notify-server 和 fechatter-server 消息格式不兼容问题
**问题**:
- 两个服务之间的消息格式不匹配，导致无法正确解析

**修复**:
- ✅ 增强了 notify-server 的消息解析能力，支持多种格式
- ✅ 添加了通用JSON解析回退机制
- ✅ 确保订阅主题和发布主题匹配
- ✅ 修复了连接参数以增强稳定性

### 4. NATS 服务顺序启动问题
**问题**:
- 服务启动顺序不正确导致 JetStream 流不存在
- JetStream 消费者配置在两个服务间不一致

**修复**:
- ✅ 创建了自动化启动脚本，确保正确的启动顺序
- ✅ 确保 NATS 服务器先启动并开启 JetStream
- ✅ 确保 fechatter-server 先启动（它负责创建 JetStream 流）
- ✅ 实施了服务健康检查和状态监控

## 📋 系统要求

### 必需服务
- **PostgreSQL** (端口 5432)
- **Redis** (端口 6379)  
- **NATS Server** (端口 4222)

### 可选服务
- **MeiliSearch** (端口 7700) - 用于搜索功能

## 🚀 快速启动

### 方法一: 使用启动脚本 (推荐)

```bash
# 检查依赖
./start_services.sh --check-only

# 启动所有服务
./start_services.sh
```

启动脚本会自动：
- 检查所有必需的依赖服务
- 自动启动 NATS (如果未运行)
- 编译和启动 fechatter-server 和 notify-server
- 提供完整的状态监控

### 方法二: 手动启动

1. **启动依赖服务**:
```bash
# PostgreSQL
brew services start postgresql

# Redis  
brew services start redis

# NATS
nats-server --jetstream -p 4222 &

# MeiliSearch (可选)
meilisearch &
```

2. **启动 fechatter-server**:
```bash
cd fechatter_server
cargo run --bin fechatter_server
```

3. **启动 notify-server**:
```bash
cd notify_server  
cargo run --bin notify_server
```

## 🔍 验证服务状态

### 检查服务是否正常运行
```bash
# fechatter-server (应该显示健康状态)
curl http://localhost:6688/health

# notify-server (应该显示健康状态)  
curl http://localhost:6687/health

# PostgreSQL
pg_isready -h localhost -p 5432

# Redis
redis-cli ping

# NATS
nats sub ping --count=1 --timeout=5s

# MeiliSearch
curl http://localhost:7700/health
```

## 📊 服务端口

| 服务 | 端口 | 描述 |
|------|------|------|
| fechatter-server | 6688 | 主要API服务器 |
| notify-server | 6687 | 通知和实时消息服务 |
| PostgreSQL | 5432 | 数据库 |
| Redis | 6379 | 缓存服务 |
| NATS | 4222 | 消息队列 |
| NATS Monitor | 8222 | NATS 监控面板 |
| MeiliSearch | 7700 | 搜索引擎 |

## 🛠️ 故障排除

### fechatter-server 无法启动
1. 检查配置文件: `fechatter_server/chat.yml`
2. 确保数据库连接正常
3. 检查端口 6688 是否被占用

### notify-server NATS 连接问题
1. 确保 NATS 服务器运行在端口 4222
2. 检查 `notify_server/notify.yml` 中的 NATS URL
3. 查看日志中的具体错误信息
4. 确保 JetStream 已启用 (`nats-server --jetstream`)
5. 检查消费者配置和心跳间隔

### NATS 消息问题
1. 使用命令行工具测试消息发布和订阅:
   ```bash
   # 订阅测试
   nats sub 'fechatter.realtime.>'
   
   # 发布测试
   nats pub fechatter.realtime.chat.123 '{"type":"MessageReceived","message":{"id":"123","chat_id":123,"sender_id":1,"content":"test","files":[],"timestamp":1234567890},"chat_id":123,"recipients":[1,2,3]}'
   ```

2. 检查两个服务间的消息格式是否匹配:
   ```bash
   # 可以通过配置 RUST_LOG=debug 环境变量查看详细日志
   RUST_LOG=debug cargo run --bin notify_server
   ```

### 数据库连接问题
```bash
# 检查 PostgreSQL 状态
brew services list | grep postgresql

# 重启 PostgreSQL
brew services restart postgresql

# 检查数据库是否存在
psql -h localhost -p 5432 -U postgres -l
```

### Redis 连接问题
```bash
# 检查 Redis 状态
brew services list | grep redis

# 重启 Redis
brew services restart redis

# 测试连接
redis-cli ping
```

## 📝 配置文件位置

- **fechatter-server**: `fechatter_server/chat.yml`
- **notify-server**: `notify_server/notify.yml`

## 🔧 开发模式配置

如果需要调试或开发，可以在配置文件中启用详细日志：

### fechatter_server/chat.yml
```yaml
features:
  message_service:
    enable_detailed_tracing: true
  observability:
    log_level: "debug"
    tracing_enabled: true
```

### notify_server/notify.yml  
```yaml
# 在终端中设置日志级别
RUST_LOG=debug cargo run --bin notify_server
```

## ✅ 修复验证

运行以下命令验证所有修复是否生效：

```bash
# 1. 检查配置解析
cd fechatter_server && cargo check
cd ../notify_server && cargo check

# 2. 启动服务并查看日志
./start_services.sh

# 3. 在另一个终端测试消息流
nats pub fechatter.realtime.chat.123 '{"type":"MessageReceived","message":{"id":"test1","chat_id":123,"sender_id":1,"content":"测试消息","files":[],"timestamp":1687654321},"chat_id":123,"recipients":[1,2,3]}'
```

如果看到类似以下输出，说明修复成功：
- ✅ 配置加载成功
- ✅ NATS 连接成功 
- ✅ 消息正确传递
- ✅ 没有错误日志

## 🆘 获得帮助

如果遇到其他问题:

1. 查看服务日志中的详细错误信息
2. 确认所有依赖服务都正常运行
3. 检查防火墙和端口占用情况
4. 验证配置文件语法正确性

---

**🎉 现在 fechatter-server 和 notify-server 应该可以正常运行了！** 