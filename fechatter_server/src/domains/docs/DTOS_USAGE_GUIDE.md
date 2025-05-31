# DTOs Usage Guide for Handlers

## 📋 Overview

作为全人类最厉害的Rust工程师设计的DTOs (Data Transfer Objects) 架构使用指南。本文档详细说明Handler层如何正确使用DTOs进行数据验证、转换和响应构建，遵循Clean Architecture的接口适配器模式。

## 🏗️ DTOs Architecture

```
┌─────────────────────────────────────────┐
│               Handler Layer             │ ← HTTP Endpoint
│  • 接收 Request DTOs                     │
│  • 返回 Response DTOs                    │
│  • 使用 Mappers 进行转换                 │
└─────────────────────────────────────────┘
                    ↓ uses
┌─────────────────────────────────────────┐
│              DTOs Layer                 │ ← Interface Adapters
│  • Request DTOs (验证+转换)              │
│  • Response DTOs (格式化+过滤)           │
│  • Mappers (数据映射)                    │
│  • Core (转换+验证+分页)                 │
└─────────────────────────────────────────┘
                    ↓ converts
┌─────────────────────────────────────────┐
│           Domain Models                 │ ← Business Logic
│  • User, Chat, Message 等               │
│  • 纯业务逻辑，无外部依赖                │
└─────────────────────────────────────────┘
```

## 📂 DTOs Directory Structure

```
src/dtos/
├── core/                    # 🧠 核心功能模块
│   ├── conversion.rs       # 类型转换和错误处理
│   ├── validation.rs       # 数据验证框架
│   ├── response.rs         # 统一响应格式
│   ├── pagination.rs       # 分页处理
│   └── mod.rs              # 核心trait定义
│
├── models/                  # 📦 具体DTO模型
│   ├── requests/           # 📥 请求DTOs
│   │   ├── auth.rs        # 认证请求
│   │   ├── chat.rs        # 聊天请求
│   │   ├── message.rs     # 消息请求
│   │   ├── user.rs        # 用户请求
│   │   └── workspace.rs   # 工作空间请求
│   │
│   └── responses/          # 📤 响应DTOs
│       ├── auth.rs        # 认证响应
│       ├── message.rs     # 消息响应
│       └── common.rs      # 通用响应
│
└── mappers/                 # 🔄 数据映射器
    ├── user_mappers.rs     # 用户数据映射
    ├── chat_mappers.rs     # 聊天数据映射
    └── message_mappers.rs  # 消息数据映射
```

## 🎯 DTOs Core Traits

### 1. BaseDto - 基础DTO特征
```rust
pub trait BaseDto: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync {
  /// DTO类型标识
  fn dto_type() -> &'static str;
  
  /// 数据验证
  fn validate(&self) -> Result<(), DtoValidationError>;
  
  /// 元数据获取
  fn metadata(&self) -> DtoMetadata;
}
```

### 2. RequestDto - 请求DTO特征
```rust
pub trait RequestDto: BaseDto {
  type DomainModel;
  
  /// 转换为领域模型
  fn to_domain(&self) -> Result<Self::DomainModel, ConversionError>;
  
  /// 预处理请求数据
  fn preprocess(&mut self) -> Result<(), DtoValidationError>;
  
  /// 获取业务上下文
  fn business_context(&self) -> BusinessContext;
}
```

### 3. ResponseDto - 响应DTO特征
```rust
pub trait ResponseDto: BaseDto {
  type DomainModel;
  
  /// 从领域模型创建
  fn from_domain(domain: &Self::DomainModel) -> Result<Self, ConversionError>;
  
  /// 批量转换
  fn from_domain_collection(domains: &[Self::DomainModel]) -> Result<Vec<Self>, ConversionError>;
  
  /// 应用响应过滤器
  fn apply_filters(&mut self, filters: &ResponseFilters) -> Result<(), ConversionError>;
}
```

## 📥 Request DTOs Usage

