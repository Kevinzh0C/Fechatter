# Services Usage Guide for Handlers

## 📋 Overview

作为全人类最厉害的Rust工程师设计的services架构使用指南。本文档详细说明Handler层如何正确调用各层services，遵循Clean Architecture的依赖关系。

## 🏗️ Architecture Layers

```
┌─────────────────────────────────────────┐
│           Handler Layer                 │ ← HTTP协调层
│  • HTTP请求/响应处理                      │
│  • 极简业务逻辑编排（≤20行）                │
└─────────────────────────────────────────┘
                    ↓ calls
┌─────────────────────────────────────────┐
│        Application Service Layer        │ ← 用例编排层
│  • Use Case协调                         │
│  • 跨领域服务协调                         │
│  • 事务边界管理                          │
│  • 缓存策略执行                          │
└─────────────────────────────────────────┘
                    ↓ calls
┌─────────────────────────────────────────┐
│       Infrastructure Service Layer      │ ← 技术实现层
│  • 数据持久化                           │
│  • 外部服务集成                          │
│  • 缓存实现                             │
│  • 消息队列                             │
└─────────────────────────────────────────┘
```

## 🎯 Handler调用原则

### ✅ 正确的调用方式

1. **Handler → Application Service** (推荐)
2. **Handler → AppState业务方法** (兼容现有代码)
3. **避免跳级调用Infrastructure Service**

### ❌ 错误的调用方式

1. ❌ Handler直接调用Infrastructure Service
2. ❌ Handler直接写SQL操作
3. ❌ Handler包含复杂业务逻辑

## 📚 Available Services

### 1. Application Services (`src/services/application/`)

#### 🔐 AuthService (认证应用服务)
```rust
// 获取方式
use crate::services::application::AuthService;

// Handler中使用
pub async fn signup_handler(
  State(state): State<AppState>,
  Json(create_user): Json<CreateUser>,
) -> Result<Json<AuthTokens>, AppError> {
  // 🎯 通过AppState获取AuthService
  let auth_service = state.auth_service(); // 返回 AuthService实例
  
  // 调用应用服务
  let auth_tokens = auth_service
    .signup(&create_user, None)
    .await?;
    
  Ok(Json(auth_tokens))
}
```

**可用方法:**
- `signup(payload, auth_context) -> AuthTokens`
- `signin(payload, auth_context) -> Option<AuthTokens>`
- `refresh_token(refresh_token, context) -> AuthTokens`
- `logout(refresh_token) -> ()`
- `logout_all(user_id) -> ()`

#### 💬 ChatService (聊天应用服务)
```rust
use crate::services::application::{ChatService, ChatServiceTrait};

pub async fn create_chat_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(create_chat): Json<CreateChat>,
) -> Result<Json<ChatDetailView>, AppError> {
  // 🎯 获取Chat应用服务
  let chat_service = state.chat_application_service()?;
  
  // 构建输入
  let input = CreateChatInput {
    name: create_chat.name,
    chat_type: create_chat.chat_type,
    description: create_chat.description,
    created_by: user.id.into(),
    workspace_id: user.workspace_id.map(Into::into),
    initial_members: create_chat.members.unwrap_or_default(),
    members: create_chat.members,
  };
  
  // 调用应用服务
  let chat_detail = chat_service.create_chat(input).await?;
  
  Ok(Json(chat_detail))
}
```

**可用方法:**
- `create_chat(input) -> ChatDetailView`
- `get_chat(id) -> Option<ChatDetailView>`
- `list_user_chats(user_id) -> Vec<ChatSidebar>`
- `add_members(chat_id, user_id, member_ids) -> ()`
- `remove_members(chat_id, user_id, member_ids) -> ()`

#### 👤 UserAppService (用户应用服务)
```rust
use crate::services::application::UserAppService;

pub async fn list_users_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Query(query): Query<ListUsersQuery>,
) -> Result<Json<Vec<User>>, AppError> {
  // 🎯 获取User应用服务
  let user_service = state.user_application_service()?;
  
  // 调用应用服务
  let users = user_service
    .list_workspace_users(user.workspace_id.into(), query.limit.unwrap_or(50))
    .await?;
    
  Ok(Json(users))
}
```

