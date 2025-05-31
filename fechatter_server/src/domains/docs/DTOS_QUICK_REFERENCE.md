# DTOs Quick Reference

## 🚀 Handler → DTOs 快速调用指南

### 📥 Request DTOs Pattern

```rust
// 标准请求处理模式
pub async fn handler(
  State(state): State<AppState>,
  Json(mut request): Json<RequestDto>,
) -> Result<Json<ResponseDto>, AppError> {
  // 1. 预处理 + 验证
  request.preprocess()?;
  request.validate()?;
  
  // 2. 转换为领域模型
  let domain_model = request.to_domain()?;
  
  // 3. 调用服务
  let result = service.operation(domain_model).await?;
  
  // 4. 转换为响应DTO
  let response = ResponseDto::from_domain(&result)?;
  
  Ok(Json(response))
}
```

## 🔐 Authentication DTOs

### 登录
```rust
use crate::dtos::models::requests::auth::LoginRequest;
use crate::dtos::models::responses::auth::AuthResponse;

// 请求
let login_request = LoginRequest {
  email: "user@example.com".to_string(),
  password: "password123".to_string(),
  device_type: Some("web".to_string()),
};

// 验证和转换
login_request.validate()?;
let signin_user = login_request.to_domain()?;

// 响应
let response = AuthResponse::from_domain(&auth_tokens)?;
```

### 注册
```rust
use crate::dtos::models::requests::auth::RegisterRequest;

let register_request = RegisterRequest {
  email: "new@example.com".to_string(),
  password: "password123".to_string(),
  fullname: "John Doe".to_string(),
  workspace_name: Some("My Company".to_string()),
};

register_request.validate()?;
let create_user = register_request.to_domain()?;
```

## 💬 Chat DTOs

### 创建聊天
```rust
use crate::dtos::models::requests::chat::CreateChatRequest;

let create_chat = CreateChatRequest {
  name: "Team Chat".to_string(),
  chat_type: ChatType::Group,
  description: Some("Our team discussion".to_string()),
  members: Some(vec![1, 2, 3]),
  workspace_id: Some(1),
};

create_chat.validate()?;
let chat_input = create_chat.to_domain()?;
```

### 聊天响应
```rust
use crate::dtos::models::responses::chat::ChatResponse;

// 单个聊天
let chat_response = ChatResponse::from_domain(&chat)?;

// 聊天列表
let chat_responses = ChatResponse::from_domain_collection(&chats)?;
```

## 📧 Message DTOs

### 发送消息
```rust
use crate::dtos::models::requests::message::SendMessageRequest;

let mut message_request = SendMessageRequest {
  content: "Hello everyone!".to_string(),
  reply_to: None,
  mentions: Some(vec![1, 2]),
  files: None,
  chat_id: Some(chat_id.into()),
  sender_id: Some(user.id.into()),
};

message_request.validate()?;
let create_message = message_request.to_domain()?;
```

### 消息响应
```rust
use crate::dtos::models::responses::message::{
  MessageResponse, MessageCreateResponse, MessageListResponse
};

// 单条消息响应
let message_response = MessageResponse::from_domain(&message)?;

// 创建消息响应
let create_response = MessageCreateResponse {
  success: true,
  message: "Message sent successfully".to_string(),
  data: message_response,
};

// 消息列表
let message_list: MessageListResponse = MessageResponse::from_domain_collection(&messages)?;
```

## 👤 User DTOs

### 用户信息
```rust
use crate::dtos::models::responses::user::UserResponse;

// 单个用户
let user_response = UserResponse::from_domain(&user)?;

// 用户列表
let user_responses = UserResponse::from_domain_collection(&users)?;

// 应用过滤器
let mut response = UserResponse::from_domain(&user)?;
response.apply_filters(&filters)?;
```

## 📤 Standard Response Patterns

### 成功响应
```rust
use crate::dtos::models::responses::common::ApiResponse;

// 单项响应
let response = ApiResponse::success(
  data,
  "Operation completed successfully".to_string(),
);

// 列表响应
let response = ApiResponse::success_with_meta(
  items,
  format!("Retrieved {} items", items.len()),
  Some(metadata),
);
```

### 错误响应
```rust
let error_response = ApiResponse::error(
  "RESOURCE_NOT_FOUND".to_string(),
  "The requested resource does not exist".to_string(),
  Some(serde_json::json!({ "resource_id": 123 })),
);
```

### 分页响应
```rust
use crate::dtos::core::pagination::{PaginatedResponse, PaginationQuery};

// 分页查询
let pagination = PaginationQuery {
  page: 1,
  limit: 20,
  sort_by: Some("created_at".to_string()),
  sort_order: Some("desc".to_string()),
};

pagination.validate()?;

// 分页响应
let response = PaginatedResponse::new(items, page, limit, total);
```

## 🔧 Validation Patterns

### 自动验证
```rust
use validator::Validate;

// 使用validator crate
request.validate()
  .map_err(|e| AppError::Validation(format!("Validation failed: {:?}", e)))?;
```

### 自定义验证
```rust
use crate::dtos::core::validation::DtoValidationError;

impl RequestDto for CustomRequest {
  fn to_domain(&self) -> Result<Self::DomainModel, ConversionError> {
    // 自定义验证逻辑
    if self.field.trim().is_empty() {
      return Err(ConversionError::missing_field(
        "field", "CustomRequest", "DomainModel"
      ));
    }
    
    // 业务规则验证
    if self.value < 0 || self.value > 100 {
      return Err(ConversionError::value_out_of_range(
        "value", &self.value.to_string(), Some("0"), Some("100")
      ));
    }
    
    Ok(DomainModel { /* ... */ })
  }
}
```

