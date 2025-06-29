# 🔍 NATS 调试测试方案

## 🚨 当前问题
- fechatter_server认证中间件问题："No request context"
- 无法通过API发送消息测试事件发布
- 需要直接验证NATS事件发布机制

## 🧪 方案一：直接NATS测试

### 1. 监听NATS主题
```bash
# 监听所有fechatter事件
nats sub "fechatter.>" --count=20 &

# 监听具体主题
nats sub "fechatter.message.created" --count=10 &
nats sub "fechatter.message.updated" --count=10 &
nats sub "fechatter.message.deleted" --count=10 &
```

### 2. 手动发布测试消息到NATS
```bash
# 测试基本NATS连接
nats pub fechatter.test.message "Hello NATS"

# 模拟消息创建事件
nats pub fechatter.message.created '{
  "version": "V1",
  "kind": "Created", 
  "msg": {
    "id": 999,
    "chat_id": 16,
    "sender_id": 21,
    "content": "Test NATS Event",
    "files": [],
    "created_at": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
    "updated_at": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"
  },
  "members": [7, 21],
  "occurred_at": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
  "sig": null
}'
```

### 3. 检查JetStream状态变化
```bash
# 发布前
curl -s http://localhost:8222/jsz | jq '{messages, streams, consumers}'

# 发布后
curl -s http://localhost:8222/jsz | jq '{messages, streams, consumers}'
```

## 🧪 方案二：数据库直接测试

### 1. 直接在数据库中创建消息
```sql
-- 插入测试消息
INSERT INTO messages (chat_id, sender_id, content, files, created_at, updated_at, idempotency_key)
VALUES (16, 21, '🔍 Direct DB Test Message', '[]', NOW(), NOW(), gen_random_uuid())
RETURNING id, chat_id, sender_id, content, created_at;
```

### 2. 检查是否触发事件发布
如果有数据库触发器或者其他机制会自动触发事件发布。

## 🧪 方案三：简化API测试

### 1. 绕过认证的健康检查
```bash
curl http://localhost:6688/health
```

### 2. 尝试不同的API端点
```bash
# 尝试获取聊天列表（可能不需要认证）
curl http://localhost:6688/v1/chats

# 尝试获取消息列表
curl http://localhost:6688/v1/chat/16/messages
```

## 🔧 方案四：代码级调试

### 1. 添加更多调试日志到AppState初始化
在 `fechatter_server/src/state.rs` 中添加EventPublisher初始化日志

### 2. 临时绕过认证中间件
注释掉部分认证中间件来测试消息发送

### 3. 创建独立的NATS测试程序
```rust
// simple_nats_test.rs
use async_nats;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    
    let test_message = r#"{
        "test": "NATS connection working",
        "timestamp": "2025-06-05T06:12:00Z"
    }"#;
    
    client.publish("fechatter.test.debug", test_message.into()).await?;
    println!("✅ Test message published to NATS");
    
    Ok(())
}
```

## 🎯 预期结果分析

### 如果NATS基本连接工作：
- 手动发布的消息应该出现在JetStream中
- 消息计数应该增加

### 如果EventPublisher代码有问题：
- 手动发布工作，但应用内发布不工作
- 需要检查EventPublisher的NATS传输层实现

### 如果JetStream配置问题：
- 消息发布成功但不持久化
- 需要检查流配置和主题匹配

## 🚀 立即执行计划

1. **启动NATS监听** ⏰ 立即
2. **手动发布测试** ⏰ 立即  
3. **检查JetStream变化** ⏰ 立即
4. **根据结果调整策略** ⏰ 后续 