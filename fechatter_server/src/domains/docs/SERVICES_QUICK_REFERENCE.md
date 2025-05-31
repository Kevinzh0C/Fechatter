# Services Quick Reference

## 🚀 Handler → Service 快速调用指南

### 🔐 Authentication Services

```rust
// 用户注册
let auth_service = state.auth_service();
let tokens = auth_service.signup(&create_user, None).await?;

// 用户登录  
let tokens = auth_service.signin(&signin_user, None).await?;

// Token刷新
let new_tokens = auth_service.refresh_token(&refresh_token, None).await?;

// 用户登出
auth_service.logout(&refresh_token).await?;

// 全部登出
auth_service.logout_all(user_id).await?;
```

### 💬 Chat Services

```rust
// 获取聊天应用服务
let chat_service = state.chat_application_service()?;

// 创建聊天
let input = CreateChatInput { /* ... */ };
let chat_detail = chat_service.create_chat(input).await?;

// 获取聊天详情
let chat = chat_service.get_chat(chat_id).await?;

// 列出用户聊天
let chats = chat_service.list_user_chats(user_id).await?;

// 添加成员
chat_service.add_members(chat_id, user_id, member_ids).await?;

// 移除成员  
chat_service.remove_members(chat_id, user_id, member_ids).await?;
```

### 👤 User Services

```rust
// 获取用户应用服务
let user_service = state.user_application_service()?;

// 列出工作空间用户
let users = user_service.list_workspace_users(workspace_id, limit).await?;

// 获取用户详情
let user = user_service.get_user(user_id).await?;

// 更新用户信息
user_service.update_user(user_id, update_data).await?;
```

### 📧 Messaging Services

```rust
// 获取消息服务
let messaging_service = state.messaging_service()?;

// 发送消息
let message = messaging_service.send_message(chat_id, user_id, create_message).await?;

// 列出消息
let messages = messaging_service.list_messages(chat_id, query).await?;

// 编辑消息
messaging_service.edit_message(message_id, user_id, new_content).await?;

// 删除消息
messaging_service.delete_message(message_id, user_id).await?;
```

### 🔔 Notification Services

```rust
// 获取通知服务
let notification_service = state.notification_service()?;

// 发送通知
let result = notification_service.send_notification(notification_request).await?;

// 获取通知设置
let settings = notification_service.get_settings(user_id).await?;

// 更新通知设置
notification_service.update_settings(user_id, new_settings).await?;
```

## 🛠️ Infrastructure Services

### 💾 Cache Services

```rust
// 获取缓存策略服务
let cache_service = state.cache_strategy_service()?;

// 缓存失效
cache_service.invalidate_user_caches(user_id).await;
cache_service.invalidate_chat_caches(chat_id, &affected_user_ids).await;
cache_service.invalidate_message_caches(chat_id, &affected_user_ids).await;

// 直接缓存操作
let cached_data: Option<T> = cache_service.get("cache_key").await?;
cache_service.set("cache_key", &data, ttl).await?;
cache_service.delete("cache_key").await?;
```

### 🔍 Search Services

```rust
// 获取搜索服务
if let Some(search_service) = state.search_service() {
  let results = search_service.search_messages(query).await?;
}
```

### 💽 Storage Services

```rust
// 获取存储服务
let storage_service = state.storage_service();

// 上传文件
let file_info = storage_service.upload_file(file_data, path).await?;

// 下载文件
let file_data = storage_service.download_file(file_path).await?;

// 删除文件
storage_service.delete_file(file_path).await?;
```

## 🎯 Handler模板

### 标准Handler模板
```rust
pub async fn operation_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  /* request params */
) -> Result<Json<Response>, AppError> {
  // 1. 获取服务
  let service = state.service_name()?;
  
  // 2. 调用服务
  let result = service.operation(input).await?;
  
  // 3. 返回响应
  Ok(Json(result))
}
```

### 带验证的Handler模板
```rust
pub async fn operation_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(resource_id): Path<i64>,
  Json(request): Json<Request>,
) -> Result<Json<Response>, AppError> {
  // 1. 获取服务
  let service = state.service_name()?;
  
  // 2. 权限验证（通过服务）
  service.verify_access(resource_id, user.id).await?;
  
  // 3. 执行操作
  let result = service.operation(resource_id, request).await?;
  
  // 4. 返回响应
  Ok(Json(result))
}
```

## ⚡ 常用错误处理

```rust
// Service获取错误处理
let service = state.service_name()
  .map_err(|e| AppError::ServiceUnavailable(e.to_string()))?;

// 业务逻辑错误转换
let result = service.operation(input).await
  .map_err(|e| match e {
    ServiceError::NotFound(id) => AppError::NotFound(vec![id.to_string()]),
    ServiceError::PermissionDenied(msg) => AppError::PermissionDenied(msg),
    ServiceError::ValidationFailed(errors) => AppError::Validation(errors.join(", ")),
    _ => AppError::InternalServerError(e.to_string()),
  })?;
```

## 🔄 Service可用性检查

```rust
// 检查服务是否可用
pub async fn health_check_handler(
  State(state): State<AppState>,
) -> Result<Json<HealthStatus>, AppError> {
  let mut status = HealthStatus::new();
  
  // 检查各服务
  status.add("auth", state.auth_service().is_ok());
  status.add("chat", state.chat_application_service().is_ok());
  status.add("user", state.user_application_service().is_ok());
  status.add("messaging", state.messaging_service().is_ok());
  status.add("notification", state.notification_service().is_ok());
  
  Ok(Json(status))
}
``` 