### 1. 认证请求DTOs
```rust
use crate::dtos::models::requests::auth::{LoginRequest, RegisterRequest};

pub async fn login_handler(
  State(state): State<AppState>,
  Json(mut login_request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
  // 1. 预处理请求数据
  login_request.preprocess().map_err(AppError::from)?;
  
  // 2. 验证请求数据
  login_request.validate().map_err(AppError::from)?;
  
  // 3. 转换为领域模型
  let signin_user = login_request.to_domain().map_err(AppError::from)?;
  
  // 4. 调用业务服务
  let auth_service = state.auth_service();
  let tokens = auth_service.signin(&signin_user, None).await?;
  
  // 5. 转换为响应DTO
  let response = AuthResponse::from_domain(&tokens)?;
  
  Ok(Json(response))
}
```

### 2. 聊天请求DTOs
```rust
use crate::dtos::models::requests::chat::CreateChatRequest;

pub async fn create_chat_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(mut create_request): Json<CreateChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
  // 1. 添加业务上下文
  let mut context = create_request.business_context();
  context.user_id = Some(user.id);
  context.workspace_id = user.workspace_id.map(Into::into);
  
  // 2. 验证和预处理
  create_request.preprocess()?;
  create_request.validate()?;
  
  // 3. 转换并调用服务
  let create_chat_input = create_request.to_domain()?;
  let chat_service = state.chat_application_service()?;
  let chat_detail = chat_service.create_chat(create_chat_input).await?;
  
  // 4. 构建响应
  let response = ChatResponse::from_domain(&chat_detail)?;
  Ok(Json(response))
}
```

### 3. 消息请求DTOs
```rust
use crate::dtos::models::requests::message::SendMessageRequest;

pub async fn send_message_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(mut message_request): Json<SendMessageRequest>,
) -> Result<Json<MessageCreateResponse>, AppError> {
  // 1. 设置发送者信息
  message_request.sender_id = Some(user.id.into());
  message_request.chat_id = Some(chat_id.into());
  
  // 2. 验证请求
  message_request.validate()?;
  
  // 3. 转换并发送
  let create_message = message_request.to_domain()?;
  let messaging_service = state.messaging_service()?;
  let message = messaging_service
    .send_message(chat_id.into(), user.id, create_message)
    .await?;
  
  // 4. 构建响应
  let message_response = MessageResponse::from_domain(&message)?;
  let response = MessageCreateResponse {
    success: true,
    message: "Message sent successfully".to_string(),
    data: message_response,
  };
  
  Ok(Json(response))
}
```

## 📤 Response DTOs Usage

### 1. 标准响应格式
```rust
use crate::dtos::models::responses::{
  common::{ApiResponse, SuccessResponse, ErrorResponse},
  message::{MessageResponse, MessageListResponse}
};

// 成功响应
pub async fn list_messages_handler(
  State(state): State<AppState>,
  Path(chat_id): Path<i64>,
  Query(query): Query<ListMessagesQuery>,
) -> Result<Json<ApiResponse<MessageListResponse>>, AppError> {
  // 1. 调用服务获取数据
  let messaging_service = state.messaging_service()?;
  let messages = messaging_service.list_messages(chat_id.into(), query).await?;
  
  // 2. 批量转换为响应DTO
  let message_responses = MessageResponse::from_domain_collection(&messages)?;
  
  // 3. 构建标准响应
  let response = ApiResponse::success(
    message_responses,
    format!("Retrieved {} messages", messages.len())
  );
  
  Ok(Json(response))
}

// 错误响应
pub async fn error_example() -> Result<Json<ApiResponse<()>>, AppError> {
  let error_response = ApiResponse::error(
    "CHAT_NOT_FOUND".to_string(),
    "The specified chat does not exist".to_string(),
    Some(serde_json::json!({
      "chat_id": 123,
      "user_id": 456
    }))
  );
  
  Ok(Json(error_response))
}
```

### 2. 分页响应
```rust
use crate::dtos::core::pagination::{PaginatedResponse, PaginationQuery};

pub async fn paginated_users_handler(
  State(state): State<AppState>,
  Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<UserResponse>>, AppError> {
  // 1. 获取分页数据
  let user_service = state.user_application_service()?;
  let (users, total) = user_service
    .list_users_paginated(pagination.page, pagination.limit)
    .await?;
  
  // 2. 转换为响应DTOs
  let user_responses = UserResponse::from_domain_collection(&users)?;
  
  // 3. 构建分页响应
  let response = PaginatedResponse::new(
    user_responses,
    pagination.page,
    pagination.limit,
    total,
  );
  
  Ok(Json(response))
}
```

