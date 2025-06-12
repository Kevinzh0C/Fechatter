# Fechatter Analytics Server

Fechatter 的分析服务，用于收集和分析用户行为数据，帮助改进产品体验。

## 功能特性

- 实时事件收集和处理
- 基于 ClickHouse 的高性能数据存储
- 会话跟踪和分析
- 用户行为分析
- 错误跟踪和监控
- 性能指标收集

## 支持的事件类型

### 应用生命周期
- `AppStartEvent` - 应用启动
- `AppExitEvent` - 应用退出

### 用户认证
- `UserLoginEvent` - 用户登录
- `UserLogoutEvent` - 用户登出
- `UserRegisterEvent` - 用户注册

### 聊天相关
- `ChatCreatedEvent` - 创建聊天
- `MessageSentEvent` - 发送消息
- `ChatJoinedEvent` - 加入聊天
- `ChatLeftEvent` - 离开聊天

### 文件操作
- `FileUploadedEvent` - 文件上传
- `FileDownloadedEvent` - 文件下载

### 用户交互
- `NavigationEvent` - 页面导航
- `SearchPerformedEvent` - 搜索操作
- `NotificationReceivedEvent` - 通知接收

### 错误跟踪
- `ErrorOccurredEvent` - 错误发生

## 快速开始

### 1. 安装 ClickHouse

```bash
# macOS
brew install clickhouse

# 启动 ClickHouse
clickhouse-server
```

### 2. 创建数据库和表

```bash
# 执行 SQL 脚本
clickhouse-client < ../protos/clickhouse.sql
```

### 3. 配置服务

编辑 `analytics.yml`:

```yaml
server:
  port: 6690
  db_url: http://localhost:8123
  db_user: default
  db_password: ~
  db_name: fechatter_analytics
  base_dir: /tmp/fechatter_analytics
```

### 4. 启动服务

```bash
cargo run --bin analytics-server
```

## 集成指南

### 前端集成

在 Fechatter 前端中集成分析 SDK：

```typescript
import { AnalyticsClient } from '@fechatter/analytics';

// 初始化客户端
const analytics = new AnalyticsClient({
  endpoint: 'http://localhost:6690',
  clientId: generateClientId(),
  appVersion: '1.0.0'
});

// 发送事件
analytics.track('user_login', {
  email: 'user@example.com',
  loginMethod: 'password'
});

analytics.track('message_sent', {
  chatId: 'chat_123',
  type: 'text',
  size: 256,
  hasMentions: true
});
```

### 后端集成

在 Fechatter 服务器中发送分析事件：

```rust
use analytics_server::pb::*;
use prost::Message;

// 创建事件
let event = AnalyticsEvent {
    context: Some(EventContext {
        client_id: client_id.clone(),
        user_id: user.id.to_string(),
        // ... 其他字段
    }),
    event_type: Some(analytics_event::EventType::MessageSent(MessageSentEvent {
        chat_id: chat_id.to_string(),
        r#type: "text".to_string(),
        size: content.len() as i32,
        // ... 其他字段
    })),
};

// 发送到分析服务
let client = reqwest::Client::new();
let data = Message::encode_to_vec(&event);
client
    .post("http://analytics-server:6690/api/event")
    .header("content-type", "application/protobuf")
    .body(data)
    .send()
    .await?;
```

## 数据查询

### 查看活跃用户

```sql
SELECT 
    date,
    count(DISTINCT user_id) as daily_active_users
FROM daily_active_users
WHERE date >= today() - 7
GROUP BY date
ORDER BY date;
```

### 查看消息统计

```sql
SELECT 
    date,
    message_type,
    sum(message_count) as total_messages,
    sum(total_size) as total_size_bytes
FROM message_statistics
WHERE date >= today() - 30
GROUP BY date, message_type
ORDER BY date, message_type;
```

### 查看错误趋势

```sql
SELECT 
    toDate(timestamp) as date,
    error_type,
    count() as error_count
FROM error_tracking
WHERE date >= today() - 7
GROUP BY date, error_type
ORDER BY date, error_count DESC;
```

## 开发

### 运行测试

```bash
# 运行单元测试
cargo test

# 运行示例
cargo run --example fechatter_sender
cargo run --example db
```

### 生成 Protobuf 代码

```bash
cargo build
```

## 性能优化

1. **批量发送**: 客户端应该批量发送事件以减少网络开销
2. **异步处理**: 使用异步方式发送事件，避免阻塞主线程
3. **采样**: 对于高频事件，可以使用采样策略
4. **压缩**: 启用 gzip 压缩减少传输数据量

## 隐私和合规

- 不收集敏感个人信息
- 支持用户选择退出数据收集
- 数据自动过期（默认 90 天）
- 符合 GDPR 要求

## 监控和告警

建议配置以下监控指标：

- 事件接收速率
- 数据库写入延迟
- 错误率
- 磁盘使用率

## 故障排查

### 常见问题

1. **连接 ClickHouse 失败**
   - 检查 ClickHouse 是否运行
   - 验证连接配置

2. **事件丢失**
   - 检查客户端网络连接
   - 查看服务器日志

3. **查询性能慢**
   - 检查索引是否正确
   - 考虑增加物化视图

## 贡献

欢迎提交 Issue 和 Pull Request！ 