#### 🔔 NotificationService (通知应用服务)
```rust
use crate::services::application::NotificationService;

pub async fn send_notification_handler(
  State(state): State<AppState>,
  Json(notification): Json<SendNotificationRequest>,
) -> Result<Json<NotificationResponse>, AppError> {
  // 🎯 获取通知服务
  let notification_service = state.notification_service()?;
  
  // 调用应用服务
  let result = notification_service
    .send_notification(notification)
    .await?;
    
  Ok(Json(result))
}
```

### 2. Infrastructure Services (`src/services/infrastructure/`)

#### 💾 Cache Services
```rust
use crate::services::infrastructure::{Cache, RedisCacheService};

// ⚠️ 通常通过Application Service调用，Handler避免直接使用
pub async fn invalidate_cache_handler(
  State(state): State<AppState>,
  Path(user_id): Path<i64>,
) -> Result<StatusCode, AppError> {
  // 🎯 通过CacheStrategyService调用（更好）
  let cache_service = state.cache_strategy_service()?;
  cache_service.invalidate_user_caches(user_id).await;
  
  Ok(StatusCode::OK)
}
```

#### 🔍 Search Services
```rust
use crate::services::infrastructure::search::SearchService;

// ⚠️ 通常通过Application Service调用
pub async fn search_messages_handler(
  State(state): State<AppState>,
  Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResults>, AppError> {
  // 🎯 通过MessageApplicationService调用（更好）
  let search_service = state.search_service()?;
  let results = search_service.search_messages(query).await?;
  
  Ok(Json(results))
}
```

## 🔗 AppState Service Access Methods

### Authentication & Authorization
```rust
impl AppState {
  /// 获取认证服务
  pub fn auth_service(&self) -> AuthService {
    self.inner.service_provider.create_service()
  }
  
  /// 验证JWT Token
  pub fn verify_token(&self, token: &str) -> Result<UserClaims, AppError> {
    self.inner.service_provider.verify_token(token)
      .map_err(Into::into)
  }
  
  /// 确保用户是聊天成员 - 兼容现有Handler
  pub async fn ensure_user_is_chat_member(
    &self,
    chat_id: i64,
    user_id: UserId,
  ) -> Result<(), AppError> {
    // 通过Chat应用服务验证
    let chat_service = self.chat_application_service()?;
    chat_service.verify_member_access(chat_id, user_id.into()).await
  }
}
```

### Application Services Access
```rust
impl AppState {
  /// 获取聊天应用服务
  pub fn chat_application_service(&self) -> Result<Arc<dyn ChatServiceTrait>, AppError> {
    Ok(self.inner.service_provider.chat_service())
  }
  
  /// 获取用户应用服务  
  pub fn user_application_service(&self) -> Result<Arc<dyn UserServiceTrait>, AppError> {
    Ok(self.inner.service_provider.user_service())
  }
  
  /// 获取消息应用服务
  pub fn messaging_service(&self) -> Result<Arc<dyn MessageServiceTrait>, AppError> {
    Ok(self.inner.service_provider.message_service())
  }
  
  /// 获取通知应用服务
  pub fn notification_service(&self) -> Result<Arc<NotificationService>, AppError> {
    Ok(self.inner.service_provider.notification_service())
  }
}
```

### Infrastructure Services Access
```rust
impl AppState {
  /// 获取缓存策略服务
  pub fn cache_strategy_service(&self) -> Result<Arc<CacheStrategyService>, AppError> {
    Ok(self.inner.service_provider.cache_strategy_service())
  }
  
  /// 获取搜索服务
  pub fn search_service(&self) -> Option<&SearchService> {
    self.inner.service_provider.search_service()
  }
  
  /// 获取存储服务
  pub fn storage_service(&self) -> &dyn StorageService {
    self.inner.storage.as_ref()
  }
}
```