### 3. 响应过滤
```rust
use crate::dtos::core::{ResponseFilters, SensitiveDataPolicy};

pub async fn user_profile_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(user_id): Path<i64>,
) -> Result<Json<UserResponse>, AppError> {
  // 1. 获取用户数据
  let user_service = state.user_application_service()?;
  let target_user = user_service.get_user(user_id.into()).await?;
  
  // 2. 创建响应DTO
  let mut response = UserResponse::from_domain(&target_user)?;
  
  // 3. 应用过滤器（基于权限）
  let filters = if user.id == user_id {
    // 自己的信息，显示全部
    ResponseFilters {
      sensitive_data_policy: SensitiveDataPolicy::ShowAll,
      ..Default::default()
    }
  } else {
    // 他人信息，隐藏敏感数据
    ResponseFilters {
      exclude_fields: Some(vec!["email".to_string(), "phone".to_string()]),
      sensitive_data_policy: SensitiveDataPolicy::HideSensitive,
      ..Default::default()
    }
  };
  
  response.apply_filters(&filters)?;
  
  Ok(Json(response))
}
```

## 🔄 Mappers Usage

### 1. 使用映射器进行复杂转换
```rust
use crate::dtos::mappers::{ChatMapper, MessageMapper, UserMapper};

pub async fn chat_detail_handler(
  State(state): State<AppState>,
  Path(chat_id): Path<i64>,
) -> Result<Json<ChatDetailResponse>, AppError> {
  // 1. 获取聊天详情
  let chat_service = state.chat_application_service()?;
  let chat = chat_service.get_chat(chat_id).await?;
  
  // 2. 获取相关数据
  let members = chat_service.get_chat_members(chat_id).await?;
  let recent_messages = state
    .messaging_service()?
    .list_recent_messages(chat_id.into(), 10)
    .await?;
  
  // 3. 使用映射器转换
  let chat_response = ChatMapper::to_detail_response(&chat, &members, &recent_messages)?;
  
  Ok(Json(chat_response))
}

// 自定义映射器示例
pub struct ChatMapper;

impl ChatMapper {
  pub fn to_detail_response(
    chat: &Chat,
    members: &[User],
    recent_messages: &[Message],
  ) -> Result<ChatDetailResponse, ConversionError> {
    // 转换聊天基础信息
    let chat_info = ChatResponse::from_domain(chat)?;
    
    // 转换成员信息
    let member_responses = UserResponse::from_domain_collection(members)?;
    
    // 转换最近消息
    let message_responses = MessageResponse::from_domain_collection(recent_messages)?;
    
    Ok(ChatDetailResponse {
      chat: chat_info,
      members: member_responses,
      recent_messages: message_responses,
      member_count: members.len() as i32,
      last_activity: recent_messages
        .first()
        .map(|m| m.created_at)
        .unwrap_or(chat.created_at),
    })
  }
}
```

## ⚡ Data Validation

### 1. 自动验证
```rust
use validator::Validate;

pub async fn register_handler(
  State(state): State<AppState>,
  Json(register_request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
  // 1. 使用validator自动验证
  register_request.validate()
    .map_err(|e| AppError::Validation(format!("Validation failed: {:?}", e)))?;
  
  // 2. 转换并处理
  let create_user = register_request.to_domain()?;
  
  // ... 业务逻辑
}
```

### 2. 自定义验证
```rust
use crate::dtos::core::validation::{ValidationContext, DtoValidationError};

impl RequestDto for CreateChatRequest {
  type DomainModel = CreateChatInput;
  
  fn to_domain(&self) -> Result<Self::DomainModel, ConversionError> {
    // 自定义验证逻辑
    if self.name.trim().is_empty() {
      return Err(ConversionError::new(
        ConversionErrorType::InvalidFormat,
        "Chat name cannot be empty".to_string(),
        "CreateChatRequest".to_string(),
        "CreateChatInput".to_string(),
      ).with_field("name".to_string()));
    }
    
    if let Some(ref members) = self.members {
      if members.len() > 50 {
        return Err(ConversionError::value_out_of_range(
          "members",
          &members.len().to_string(),
          Some("0"),
          Some("50"),
        ));
      }
    }
    
    // 转换为领域模型
    Ok(CreateChatInput {
      name: self.name.clone(),
      chat_type: self.chat_type,
      description: self.description.clone(),
      created_by: self.created_by.unwrap_or_default().into(),
      workspace_id: self.workspace_id.map(Into::into),
      initial_members: self.members.clone().unwrap_or_default(),
      members: self.members.clone(),
    })
  }
}
```

