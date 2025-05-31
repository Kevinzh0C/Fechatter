# Fechatter架构数据结构使用指南

## 概述

本文档定义了Fechatter项目中DTOs（数据传输对象）和Domain（领域层）的标准使用模式，基于**Clean Architecture**和**DDD（领域驱动设计）**原则。

## 架构层次和数据流

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   API Layer     │    │   Handler Layer │    │  Domain Layer   │
│                 │    │                 │    │                 │
│ Request DTOs ──→│───→│ Function-based ─│───→│ Aggregates &    │
│ Response DTOs ←─│←───│ Single Responsibility│ │ Domain Services │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 1. DTOs (数据传输对象) 使用规范

### 1.1 Request DTOs - 输入验证和转换

**位置**: `src/dtos/models/requests/`

**职责**:
- API输入数据的强类型定义
- 业务规则前置验证  
- 转换为Domain模型

**标准模式**:
```rust
// 示例：发送消息请求DTO
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SendMessageRequest {
  #[validate(length(max = 4000))]
  pub content: String,
  
  pub files: Option<Vec<String>>,
  
  pub idempotency_key: Option<Uuid>,
  
  pub reply_to: Option<i64>,
  
  pub mentions: Option<Vec<i64>>,
}

impl RequestDto for SendMessageRequest {
  type DomainModel = fechatter_core::CreateMessage;
  
  fn to_domain(&self) -> Result<Self::DomainModel, ConversionError> {
    Ok(fechatter_core::CreateMessage {
      content: self.content.clone(),
      files: self.files.clone().unwrap_or_default(),
      idempotency_key: self.idempotency_key.map(|u| u.to_string()),
      reply_to: self.reply_to,
      mentions: self.mentions.clone(),
    })
  }
}
```

### 1.2 Response DTOs - 输出格式化

**位置**: `src/dtos/models/responses/`

**职责**:
- API输出数据的标准化格式
- 隐藏内部实现细节
- 提供前端友好的数据结构

**标准模式**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
  pub id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Option<Vec<String>>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub reply_to: Option<i64>,
  pub mentions: Option<Vec<i64>>,
}

impl ResponseDto for MessageResponse {
  type DomainModel = fechatter_core::Message;
  
  fn from_domain(domain: &Self::DomainModel) -> Result<Self, ConversionError> {
    Ok(Self {
      id: domain.id.into(),
      chat_id: domain.chat_id.into(),
      sender_id: domain.sender_id.into(),
      content: domain.content.clone(),
      files: domain.files.clone(),
      created_at: domain.created_at,
      reply_to: domain.reply_to.map(|id| id.into()),
      mentions: domain.mentions.clone(),
    })
  }
}
```

### 1.3 统一响应包装

**所有API响应必须使用标准包装器**:
```rust
// 单个实体响应
pub type ApiResult<T> = Result<ApiResponse<T>, AppError>;

// 列表响应  
pub type ListResult<T> = Result<ApiResponse<PaginatedResponse<T>>, AppError>;

// 操作响应
pub type OperationResult = Result<ApiResponse<OperationResponse>, AppError>;
```

## 2. Domain Layer 使用规范

### 2.1 Aggregates (聚合根) - 业务逻辑封装

**位置**: `src/domains/{domain}/entities.rs`

**职责**:
- 封装核心业务规则
- 维护数据一致性
- 提供业务行为方法

**标准模式**:
```rust
#[derive(Debug, Clone)]
pub struct MessageAggregate {
  pub message: Message,
}

impl MessageAggregate {
  // 工厂方法
  pub fn new(message: Message) -> Self {
    Self { message }
  }
  
  // 业务规则方法
  pub fn can_edit(&self, user_id: i64) -> bool {
    i64::from(self.message.sender_id) == user_id
  }
  
  pub fn can_delete(&self, user_id: i64) -> bool {
    i64::from(self.message.sender_id) == user_id
  }
  
  // 业务查询方法
  pub fn has_attachments(&self) -> bool {
    self.message.files.as_ref().map_or(false, |files| !files.is_empty())
  }
  
