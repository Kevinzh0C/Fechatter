# 🔍 NATS 事件发布调试计划

## 🎯 问题现象
- **API调用成功**: 消息发送API返回正常，消息ID递增 (8, 9, 10)
- **事件未到达**: JetStream消息计数保持在3，新消息未出现
- **影响范围**: 不影响核心消息功能，但影响实时通知

## 🧪 调试步骤

### 1. 验证NATS连接状态
```bash
# 检查NATS JetStream状态
curl -s http://localhost:8222/jsz | jq '{messages, streams, consumers}'

# 监听事件主题
nats sub "fechatter.message.created" --count=10 &
nats sub "fechatter.message.*" --count=10 &
```

### 2. 发送测试消息并观察日志
```bash
# 发送测试消息
curl -X POST http://localhost:6688/v1/chat/16/messages \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"content": "🔍 Debug NATS Event Test Message", "idempotency_key": "'$(uuidgen)'"}'
```

### 3. 检查事件发布链路

#### 3.1 AppState中的EventPublisher状态
- 查看`AppState.event_publisher_dyn()`是否为Some
- 验证EventPublisher的传输层类型

#### 3.2 DualStreamMessageService中的事件发布
关键代码位置：`fechatter_server/src/services/application/workers/message/service.rs:597-614`

```rust
// 🎯 2. MISSING LINK: Publish message created event to NATS JetStream
let event_publisher = Arc::clone(&self.event_publisher);
let jetstream_message = saved_message.clone();
let jetstream_members = chat_members.clone();
tokio::spawn(async move {
  if let Err(e) = event_publisher
    .publish_message_created(&jetstream_message, jetstream_members)
    .await
  {
    warn!("Failed to publish message created event to NATS JetStream: {}", e);
  } else {
    info!("✅ Message created event published to NATS JetStream: message_id={}", jetstream_message.id);
  }
});
```

#### 3.3 AppStateEventPublisher的实现
检查`AppStateEventPublisher.publish_message_created()`方法是否被正确调用

### 4. 可能的根本原因

#### 4.1 AppState初始化问题
```rust
// AppState中EventPublisher可能为None
if let Some(publisher) = &self.publisher {
  // 正常发布
} else {
  warn!("Event publisher not available, skipping message created event");
  // 问题：事件被跳过！
}
```

#### 4.2 NATS传输层配置问题
- EventPublisher使用InMemoryTransport而非NatsTransport
- NATS连接未正确建立

#### 4.3 事件主题配置问题
- 事件发布到错误的主题
- JetStream流配置不匹配

## 🔧 调试代码注入

在关键位置添加详细日志：

```rust
// 在 AppStateEventPublisher::publish_message_created 开头添加
info!("🔍 [DEBUG] AppStateEventPublisher::publish_message_created called");
info!("🔍 [DEBUG] Publisher available: {}", self.publisher.is_some());

if let Some(publisher) = &self.publisher {
  info!("🔍 [DEBUG] Publisher type: {:?}", std::any::type_name_of_val(publisher.as_ref()));
  
  // 在 publish_message_event 调用前后添加
  info!("🔍 [DEBUG] About to call publish_message_event with MessageLifecycle::Created");
  let result = publisher.publish_message_event(...).await;
  info!("🔍 [DEBUG] publish_message_event result: {:?}", result);
}
```

## 🎯 预期结果

成功调试后应该看到：
1. AppState中EventPublisher为Some且类型为NatsTransport
2. 事件发布日志显示成功发送到正确主题
3. JetStream消息计数正确递增
4. notify_server接收到消息事件

## 📊 验证指标

- [x] 编译错误修复 ✅
- [x] 服务正常启动 ✅
- [ ] EventPublisher状态检查
- [ ] 事件发布日志追踪
- [ ] JetStream消息计数验证
- [ ] 完整消息流测试 