## 📖 Handler Templates

### Template 1: 标准CRUD Handler
```rust
pub async fn create_resource_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(create_request): Json<CreateResourceRequest>,
) -> Result<Json<ResourceResponse>, AppError> {
  // 1. 获取应用服务
  let resource_service = state.resource_application_service()?;
  
  // 2. 构建输入（数据转换）
  let input = CreateResourceInput {
    name: create_request.name,
    created_by: user.id.into(),
    workspace_id: user.workspace_id.into(),
    // ... 其他字段
  };
  
  // 3. 调用应用服务（业务逻辑）
  let resource = resource_service.create_resource(input).await?;
  
  // 4. 构建响应
  Ok(Json(ResourceResponse::from(resource)))
}
```

### Template 2: 列表查询Handler  
```rust
pub async fn list_resources_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Query(query): Query<ListResourcesQuery>,
) -> Result<Json<Vec<ResourceSummary>>, AppError> {
  // 1. 获取应用服务
  let resource_service = state.resource_application_service()?;
  
  // 2. 调用应用服务
  let resources = resource_service
    .list_user_resources(user.id.into(), query.limit.unwrap_or(50))
    .await?;
  
  // 3. 构建响应
  Ok(Json(resources))
}
```

### Template 3: 批量操作Handler
```rust
pub async fn batch_update_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(batch_request): Json<BatchUpdateRequest>,
) -> Result<Json<BatchUpdateResponse>, AppError> {
  // 1. 获取应用服务
  let resource_service = state.resource_application_service()?;
  
  // 2. 批量处理
  let mut results = Vec::new();
  for update_request in batch_request.updates {
    let result = resource_service
      .update_resource(update_request.id, update_request.data, user.id.into())
      .await;
    results.push(result);
  }
  
  // 3. 构建响应
  Ok(Json(BatchUpdateResponse { results }))
}
```

## ⚠️ Common Mistakes to Avoid

### ❌ 错误示例 1: Handler直接写SQL
```rust
// ❌ 错误：Handler直接操作数据库
pub async fn bad_create_chat_handler(
  State(state): State<AppState>,
  Json(create_chat): Json<CreateChat>,
) -> Result<Json<Chat>, AppError> {
  // ❌ 直接SQL操作违反分层架构
  let chat = sqlx::query_as!(
    Chat,
    "INSERT INTO chats (name, created_by) VALUES ($1, $2) RETURNING *",
    create_chat.name,
    create_chat.created_by
  )
  .fetch_one(state.pool())
  .await?;
  
  Ok(Json(chat))
}
```

### ❌ 错误示例 2: Handler包含复杂业务逻辑
```rust
// ❌ 错误：Handler包含复杂业务逻辑
pub async fn bad_send_message_handler(
  State(state): State<AppState>,
  Json(message): Json<CreateMessage>,
) -> Result<Json<Message>, AppError> {
  // ❌ 复杂的业务逻辑应该在Application Service中
  if message.content.len() > 4000 {
    return Err(AppError::Validation("Message too long".to_string()));
  }
  
  // ❌ 复杂的权限检查逻辑
  let user_roles = sqlx::query!("SELECT role FROM user_roles WHERE user_id = $1", message.sender_id)
    .fetch_all(state.pool())
    .await?;
    
  if !user_roles.iter().any(|r| r.role == "member" || r.role == "admin") {
    return Err(AppError::PermissionDenied("No permission to send message".to_string()));
  }
  
  // ❌ 这些逻辑都应该在MessageApplicationService中
  // ...
}
```

### ✅ 正确示例: 极简Handler
```rust
// ✅ 正确：极简Handler，仅做协调
pub async fn good_send_message_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(message): Json<CreateMessage>,
) -> Result<Json<Message>, AppError> {
  // 1. 获取应用服务
  let messaging_service = state.messaging_service()?;
  
  // 2. 调用应用服务（业务逻辑在这里处理）
  let created_message = messaging_service
    .send_message(chat_id.into(), user.id, message)
    .await?;
  
  // 3. 构建响应
  Ok(Json(created_message))
}
```

