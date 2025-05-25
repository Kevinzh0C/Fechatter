# Fechatter 实现计划

## 当前架构

### 消息流程

1. **消息创建**：
   - 用户通过 `fechatter_server` 发送消息
   - 消息存储到 PostgreSQL 数据库
   - PostgreSQL 触发器自动发送 `pg_notify('message_created', payload)`

2. **实时通知**：
   - `notify_server` 监听 PostgreSQL NOTIFY 事件
   - 收到事件后通过 SSE 推送给浏览器客户端
   - 客户端通过 `/events` 端点接收实时通知

3. **当前文件结构**：
```
├── fechatter_server/     # 聊天服务器（业务逻辑）
│   ├── src/handlers/messages.rs  # 消息处理
│   └── chat.yml         # 配置文件
├── notify_server/       # 通知服务器（实时推送）
│   ├── src/notify.rs    # PostgreSQL NOTIFY 监听
│   ├── src/sse.rs       # SSE 实现
│   └── notify.yml       # 配置文件
└── migrations/          # 数据库迁移
    ├── 0005_notify_trigger.sql  # PG 触发器
    └── 0006_add_idempotency_key.sql
```

## 未来迁移计划

### 第一阶段：NATS 集成

**目标**：替代 PostgreSQL NOTIFY/LISTEN 机制

**实现步骤**：
1. 在 `fechatter_server` 中集成 NATS 发布器
2. 在消息创建后发布 NATS 事件（而非依赖 PG 触发器）
3. 修改 `notify_server` 订阅 NATS 事件
4. 保持 SSE 推送机制不变

**配置调整**：
```yaml
# fechatter_server/chat.yml
messaging:
  enabled: true  # 启用 NATS
```

```yaml
# notify_server/notify.yml  
messaging:
  enabled: true  # 启用 NATS 订阅
```

### 第二阶段：Meilisearch 集成

**目标**：提供强大的消息搜索功能

**实现步骤**：
1. 添加 Meilisearch 客户端到服务中
2. 在消息创建时同步索引到 Meilisearch
3. 实现搜索 API 端点
4. 前端集成搜索功能

**配置调整**：
```yaml
# fechatter_server/chat.yml 和 notify_server/notify.yml
search:
  enabled: true  # 启用搜索功能
```

## 代码实现要点

### 当前 PostgreSQL 通知机制

```sql
-- migrations/0005_notify_trigger.sql
CREATE OR REPLACE FUNCTION add_message_to_chat() 
RETURNS TRIGGER AS $$
BEGIN
  PERFORM pg_notify('message_created', json_build_object(
    'message', jsonb_build_object(...),
    'chat_members', USERS
  )::text);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

```rust
// notify_server/src/notify.rs
pub async fn set_up_pg_listener(state: AppState) -> Result<()> {
  let mut listener = PgListener::connect(&state.config.server.db_url).await?;
  listener.listen("message_created").await?;
  // ... 处理通知
}
```

### 未来 NATS 实现（计划）

```rust
// fechatter_server 中发布事件
async fn publish_message_event(nats_client: &NatsClient, message: &Message) {
  let event = MessageCreatedEvent {
    message: message.clone(),
    chat_members: get_chat_members(message.chat_id).await,
  };
  
  nats_client
    .publish("fechatter.messages.created", serde_json::to_vec(&event)?)
    .await?;
}
```

```rust
// notify_server 中订阅事件
async fn subscribe_to_message_events(nats_client: &NatsClient) {
  let mut subscription = nats_client
    .subscribe("fechatter.messages.created")
    .await?;
    
  while let Some(message) = subscription.next().await {
    // 处理消息并推送给客户端
  }
}
```

### 未来 Meilisearch 实现（计划）

```rust
// 消息索引
async fn index_message(meilisearch_client: &MeilisearchClient, message: &Message) {
  let index = meilisearch_client.index("fechatter_messages");
  let document = MessageDocument {
    id: message.id,
    content: message.content.clone(),
    chat_id: message.chat_id,
    sender_id: message.sender_id,
    created_at: message.created_at,
  };
  
  index.add_documents(&[document], Some("id")).await?;
}

// 搜索实现
async fn search_messages(
  meilisearch_client: &MeilisearchClient,
  query: &str,
  chat_id: Option<i64>,
) -> Result<Vec<Message>> {
  let index = meilisearch_client.index("fechatter_messages");
  let mut search = index.search();
  search.with_query(query);
  
  if let Some(chat_id) = chat_id {
    search.with_filter(&format!("chat_id = {}", chat_id));
  }
  
  let results = search.execute::<MessageDocument>().await?;
  // 转换结果为 Message
}
```

## 配置说明

### 当前配置状态

- **NATS 配置**: `enabled: false` - 预留配置，等待实现
- **Meilisearch 配置**: `enabled: false` - 预留配置，等待实现
- **PostgreSQL**: 当前主要通信机制
- **SSE**: 当前浏览器推送机制

### 迁移时的配置变更

1. **启用 NATS**: 将 `messaging.enabled` 设为 `true`
2. **启用搜索**: 将 `search.enabled` 设为 `true`
3. **逐步迁移**: 可以并行运行 PostgreSQL 和 NATS，然后逐步切换

## 测试策略

### 当前测试

- 集成测试验证 PostgreSQL NOTIFY 和 SSE 推送
- 消息创建和实时通知的端到端测试

### 未来测试

- NATS 事件发布和订阅测试
- Meilisearch 索引和搜索功能测试
- 性能和并发测试

## 部署考虑

### 当前部署

- PostgreSQL 数据库
- fechatter_server (端口 6688)
- notify_server (端口 6687)

### 未来部署

- 添加 NATS 服务器
- 添加 Meilisearch 服务器
- 考虑服务发现和配置管理

这个分阶段的实现计划确保了系统的稳定性，同时为未来的扩展做好了准备。 