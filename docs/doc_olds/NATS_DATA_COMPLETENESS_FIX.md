# NATS Event Data Completeness Fix

## 🔍 问题分析

从废弃的PostgreSQL NOTIFY触发器（0005_notify_trigger.sql）可以看出，原始设计的事件通知包含完整的数据：

### PostgreSQL NOTIFY 原始设计
```sql
-- 消息创建事件包含：
{
  "message": {
    "id": 消息ID,
    "chat_id": 聊天ID,
    "sender_id": 发送者ID,
    "content": 内容,
    "files": 文件列表,          -- ✅ 包含文件
    "created_at": 创建时间,
    "idempotency_key": 幂等键   -- ✅ 包含幂等键
  },
  "chat_members": [用户ID列表]   -- ✅ 包含聊天成员
}

-- 聊天成员变更事件包含：
{
  "op": "INSERT/UPDATE/DELETE",  -- ✅ 包含操作类型
  "old": {...},
  "new": {...}
}
```

### 当前NATS实现的问题
1. ❌ **缺少文件信息**: 消息事件不传递 `files` 字段
2. ❌ **缺少幂等性key**: 消息事件不传递 `idempotency_key`
3. ❌ **缺少聊天成员**: 消息事件不传递 `chat_members` 列表
4. ❌ **缺少操作类型**: 聊天成员变更事件没有 `TG_OP` 信息

## 🛠️ 修复实施

### 1. 更新MessageEvent结构 (application_event_publisher.rs)
```rust
/// Message domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageEvent {
  /// Message sent
  MessageSent {
    message_id: i64,
    chat_id: i64,
    sender_id: i64,
    content: String,                      // ✅ 完整内容，而不是预览
    files: Option<Vec<String>>,           // ✅ 添加文件列表
    idempotency_key: Option<uuid::Uuid>,  // ✅ 添加幂等键
    chat_members: Vec<i64>,               // ✅ 添加聊天成员列表
    mentioned_users: Vec<i64>,
  },
  // ... 其他事件类型
}
```

### 2. 更新事件发布逻辑 (message_app_service.rs)
```rust
// 获取聊天成员列表
let chat_members = self
  .domain_service
  .get_chat_members(i64::from(chat_id))
  .await
  .map_err(AppError::from)?
  .into_iter()
  .map(|member_id| UserId(member_id))
  .collect::<Vec<_>>();

// 发布包含完整数据的事件
self.publish_message_event(MessageEvent::MessageSent {
  message_id: i64::from(message.id),
  chat_id: i64::from(message.chat_id),
  sender_id: i64::from(message.sender_id),
  content: message.content.clone(),         // ✅ 完整内容
  files: message.files.clone(),            // ✅ 文件列表
  idempotency_key: message.idempotency_key, // ✅ 幂等键
  chat_members: chat_members.iter().map(|id| i64::from(*id)).collect(), // ✅ 成员列表
  mentioned_users: self.parse_mentions(&message.content),
})
.await;
```

### 3. 实现聊天成员获取 (messaging repository)
```rust
/// Get chat members
pub async fn get_chat_members(&self, chat_id: i64) -> Result<Vec<i64>, CoreError> {
  let members = sqlx::query_scalar::<_, i64>(
    "SELECT user_id FROM chat_members WHERE chat_id = $1"
  )
  .bind(chat_id)
  .fetch_all(&*self.pool)
  .await
  .map_err(|e| CoreError::Database(e.to_string()))?;

  Ok(members)
}
```

### 4. 修复NATS事件转换 (application_event_publisher.rs)
```rust
ApplicationEvent::Message(MessageEvent::MessageSent {
  message_id,
  chat_id,
  sender_id,
  content,
  files,
  idempotency_key,
  chat_members,
  ..
}) => {
  // 使用完整数据创建核心消息事件
  let message = fechatter_core::Message {
    id: fechatter_core::MessageId(*message_id),
    chat_id: fechatter_core::ChatId(*chat_id),
    sender_id: fechatter_core::UserId(*sender_id),
    content: content.clone(),
    files: files.clone(),
    created_at: chrono::Utc::now(),
    idempotency_key: *idempotency_key,
  };

  let core_event = CoreMessageEvent {
    version: fechatter_core::contracts::events::EventVersion::V1,
    kind: MessageLifecycle::Created,
    msg: message,
    members: chat_members.iter().map(|id| UserId(*id)).collect(), // ✅ 真实成员列表
    occurred_at: Utc::now(),
    sig: None,
  };
  
  // 发布到NATS
}
```

### 5. 确保事件同步发布 (chat_app_service.rs)
```rust
// 使用 publish_sync 而不是 publish_async，确保NATS事件被立即发布
if let Err(e) = self.event_publisher.publish_sync(event).await {
  warn!("Failed to publish members added event: {}", e);
}
```

## 🔄 数据流程

1. **消息创建流程**:
   ```
   Handler → MessageApplicationService → MessageDomainService → Repository
                    ↓
            获取聊天成员列表
                    ↓
            发布ApplicationEvent (包含完整数据)
                    ↓
   ApplicationEventPublisher → 转换为CoreEvent → NATS
   ```

2. **成员变更流程**:
   ```
   Handler → ChatApplicationService → ChatDomainService
                    ↓
            发布ApplicationEvent (包含操作者信息)
                    ↓
   ApplicationEventPublisher → 转换为CoreEvent → NATS
   ```

## ⚡ 性能考虑

1. **聊天成员获取优化**:
   - 考虑缓存聊天成员列表
   - 对于大型聊天室，可能需要限制成员数量或使用分页

2. **事件大小限制**:
   - NATS默认消息大小限制为1MB
   - 对于大型聊天室或包含很多文件的消息，需要考虑压缩或分片

## 🚀 后续优化建议

1. **添加操作类型到成员事件**:
   - 扩展 `ChatMemberJoinedEvent` 和 `ChatMemberLeftEvent` 以包含操作类型
   - 类似PostgreSQL的 `TG_OP`

2. **事件签名验证**:
   - 实现事件签名机制，确保事件完整性
   - 使用 `sig` 字段进行验证

3. **批量事件处理**:
   - 对于批量成员添加/删除，考虑发送单个批量事件而不是多个单独事件

4. **事件重放机制**:
   - 考虑添加事件序列号，支持事件重放和去重

## ✅ 验证清单

- [x] MessageEvent 包含完整的消息数据（content, files, idempotency_key）
- [x] MessageEvent 包含聊天成员列表
- [x] 实现了 get_chat_members 方法
- [x] ApplicationEventPublisher 正确转换所有字段到 NATS 事件
- [x] 聊天成员事件使用同步发布
- [ ] NATS 客户端在 ApplicationServiceProvider 中正确初始化
- [ ] 集成测试验证事件数据完整性

## 📝 注意事项

1. **向后兼容性**: 这些更改可能影响现有的事件消费者，需要协调更新
2. **数据隐私**: 确保只有授权的用户才能接收包含完整成员列表的事件
3. **错误处理**: 如果获取聊天成员失败，消息发送不应该失败，可以发送不含成员列表的事件