## 🚀 Best Practices

### 1. Handler职责边界
- ✅ **HTTP协调**: 请求解析、响应构建
- ✅ **简单验证**: 基本的数据格式检查  
- ✅ **服务调用**: 调用Application Service
- ❌ **业务逻辑**: 复杂的业务规则处理
- ❌ **数据访问**: 直接的SQL操作
- ❌ **外部集成**: 直接调用第三方服务

### 2. 错误处理模式
```rust
pub async fn handler_with_proper_error_handling(
  State(state): State<AppState>,
  /* ... */
) -> Result<Json<Response>, AppError> {
  // 获取服务时的错误处理
  let service = state.application_service()
    .map_err(|e| AppError::ServiceUnavailable(e.to_string()))?;
  
  // 业务逻辑调用时的错误处理
  let result = service
    .business_operation(input)
    .await
    .map_err(|e| match e {
      BusinessError::NotFound(id) => AppError::NotFound(vec![format!("Resource {}", id)]),
      BusinessError::PermissionDenied(msg) => AppError::PermissionDenied(msg),
      _ => AppError::InternalServerError(e.to_string()),
    })?;
  
  Ok(Json(result))
}
```

### 3. 异步处理模式
```rust
pub async fn handler_with_async_processing(
  State(state): State<AppState>,
  Json(request): Json<AsyncProcessingRequest>,
) -> Result<Json<ProcessingResponse>, AppError> {
  // 获取服务
  let processing_service = state.processing_service()?;
  let event_publisher = state.event_publisher()?;
  
  // 同步响应
  let task_id = processing_service
    .start_async_task(request)
    .await?;
  
  // 异步事件发布
  event_publisher
    .publish(TaskStartedEvent { task_id })
    .await
    .map_err(|e| tracing::warn!("Failed to publish event: {}", e))
    .ok();
  
  Ok(Json(ProcessingResponse { task_id }))
}
```

## 📄 Migration Guide

### 从现有Handler迁移到新架构

1. **识别Handler中的业务逻辑**
2. **将业务逻辑迁移到Application Service**  
3. **更新Handler调用Application Service**
4. **移除Handler中的直接SQL操作**

### 示例迁移
```rust
// 迁移前
pub async fn old_handler(/* ... */) -> Result<Json<Response>, AppError> {
  // 大量业务逻辑和SQL操作
  // 150+ 行代码
}

// 迁移后
pub async fn new_handler(
  State(state): State<AppState>,
  /* 其他参数 */
) -> Result<Json<Response>, AppError> {
  // ≤20行，仅做协调
  let service = state.application_service()?;
  let result = service.business_operation(input).await?;
  Ok(Json(result))
}
```

## 🔍 Debugging Tips

### 1. Service层调试
```rust
// 在Handler中添加调试日志
tracing::info!("Calling application service: {}", service_name);
let result = service.operation(input).await;
tracing::info!("Service call completed: {:?}", result);
```

### 2. 依赖关系验证
```rust
// 验证服务是否正确初始化
pub async fn health_check_handler(
  State(state): State<AppState>,
) -> Result<Json<HealthStatus>, AppError> {
  let mut status = HealthStatus::new();
  
  // 检查各个服务的健康状态
  if let Ok(auth_service) = state.auth_service() {
    status.add_service("auth", "healthy");
  } else {
    status.add_service("auth", "unhealthy");
  }
  
  Ok(Json(status))
}
```

---

## 总结

这个架构设计确保了：
- 🎯 **职责分离**: Handler极简，Application Service处理业务逻辑
- 🔗 **依赖正确**: Handler → Application → Infrastructure
- 🛡️ **错误边界**: 每层有明确的错误处理责任
- 🚀 **可维护性**: 代码结构清晰，易于测试和扩展

遵循这个指南，你的Handler将保持极简且高效！🎉 