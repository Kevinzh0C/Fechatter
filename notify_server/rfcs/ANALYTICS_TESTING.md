# Notify Server Analytics 集成测试

## 🧪 **测试 Analytics 功能**

### 1. **配置测试**

```bash
# 检查配置是否正确加载
cargo test analytics_config_test --lib

# 测试禁用状态下的发布器
cargo test analytics_publisher_disabled --lib
```

### 2. **本地测试环境**

```bash
# 1. 启动 NATS 服务器
docker run -d --name nats-server -p 4222:4222 nats:latest

# 2. 启动 analytics_server
cargo run --bin analytics_server

# 3. 启动 notify_server (启用 analytics)
ANALYTICS_ENABLED=true cargo run --bin notify_server
```

### 3. **验证事件发送**

```bash
# 监听 NATS 中的 analytics 事件
nats sub "fechatter.analytics.notify.>"

# 示例输出:
# fechatter.analytics.notify.user.connected: {"context":{"client_id":"notify_server_uuid"...}}
# fechatter.analytics.notify.notification.received: {"event_type":"NewMessage"...}
```

### 4. **SSE 连接测试**

```bash
# 连接 SSE（会触发用户连接事件）
curl -N -H "Authorization: Bearer YOUR_JWT" \
  http://localhost:6687/events

# 应该在 NATS 中看到:
# fechatter.analytics.notify.user.connected
# fechatter.analytics.notify.user.disconnected (断开时)
```

### 5. **Analytics Server 验证**

```bash
# 检查 analytics_server 是否收到事件
curl http://localhost:6690/metrics

# 示例响应:
{
  "events_received_total": 15,
  "events_processed_total": 15,
  "events_failed_total": 0
}
```

## 📊 **测试用例**

### 用户连接/断开事件
```rust
#[tokio::test]
async fn test_user_connection_analytics() {
    let config = AnalyticsConfig {
        enabled: true,
        nats_url: "nats://localhost:4222".to_string(),
        ..Default::default()
    };
    
    let publisher = AnalyticsPublisher::new(config).await.unwrap();
    
    // 测试用户连接事件
    publisher.user_connected(
        UserId(123),
        "conn_123".to_string(),
        Some("Mozilla/5.0".to_string()),
    );
    
    // 测试用户断开事件
    publisher.user_disconnected(
        UserId(123),
        "conn_123".to_string(),
        300000, // 5 分钟
    );
    
    // 验证事件被发送到 NATS
}
```

### 通知送达事件
```rust
#[tokio::test]
async fn test_notification_delivery_analytics() {
    let publisher = create_test_publisher().await;
    
    publisher.notification_received(
        UserId(123),
        "NewMessage".to_string(),
        "sse".to_string(),
        true,
        Some(5), // 5ms 送达延迟
    );
}
```

## ⚡ **性能测试**

### 批量事件处理
```rust
#[tokio::test]
async fn test_batch_event_processing() {
    let publisher = create_test_publisher().await;
    
    // 发送 1000 个事件
    for i in 0..1000 {
        publisher.user_connected(
            UserId(i),
            format!("conn_{}", i),
            None,
        );
    }
    
    // 等待批量刷新
    tokio::time::sleep(Duration::from_secs(6)).await;
    
    // 验证所有事件都被处理
}
```

### 零性能影响测试
```rust
#[tokio::test]
async fn test_analytics_performance_impact() {
    let start = Instant::now();
    
    // 模拟 SSE 连接处理（不启用 analytics）
    simulate_sse_connections(1000, false).await;
    let baseline = start.elapsed();
    
    let start = Instant::now();
    
    // 模拟 SSE 连接处理（启用 analytics）
    simulate_sse_connections(1000, true).await;
    let with_analytics = start.elapsed();
    
    // analytics 开销应该 < 5%
    assert!(with_analytics.as_millis() < baseline.as_millis() * 105 / 100);
}
```

## 🔍 **故障排查**

### 常见问题检查

1. **NATS 连接失败**
```bash
# 检查 NATS 服务状态
docker ps | grep nats
nats server info
```

2. **Analytics 事件未发送**
```bash
# 检查配置
cat notify.yml | grep -A 5 analytics

# 检查日志
tail -f logs/notify_server.log | grep analytics
```

3. **批量刷新延迟**
```bash
# 调试模式查看详细日志
RUST_LOG=debug cargo run --bin notify_server
```

## ✅ **验收标准**

- [ ] **用户连接事件**: 每次 SSE 连接都发送 user_connected 事件
- [ ] **用户断开事件**: 每次 SSE 断开都发送 user_disconnected 事件  
- [ ] **通知送达事件**: 每次通知推送都记录送达性能
- [ ] **聊天成员事件**: 用户加入/离开聊天时发送相应事件
- [ ] **实时事件性能**: 实时事件处理性能被记录
- [ ] **批量处理**: 事件能够批量发送以优化性能
- [ ] **错误处理**: Analytics 失败不影响主功能
- [ ] **配置开关**: 可通过配置完全禁用 analytics
- [ ] **零阻塞**: 所有 analytics 操作都是异步的

## 📈 **监控指标**

在生产环境中，应监控以下指标：

```bash
# 事件发送速率
fechatter_analytics_events_sent_per_second

# 批量刷新频率
fechatter_analytics_batch_flushes_per_minute

# NATS 连接状态
fechatter_analytics_nats_connection_status

# 事件处理延迟
fechatter_analytics_event_processing_duration_ms
```

---

通过这些测试，确保 notify_server 的 analytics 集成功能完整且高性能！ 🎯 