## 🔄 Mappers Usage

### 简单映射
```rust
use crate::dtos::mappers::UserMapper;

// 单项映射
let response = UserMapper::to_response(&user)?;

// 批量映射
let responses = UserMapper::to_response_list(&users)?;
```

### 复杂映射
```rust
use crate::dtos::mappers::ChatMapper;

// 组合多个数据源
let chat_detail = ChatMapper::to_detail_response(
  &chat,
  &members,
  &recent_messages,
)?;
```

## ⚡ Error Handling Patterns

### 转换错误
```rust
use crate::dtos::core::conversion::{ConversionError, ConversionErrorType};

let domain_model = request.to_domain()
  .map_err(|e| match e.error_type {
    ConversionErrorType::MissingField => AppError::Validation(
      format!("Missing field: {}", e.failed_field.unwrap_or_default())
    ),
    ConversionErrorType::TypeMismatch => AppError::Validation(e.message),
    ConversionErrorType::ValueOutOfRange => AppError::Validation(e.message),
    _ => AppError::InternalServerError(e.message),
  })?;
```

### 批量处理错误
```rust
use crate::dtos::core::conversion::{BatchConverter, BatchErrorStrategy};

let converter = BatchConverter::new(
  Box::new(ItemConverter),
  BatchErrorStrategy::CollectErrors,
);

let result = converter.convert_batch(requests, &context);

// 处理结果
let successful: Vec<_> = result.successful.into_iter().map(|i| i.item).collect();
let failed: Vec<_> = result.failed.into_iter().map(|e| e.error.message).collect();
```

## 🎯 Handler Templates

### CRUD Template
```rust
pub async fn crud_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(request): Json<CreateRequest>,
) -> Result<Json<ApiResponse<ResourceResponse>>, AppError> {
  // 验证
  request.validate()?;
  
  // 转换
  let domain_input = request.to_domain()?;
  
  // 服务调用
  let resource = state.service()?.create(domain_input).await?;
  
  // 响应
  let response = ResourceResponse::from_domain(&resource)?;
  Ok(Json(ApiResponse::success(response, "Created".to_string())))
}
```

### 查询Template
```rust
pub async fn query_handler(
  State(state): State<AppState>,
  Query(pagination): Query<PaginationQuery>,
  Query(filters): Query<FilterQuery>,
) -> Result<Json<PaginatedResponse<ItemResponse>>, AppError> {
  // 验证
  pagination.validate()?;
  filters.validate()?;
  
  // 查询
  let (items, total) = state.service()?
    .list_paginated(pagination, filters.to_domain()?)
    .await?;
  
  // 响应
  let responses = ItemResponse::from_domain_collection(&items)?;
  Ok(Json(PaginatedResponse::new(responses, pagination.page, pagination.limit, total)))
}
```

### 文件上传Template
```rust
pub async fn upload_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, AppError> {
  let mut files = Vec::new();
  
  while let Some(field) = multipart.next_field().await? {
    let upload_request = FileUploadRequest {
      filename: field.file_name().unwrap_or_default().to_string(),
      data: field.bytes().await?.to_vec(),
      user_id: user.id,
    };
    
    upload_request.validate()?;
    files.push(upload_request.to_domain()?);
  }
  
  let results = state.storage_service().upload_batch(files).await?;
  let responses = FileResponse::from_domain_collection(&results)?;
  
  Ok(Json(FileUploadResponse { files: responses }))
}
```

## 🔍 Debugging Shortcuts

### 请求调试
```rust
use tracing::{info, debug};

// 记录请求
debug!("Request received: {:?}", request);
info!("Request metadata: {:?}", request.metadata());

// 验证状态
if let Err(e) = request.validate() {
  warn!("Validation failed: {:?}", e);
}
```

### 转换调试
```rust
// 转换跟踪
let domain_model = request.to_domain()
  .map_err(|e| {
    error!("Conversion failed: {:?}", e);
    error!("Context: {:?}", e.context);
    AppError::InternalServerError(e.to_string())
  })?;

info!("Conversion successful: {:?}", domain_model);
```

## 💡 Tips & Tricks

### 1. 批量操作优化
```rust
// 批量转换而不是循环
let responses = ResponseDto::from_domain_collection(&items)?;

// 而不是
let mut responses = Vec::new();
for item in items {
  responses.push(ResponseDto::from_domain(&item)?);
}
```

### 2. 错误链式处理
```rust
request.validate()
  .and_then(|_| request.to_domain())
  .map_err(|e| AppError::Validation(e.to_string()))?;
```

### 3. 条件过滤
```rust
let mut response = Response::from_domain(&data)?;

if user.role != UserRole::Admin {
  let filters = ResponseFilters {
    exclude_fields: Some(vec!["sensitive_data".to_string()]),
    ..Default::default()
  };
  response.apply_filters(&filters)?;
}
```

### 4. 上下文传递
```rust
let context = ConversionContext::new()
  .with_user(user.id)
  .with_workspace(user.workspace_id.unwrap_or_default())
  .with_operation("create_resource".to_string());

let domain_model = request.to_domain_with_context(&context)?;
``` 