## 🔧 Error Handling

### 1. 转换错误处理
```rust
use crate::dtos::core::conversion::{ConversionError, ConversionErrorType};

pub async fn safe_conversion_handler(
  Json(request): Json<SomeRequest>,
) -> Result<Json<SomeResponse>, AppError> {
  // 转换为领域模型，处理各种转换错误
  let domain_model = request.to_domain()
    .map_err(|e| match e.error_type {
      ConversionErrorType::MissingField => AppError::Validation(
        format!("Missing required field: {}", e.failed_field.unwrap_or_default())
      ),
      ConversionErrorType::TypeMismatch => AppError::Validation(
        format!("Type mismatch: {}", e.message)
      ),
      ConversionErrorType::ValueOutOfRange => AppError::Validation(
        format!("Value out of range: {}", e.message)
      ),
      ConversionErrorType::BusinessRuleViolation => AppError::BusinessRuleViolation(e.message),
      _ => AppError::InternalServerError(format!("Conversion error: {}", e.message)),
    })?;
  
  // ... 处理业务逻辑
}
```

### 2. 批量转换错误处理
```rust
use crate::dtos::core::conversion::{BatchConverter, BatchErrorStrategy};

pub async fn batch_create_handler(
  State(state): State<AppState>,
  Json(requests): Json<Vec<CreateItemRequest>>,
) -> Result<Json<BatchCreateResponse>, AppError> {
  // 创建批量转换器
  let converter = BatchConverter::new(
    Box::new(CreateItemConverter),
    BatchErrorStrategy::CollectErrors,
  );
  
  // 执行批量转换
  let context = ConversionContext::new();
  let result = converter.convert_batch(requests, &context);
  
  // 处理成功和失败的项目
  let successful_items: Vec<_> = result.successful
    .into_iter()
    .map(|item| item.item)
    .collect();
    
  let errors: Vec<_> = result.failed
    .into_iter()
    .map(|error| format!("Item {}: {}", error.index, error.error.message))
    .collect();
  
  // 构建批量响应
  let response = BatchCreateResponse {
    successful: successful_items,
    errors,
    stats: result.stats,
  };
  
  Ok(Json(response))
}
```

## 📖 Handler Templates

### Template 1: 标准CRUD Handler
```rust
pub async fn standard_crud_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(mut request): Json<CreateResourceRequest>,
) -> Result<Json<ApiResponse<ResourceResponse>>, AppError> {
  // 1. 设置请求上下文
  request.set_user_context(user.id, user.workspace_id.map(Into::into));
  
  // 2. 验证和预处理
  request.preprocess()?;
  request.validate()?;
  
  // 3. 转换为领域模型
  let domain_input = request.to_domain()
    .map_err(|e| AppError::Validation(e.to_string()))?;
  
  // 4. 调用业务服务
  let service = state.resource_service()?;
  let resource = service.create_resource(domain_input).await?;
  
  // 5. 转换为响应DTO
  let response_dto = ResourceResponse::from_domain(&resource)
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;
  
  // 6. 构建API响应
  let api_response = ApiResponse::success(
    response_dto,
    "Resource created successfully".to_string(),
  );
  
  Ok(Json(api_response))
}
```

### Template 2: 分页查询Handler
```rust
pub async fn paginated_query_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Query(pagination): Query<PaginationQuery>,
  Query(filters): Query<ResourceFilterQuery>,
) -> Result<Json<PaginatedResponse<ResourceResponse>>, AppError> {
  // 1. 验证分页参数
  pagination.validate()?;
  filters.validate()?;
  
  // 2. 转换查询参数
  let query_params = ResourceQueryParams {
    pagination: pagination.clone(),
    filters: filters.to_domain()?,
    user_context: BusinessContext {
      user_id: Some(user.id),
      workspace_id: user.workspace_id.map(Into::into),
      ..Default::default()
    },
  };
  
  // 3. 调用服务
  let service = state.resource_service()?;
  let (resources, total) = service
    .list_resources_paginated(query_params)
    .await?;
  
  // 4. 批量转换响应
  let response_dtos = ResourceResponse::from_domain_collection(&resources)
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;
  
  // 5. 构建分页响应
  let paginated_response = PaginatedResponse::new(
    response_dtos,
    pagination.page,
    pagination.limit,
    total,
  );
  
  Ok(Json(paginated_response))
}
```