  pub fn is_reply(&self) -> bool {
    self.message.reply_to.is_some()
  }
}
```

### 2.2 Domain Services - 复杂业务逻辑

**位置**: `src/domains/{domain}/{domain}_domain.rs`

**职责**:
- 协调多个聚合
- 实现复杂业务流程
- 调用外部服务

**标准模式**:
```rust
#[async_trait]
pub trait MessageDomainService: Send + Sync {
  async fn send_message(
    &self,
    message: CreateMessage,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Message, CoreError>;
  
  async fn edit_message(
    &self,
    id: i64,
    content: String,
    editor_id: i64,
  ) -> Result<Message, CoreError>;
}
```

### 2.3 Repositories - 数据访问抽象

**位置**: `src/domains/{domain}/repository.rs`

**职责**:
- 数据持久化抽象
- 查询接口定义
- 事务边界管理

## 3. Handler Functions 职责划分标准

### 3.1 Handler函数标准结构

每个Handler函数必须严格按照以下步骤执行：

```rust
pub async fn {action}_{resource}_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(id): Path<i64>,
  Json(payload): Json<RequestDto>,
) -> Result<impl IntoResponse, AppError> {
  // 1. 输入验证 (Input Validation)
  validate_input(&payload)?;
  
  // 2. 权限检查 (Authorization)
  check_permissions(&state, &user, id).await?;
  
  // 3. DTO转换 (DTO Conversion)  
  let domain_input = payload.to_domain()?;
  
  // 4. 业务逻辑调用 (Business Logic)
  let result = execute_business_logic(&state, domain_input).await?;
  
  // 5. 响应构建 (Response Building)
  let response = build_response(result)?;
  
  // 6. 副作用处理 (Side Effects)
  handle_side_effects(&state, &result).await?;
  
  Ok(response)
}
```

### 3.2 职责分离原则

#### **3.2.1 输入验证函数**
- **职责**: 验证请求数据的有效性
- **返回**: `Result<(), ValidationError>`
- **不允许**: 访问数据库、调用外部服务

```rust
fn validate_input(payload: &RequestDto) -> Result<(), AppError> {
  // 只做数据格式和业务规则验证
}
```

#### **3.2.2 权限检查函数**  
- **职责**: 验证用户操作权限
- **返回**: `Result<(), AppError>`
- **允许**: 查询用户权限、成员关系

```rust
async fn check_permissions(
  state: &AppState, 
  user: &AuthUser, 
  resource_id: i64
) -> Result<(), AppError> {
  // 只做权限相关的数据库查询
}
```

#### **3.2.3 业务逻辑执行函数**
- **职责**: 调用Domain Service执行核心业务
- **返回**: `Result<DomainModel, AppError>`
- **不允许**: 直接数据库操作、权限检查

```rust
async fn execute_business_logic(
  state: &AppState,
  input: DomainInput,
) -> Result<DomainModel, AppError> {
  // 只调用Domain Service
}
```

#### **3.2.4 响应构建函数**
- **职责**: 将Domain结果转换为API响应
- **返回**: `Result<ApiResponse<ResponseDto>, AppError>`
- **不允许**: 业务逻辑、数据库访问

```rust
fn build_response(domain_result: DomainModel) -> Result<impl IntoResponse, AppError> {
  // 只做数据转换和响应构建
}
```

#### **3.2.5 副作用处理函数**
- **职责**: 处理事件发布、缓存更新等
- **返回**: `Result<(), AppError>`
- **允许**: 异步通知、缓存操作

```rust
async fn handle_side_effects(
  state: &AppState,
  result: &DomainModel,
) -> Result<(), AppError> {
  // 事件发布、实时通知、缓存更新
}
```

## 4. 命名规范

### 4.1 Handler函数命名
- 格式: `{action}_{resource}_handler`
- 示例: `create_message_handler`, `list_messages_handler`

### 4.2 DTO命名
- Request: `{Action}{Resource}Request`
- Response: `{Resource}Response`
- 示例: `SendMessageRequest`, `MessageResponse`

### 4.3 Domain函数命名
- Service方法: `{action}_{resource}`
- 示例: `send_message`, `edit_message`

## 5. 错误处理标准

### 5.1 错误转换链
```
Domain Error → Core Error → App Error → API Error → HTTP Response
```

### 5.2 标准错误响应
```rust
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
  pub error_code: String,
  pub error_message: String,
  pub details: Option<serde_json::Value>,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub request_id: String,
}
```

## 6. 测试策略

### 6.1 单元测试覆盖
- DTOs转换测试
- Domain业务逻辑测试  
- Handler函数职责测试

### 6.2 集成测试
- 完整请求流程测试
- 数据库事务测试
- API契约测试

## 7. 性能优化原则

### 7.1 零成本抽象
- 编译时多态
- 避免运行时反射
- 内联小函数

### 7.2 内存优化
- 避免不必要的克隆
- 使用引用传递
- 合理使用Arc和Rc

## 8. 迁移策略

### 8.1 渐进式重构
1. 先实现新的ResponseDto
2. 更新Handler使用新的转换框架
3. 逐步迁移现有代码
4. 移除过时的映射器

### 8.2 向后兼容
- 保持API接口不变
- 内部实现逐步升级
- 提供过渡期支持

## 9. 代码审查清单

### 9.1 必检项目
- [ ] Handler函数职责单一
- [ ] DTO转换正确实现
- [ ] 错误处理完整
- [ ] 权限检查到位
- [ ] 性能影响评估

### 9.2 架构一致性
- [ ] 符合Clean Architecture原则
- [ ] 遵循DDD设计模式
- [ ] 依赖方向正确
- [ ] 模块边界清晰

---

**注意**: 此文档为活文档，随着项目演进持续更新。所有开发者必须严格遵循此规范，确保代码质量和架构一致性。 