### Template 3: 文件上传Handler
```rust
pub async fn file_upload_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, AppError> {
  let mut upload_requests = Vec::new();
  
  // 1. 解析multipart数据
  while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
    let name = field.name().unwrap_or_default().to_string();
    let filename = field.file_name().map(|s| s.to_string());
    let content_type = field.content_type().map(|s| s.to_string());
    let data = field.bytes().await.map_err(AppError::from)?;
    
    // 构建上传请求DTO
    let upload_request = FileUploadRequest {
      filename: filename.unwrap_or_default(),
      content_type,
      data: data.to_vec(),
      user_id: user.id,
      workspace_id: user.workspace_id.map(Into::into),
    };
    
    // 验证文件
    upload_request.validate()?;
    upload_requests.push(upload_request);
  }
  
  // 2. 批量处理文件
  let storage_service = state.storage_service();
  let mut file_responses = Vec::new();
  
  for upload_request in upload_requests {
    let file_info = storage_service
      .upload_file(upload_request.to_domain()?)
      .await?;
    
    let file_response = FileResponse::from_domain(&file_info)?;
    file_responses.push(file_response);
  }
  
  // 3. 构建响应
  let response = FileUploadResponse {
    success: true,
    message: format!("Uploaded {} files successfully", file_responses.len()),
    files: file_responses,
  };
  
  Ok(Json(response))
}
```

## 🚀 Best Practices

### 1. DTO设计原则
- ✅ **单一职责**: 每个DTO只负责一种数据传输场景
- ✅ **不可变性**: DTO字段尽量使用不可变类型
- ✅ **验证优先**: 在转换为领域模型前完成所有验证
- ✅ **错误友好**: 提供清晰的错误消息和上下文
- ❌ **避免嵌套**: 避免过深的DTO嵌套结构
- ❌ **避免泄漏**: 不要在DTO中暴露内部实现细节

### 2. 转换最佳实践
```rust
// ✅ 好的转换实现
impl RequestDto for CreateUserRequest {
  type DomainModel = CreateUser;
  
  fn to_domain(&self) -> Result<Self::DomainModel, ConversionError> {
    // 1. 数据清洗
    let email = self.email.trim().to_lowercase();
    let fullname = self.fullname.trim().to_string();
    
    // 2. 业务规则验证
    if email.is_empty() {
      return Err(ConversionError::missing_field("email", "CreateUserRequest", "CreateUser"));
    }
    
    if !email.contains('@') {
      return Err(ConversionError::new(
        ConversionErrorType::InvalidFormat,
        "Invalid email format".to_string(),
        "CreateUserRequest".to_string(),
        "CreateUser".to_string(),
      ).with_field("email".to_string()).with_value(email));
    }
    
    // 3. 安全转换
    Ok(CreateUser {
      email,
      fullname,
      password: self.password.clone(), // 注意：实际中应该hash密码
      workspace: self.workspace_name.clone().unwrap_or_default(),
    })
  }
}

// ❌ 不好的转换实现
impl RequestDto for BadCreateUserRequest {
  type DomainModel = CreateUser;
  
  fn to_domain(&self) -> Result<Self::DomainModel, ConversionError> {
    // ❌ 没有验证，直接转换
    // ❌ 没有错误处理
    // ❌ 没有数据清洗
    Ok(CreateUser {
      email: self.email.clone(),
      fullname: self.fullname.clone(),
      password: self.password.clone(),
      workspace: "default".to_string(),
    })
  }
}
```

### 3. 响应构建最佳实践
```rust
// ✅ 好的响应构建
pub async fn good_response_handler() -> Result<Json<ApiResponse<UserResponse>>, AppError> {
  // 1. 获取数据
  let user = get_user_from_service().await?;
  
  // 2. 安全转换
  let mut user_response = UserResponse::from_domain(&user)
    .map_err(|e| AppError::InternalServerError(format!("Response conversion failed: {}", e)))?;
  
  // 3. 应用过滤器
  let filters = ResponseFilters {
    sensitive_data_policy: SensitiveDataPolicy::MaskSensitive,
    ..Default::default()
  };
  user_response.apply_filters(&filters)?;
  
  // 4. 构建标准响应
  let response = ApiResponse::success(
    user_response,
    "User information retrieved successfully".to_string(),
  );
  
  Ok(Json(response))
}

// ❌ 不好的响应构建
pub async fn bad_response_handler() -> Result<Json<User>, AppError> {
  // ❌ 直接返回领域模型
  // ❌ 没有错误处理
  // ❌ 没有数据过滤
  // ❌ 没有统一的响应格式
  let user = get_user_from_service().await?;
  Ok(Json(user)) // 直接暴露内部模型
}
```

## 🔍 Debugging and Testing

### 1. DTO转换调试
```rust
use tracing::{info, warn, error};

pub async fn debug_conversion_handler(
  Json(request): Json<CreateResourceRequest>,
) -> Result<Json<ResourceResponse>, AppError> {
  // 调试请求DTO
  info!("Received request: {:?}", request);
  info!("Request metadata: {:?}", request.metadata());
  
  // 验证阶段调试
  if let Err(validation_error) = request.validate() {
    warn!("Validation failed: {:?}", validation_error);
    return Err(AppError::Validation(validation_error.to_string()));
  }
  
  // 转换阶段调试
  let domain_model = match request.to_domain() {
    Ok(model) => {
      info!("Conversion successful: {:?}", model);
      model
    }
    Err(conversion_error) => {
      error!("Conversion failed: {:?}", conversion_error);
      error!("Error context: {:?}", conversion_error.context);
      return Err(AppError::InternalServerError(conversion_error.to_string()));
    }
  };
  
  // ... 处理业务逻辑
}
```

### 2. DTO单元测试
```rust
#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_create_user_request_validation() {
    // 测试有效请求
    let valid_request = CreateUserRequest {
      email: "test@example.com".to_string(),
      password: "password123".to_string(),
      fullname: "Test User".to_string(),
      workspace_name: Some("Test Workspace".to_string()),
    };
    
    assert!(valid_request.validate().is_ok());
    assert!(valid_request.to_domain().is_ok());
    
    // 测试无效邮箱
    let invalid_email_request = CreateUserRequest {
      email: "invalid-email".to_string(),
      password: "password123".to_string(),
      fullname: "Test User".to_string(),
      workspace_name: None,
    };
    
    assert!(invalid_email_request.validate().is_err());
    
    // 测试密码过短
    let short_password_request = CreateUserRequest {
      email: "test@example.com".to_string(),
      password: "123".to_string(),
      fullname: "Test User".to_string(),
      workspace_name: None,
    };
    
    assert!(short_password_request.validate().is_err());
  }
  
  #[test]
  fn test_response_dto_conversion() {
    // 创建测试数据
    let user = User {
      id: UserId(1),
      email: "test@example.com".to_string(),
      fullname: "Test User".to_string(),
      // ... 其他字段
    };
    
    // 测试转换
    let response = UserResponse::from_domain(&user).unwrap();
    assert_eq!(response.id, 1);
    assert_eq!(response.email, "test@example.com");
    assert_eq!(response.fullname, "Test User");
    
    // 测试批量转换
    let users = vec![user.clone(), user.clone()];
    let responses = UserResponse::from_domain_collection(&users).unwrap();
    assert_eq!(responses.len(), 2);
  }
}
```

---

## 总结

这个DTOs架构设计确保了：
- 🎯 **类型安全**: 强类型转换和验证机制
- 🔒 **数据安全**: 响应过滤和敏感数据处理
- ⚡ **性能优化**: 批量转换和缓存支持
- 🛡️ **错误处理**: 详细的错误信息和上下文
- 📚 **易于使用**: 清晰的API和丰富的文档

遵循这个指南，你的Handler将能够安全、高效地处理各种数据传输场景！🎉 