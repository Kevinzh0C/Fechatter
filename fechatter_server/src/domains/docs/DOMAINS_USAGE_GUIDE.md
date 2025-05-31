# Domains Usage Guide for Services

## 📋 Overview

作为全人类最厉害的Rust工程师设计的Domains层架构使用指南。本文档详细说明Service层如何正确使用Domain层的业务逻辑、Repository抽象和领域模型，遵循Domain-Driven Design (DDD) 原则。

## 🏗️ Domain Architecture

```
┌─────────────────────────────────────────┐
│          Application Services           │ ← Use Case Orchestration
│  • Chat Application Service             │
│  • User Application Service             │
│  • Messaging Application Service        │
└─────────────────────────────────────────┘
                    ↓ calls
┌─────────────────────────────────────────┐
│            Domain Layer                 │ ← Business Logic
│  • Domain Services (业务逻辑)           │
│  • Domain Models (聚合根)               │
│  • Repository Traits (数据抽象)         │
│  • Domain Events (领域事件)             │
└─────────────────────────────────────────┘
                    ↓ implements
┌─────────────────────────────────────────┐
│        Infrastructure Layer             │ ← Technical Implementation
│  • Repository Implementations           │
│  • Database Operations                  │
│  • Event Publishers                     │
└─────────────────────────────────────────┘
```

## 📂 Domains Directory Structure

```
src/domains/
├── chat/                       # 💬 聊天领域
│   ├── chat_domain.rs         # 聊天业务逻辑服务
│   ├── repository.rs          # 聊天仓储trait和实现
│   ├── chat_member_repository.rs # 聊天成员仓储
│   └── events.rs             # 聊天领域事件
│
├── messaging/                  # 📧 消息领域
│   ├── messaging_domain.rs    # 消息业务逻辑服务
│   ├── repository.rs          # 消息仓储trait和实现
│   └── events.rs             # 消息领域事件
│
├── user/                       # 👤 用户领域
│   ├── user_domain.rs         # 用户业务逻辑服务
│   ├── repository.rs          # 用户仓储trait和实现
│   └── password.rs           # 密码管理业务逻辑
│
├── workspace/                  # 🏢 工作空间领域
│   ├── workspace_domain.rs    # 工作空间业务逻辑服务
│   ├── repository.rs          # 工作空间仓储trait和实现
│   └── events.rs             # 工作空间领域事件
│
└── notification/               # 🔔 通知领域
    ├── repository.rs          # 通知仓储trait和实现
    └── events.rs             # 通知领域事件
```

## 🎯 Domain Service Patterns

### 1. Chat Domain Service

#### Service Interface
```rust
use fechatter_server::domains::chat::chat_domain::{ChatDomainService, ChatDomainServiceImpl};
use fechatter_core::models::{Chat, ChatId, ChatSidebar, CreateChat, UserId};

#[async_trait]
pub trait ChatDomainService: Send + Sync {
  /// 创建聊天（包含业务规则验证）
  async fn create_chat(
    &self,
    input: CreateChat,
    created_by: i64,
    workspace_id: Option<i64>,
  ) -> Result<Chat, CoreError>;

  /// 获取聊天详情
  async fn get_chat(&self, chat_id: i64) -> Result<Option<Chat>, CoreError>;

  /// 获取用户聊天列表
  async fn get_user_chats(&self, user_id: i64) -> Result<Vec<ChatSidebar>, CoreError>;

  /// 更新聊天名称（包含权限检查）
  async fn update_chat_name(
    &self,
    chat_id: i64,
    user_id: i64,
    new_name: String,
  ) -> Result<Chat, CoreError>;

  /// 更新聊天描述（包含权限检查）
  async fn update_chat_description(
    &self,
    chat_id: i64,
    user_id: i64,
    new_description: String,
  ) -> Result<Chat, CoreError>;

  /// 删除聊天（包含权限检查）
  async fn delete_chat(&self, chat_id: i64, user_id: i64) -> Result<bool, CoreError>;
}
```

#### Service使用示例
```rust
use fechatter_server::domains::chat::{
  chat_domain::{ChatDomainServiceImpl, ChatConfig},
  repository::ChatRepository,
  chat_member_repository::ChatMemberRepository,
};

impl ChatApplicationService {
  pub async fn create_chat(&self, input: CreateChatInput) -> Result<ChatDetailView, AppError> {
    // 1. 获取Domain Service
    let chat_domain_service = self.get_chat_domain_service();
    
    // 2. 转换为领域模型
    let create_chat = CreateChat {
      name: input.name,
      chat_type: input.chat_type,
      description: input.description,
      members: input.initial_members,
    };
    
    // 3. 调用Domain Service（包含业务逻辑）
    let chat = chat_domain_service
      .create_chat(create_chat, input.created_by.into(), input.workspace_id.map(Into::into))
      .await
      .map_err(AppError::from)?;
    
    // 4. 处理副作用（缓存失效、事件发布等）
    self.invalidate_chat_caches(&[input.created_by]).await;
    
    // 5. 转换为应用层响应
    let member_count = self.chat_member_repository
      .count_members(ChatId(chat.id.into()))
      .await?;
      
    Ok(ChatDetailView::from_chat(chat, member_count as i32))
  }
  
  fn get_chat_domain_service(&self) -> ChatDomainServiceImpl {
    ChatDomainServiceImpl::new(
      self.chat_repository.clone(),
      self.chat_member_repository.clone(),
      ChatConfig::default(),
    )
  }
}
```

#### Repository接口
```rust
use fechatter_server::domains::chat::repository::ChatRepository;

// 聊天仓储 - 数据访问抽象
impl ChatRepository {
  /// 创建聊天
  pub async fn create_chat(
    &self,
    input: CreateChat,
    created_by: i64,
    workspace_id: Option<i64>,
  ) -> Result<Chat, CoreError>;
  
  /// 根据ID查找聊天
  pub async fn find_chat_by_id(&self, chat_id: i64) -> Result<Option<Chat>, CoreError>;
  
  /// 获取用户侧边栏聊天列表
  pub async fn get_sidebar_chats(&self, user_id: i64) -> Result<Vec<ChatSidebar>, CoreError>;
  
  /// 更新聊天名称
  pub async fn update_chat_name(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    new_name: &str,
  ) -> Result<Chat, CoreError>;
  
  /// 更新聊天描述
  pub async fn update_chat_description(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    new_description: &str,
  ) -> Result<Chat, CoreError>;
  
  /// 删除聊天
  pub async fn delete_chat(
    &self,
    chat_id: ChatId,
    user_id: UserId,
  ) -> Result<bool, CoreError>;
}
```

### 2. Messaging Domain Service

#### Service Interface
```rust
use fechatter_server::domains::messaging::messaging_domain::{
  MessagingDomainService, MessagingDomainServiceImpl
};

#[async_trait]
pub trait MessagingDomainService: Send + Sync {
  /// 发送消息（包含权限检查和业务规则）
  async fn send_message(
    &self,
    chat_id: i64,
    sender_id: i64,
    content: String,
    reply_to: Option<i64>,
    mentions: Option<Vec<i64>>,
    files: Option<Vec<String>>,
    idempotency_key: Option<String>,
  ) -> Result<Message, CoreError>;

  /// 编辑消息（包含权限检查）
  async fn edit_message(
    &self,
    message_id: i64,
    user_id: i64,
    new_content: String,
  ) -> Result<Message, CoreError>;

  /// 删除消息（包含权限检查）
  async fn delete_message(
    &self,
    message_id: i64,
    user_id: i64,
  ) -> Result<bool, CoreError>;

  /// 获取聊天消息列表
  async fn list_messages(
    &self,
    chat_id: i64,
    user_id: i64,
    limit: Option<i32>,
    before: Option<i64>,
  ) -> Result<Vec<Message>, CoreError>;
}
```

#### Service使用示例
```rust
impl MessageApplicationService {
  pub async fn send_message(
    &self,
    chat_id: ChatId,
    sender_id: UserId,
    create_message: CreateMessage,
  ) -> Result<Message, AppError> {
    // 1. 获取Messaging Domain Service
    let messaging_service = self.get_messaging_domain_service();
    
    // 2. 调用Domain Service（包含权限检查和业务逻辑）
    let message = messaging_service
      .send_message(
        chat_id.into(),
        sender_id.into(),
        create_message.content,
        create_message.reply_to.map(Into::into),
        create_message.mentions.map(|m| m.into_iter().map(Into::into).collect()),
        create_message.files,
        create_message.idempotency_key,
      )
      .await
      .map_err(AppError::from)?;
    
    // 3. 处理副作用
    self.invalidate_message_caches(chat_id, &[sender_id]).await;
    self.publish_message_sent_event(&message).await;
    
    Ok(message)
  }
  
  fn get_messaging_domain_service(&self) -> MessagingDomainServiceImpl {
    MessagingDomainServiceImpl::new(
      self.message_repository.clone(),
      self.chat_member_repository.clone(),
      MessagingConfig::default(),
    )
  }
}
```

### 3. User Domain Service

#### Service Interface
```rust
use fechatter_server::domains::user::{
  user_domain::{UserDomainService, UserDomainServiceImpl},
  password::PasswordService,
};

#[async_trait]
pub trait UserDomainService: Send + Sync {
  /// 创建用户（包含密码验证）
  async fn create_user(
    &self,
    create_user: CreateUser,
    workspace_id: i64,
  ) -> Result<User, CoreError>;

  /// 用户认证
  async fn authenticate_user(
    &self,
    email: &str,
    password: &str,
  ) -> Result<Option<User>, CoreError>;

  /// 更新用户资料
  async fn update_user_profile(
    &self,
    user_id: i64,
    fullname: Option<String>,
    email: Option<String>,
  ) -> Result<User, CoreError>;

  /// 修改密码
  async fn change_password(
    &self,
    user_id: i64,
    current_password: &str,
    new_password: &str,
  ) -> Result<(), CoreError>;

  /// 停用用户
  async fn suspend_user(&self, user_id: i64, admin_id: i64) -> Result<(), CoreError>;
}
```

#### Service使用示例
```rust
impl UserApplicationService {
  pub async fn create_user(&self, create_user_input: CreateUserInput) -> Result<User, AppError> {
    // 1. 获取User Domain Service
    let user_domain_service = self.get_user_domain_service();
    
    // 2. 转换为领域模型
    let create_user = CreateUser {
      email: create_user_input.email,
      fullname: create_user_input.fullname,
      password: create_user_input.password,
      workspace: create_user_input.workspace_name.unwrap_or_default(),
    };
    
    // 3. 调用Domain Service（包含密码验证等业务逻辑）
    let user = user_domain_service
      .create_user(create_user, create_user_input.workspace_id.unwrap_or_default())
      .await
      .map_err(AppError::from)?;
    
    // 4. 处理副作用
    self.invalidate_user_caches(user.id).await;
    self.publish_user_created_event(&user).await;
    
    Ok(user)
  }
  
  fn get_user_domain_service(&self) -> UserDomainServiceImpl {
    UserDomainServiceImpl::new(
      self.user_repository.clone(),
      self.password_service.clone(),
      UserConfig::default(),
    )
  }
}
```

### 4. Workspace Domain Service

#### Service Interface
```rust
use fechatter_server::domains::workspace::workspace_domain::{
  WorkspaceDomainService, WorkspaceDomainServiceImpl
};

#[async_trait]
pub trait WorkspaceDomainService: Send + Sync {
  /// 创建工作空间
  async fn create_workspace(
    &self,
    name: String,
    owner_id: i64,
  ) -> Result<Workspace, CoreError>;

  /// 邀请用户到工作空间
  async fn invite_user_to_workspace(
    &self,
    workspace_id: i64,
    inviter_id: i64,
    invitee_email: String,
  ) -> Result<(), CoreError>;

  /// 移除工作空间成员
  async fn remove_workspace_member(
    &self,
    workspace_id: i64,
    admin_id: i64,
    member_id: i64,
  ) -> Result<(), CoreError>;

  /// 转移工作空间所有权
  async fn transfer_ownership(
    &self,
    workspace_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<(), CoreError>;
}
```

## 🔄 Repository Pattern Usage

### 1. Repository接口设计
```rust
// Repository trait定义 - 抽象数据访问
#[async_trait]
pub trait ChatRepositoryTrait: Send + Sync {
  async fn create(&self, input: CreateChat) -> Result<Chat, CoreError>;
  async fn find_by_id(&self, id: ChatId) -> Result<Option<Chat>, CoreError>;
  async fn update(&self, chat: &Chat) -> Result<Chat, CoreError>;
  async fn delete(&self, id: ChatId) -> Result<bool, CoreError>;
  async fn list_by_user(&self, user_id: UserId) -> Result<Vec<Chat>, CoreError>;
}

// Repository具体实现 - PostgreSQL实现
pub struct PostgresChatRepository {
  pool: Arc<PgPool>,
}

#[async_trait]
impl ChatRepositoryTrait for PostgresChatRepository {
  async fn create(&self, input: CreateChat) -> Result<Chat, CoreError> {
    // SQL实现
    let chat = sqlx::query_as!(
      Chat,
      r#"
      INSERT INTO chats (name, chat_type, description, created_by, workspace_id)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id, name, chat_type as "chat_type: ChatType", description, 
                created_by, workspace_id, created_at, updated_at, chat_members
      "#,
      input.name,
      input.chat_type as ChatType,
      input.description,
      input.created_by,
      input.workspace_id,
    )
    .fetch_one(&*self.pool)
    .await?;
    
    Ok(chat)
  }
  
  // ... 其他方法实现
}
```

### 2. Service中Repository使用
```rust
impl ChatApplicationService {
  pub async fn get_chat(&self, chat_id: i64) -> Result<Option<ChatDetailView>, AppError> {
    // 1. 通过Repository抽象访问数据
    let chat = self.chat_repository
      .find_by_id(ChatId(chat_id))
      .await
      .map_err(AppError::from)?;
    
    if let Some(chat) = chat {
      // 2. 获取关联数据
      let member_count = self.chat_member_repository
        .count_members(ChatId(chat_id))
        .await
        .map_err(AppError::from)?;
      
      // 3. 构建应用层视图
      let detail_view = ChatDetailView::from_chat(chat, member_count as i32);
      Ok(Some(detail_view))
    } else {
      Ok(None)
    }
  }
}
```

## 🎯 Domain Model Integration

### 1. 聚合根使用
```rust
// 聊天聚合根
use fechatter_core::models::{Chat, ChatId, ChatType, UserId, WorkspaceId};

impl ChatApplicationService {
  pub async fn update_chat_with_business_rules(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    updates: ChatUpdateRequest,
  ) -> Result<Chat, AppError> {
    // 1. 加载聚合根
    let mut chat = self.chat_repository
      .find_by_id(chat_id)
      .await?
      .ok_or_else(|| AppError::NotFound(vec![format!("Chat {}", chat_id)]))?;
    
    // 2. 业务规则验证
    self.validate_update_permissions(&chat, user_id).await?;
    
    // 3. 应用业务逻辑
    if let Some(new_name) = updates.name {
      chat.update_name(new_name)?; // 聚合根内部验证
    }
    
    if let Some(new_description) = updates.description {
      chat.update_description(new_description)?; // 聚合根内部验证
    }
    
    // 4. 持久化聚合根
    let updated_chat = self.chat_repository
      .update(&chat)
      .await?;
    
    // 5. 发布领域事件
    self.publish_chat_updated_event(&updated_chat).await;
    
    Ok(updated_chat)
  }
}
```

### 2. 值对象使用
```rust
// 使用强类型ID
use fechatter_core::models::{UserId, ChatId, MessageId, WorkspaceId};

impl MessageApplicationService {
  pub async fn send_message(
    &self,
    chat_id: ChatId,        // 强类型，不会与其他ID混淆
    sender_id: UserId,      // 强类型，编译时安全
    content: String,
  ) -> Result<Message, AppError> {
    // 1. 类型安全的操作
    let message = Message {
      id: MessageId::default(), // 数据库生成
      chat_id,                  // 类型匹配
      sender_id,                // 类型匹配
      content,
      files: None,
      created_at: chrono::Utc::now(),
      idempotency_key: None,
    };
    
    // 2. Repository操作也是类型安全的
    self.message_repository
      .create(message)
      .await
      .map_err(AppError::from)
  }
}
```

## 🔧 Domain Events

### 1. 事件定义
```rust
// 领域事件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatDomainEvent {
  ChatCreated {
    chat_id: ChatId,
    creator_id: UserId,
    workspace_id: WorkspaceId,
    chat_type: ChatType,
    initial_members: Vec<UserId>,
  },
  ChatUpdated {
    chat_id: ChatId,
    updated_by: UserId,
    changes: ChatChanges,
  },
  ChatDeleted {
    chat_id: ChatId,
    deleted_by: UserId,
  },
  MemberAdded {
    chat_id: ChatId,
    added_by: UserId,
    new_member: UserId,
  },
  MemberRemoved {
    chat_id: ChatId,
    removed_by: UserId,
    removed_member: UserId,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChanges {
  pub name: Option<String>,
  pub description: Option<String>,
}
```

### 2. 事件发布
```rust
impl ChatApplicationService {
  async fn publish_chat_created_event(&self, chat: &Chat) {
    let event = ChatDomainEvent::ChatCreated {
      chat_id: chat.id,
      creator_id: chat.created_by,
      workspace_id: chat.workspace_id,
      chat_type: chat.chat_type,
      initial_members: chat.chat_members.clone(),
    };
    
    // 通过事件发布器发布事件
    if let Err(e) = self.event_publisher.publish(event).await {
      tracing::error!("Failed to publish chat created event: {}", e);
      // 不影响主流程，但记录错误
    }
  }
}
```

## 🚀 Best Practices

### 1. Domain Service使用原则
```rust
// ✅ 正确：Domain Service处理复杂业务逻辑
impl ChatApplicationService {
  pub async fn create_group_chat_with_validation(
    &self,
    input: CreateGroupChatInput,
  ) -> Result<Chat, AppError> {
    // 获取Domain Service
    let chat_domain_service = self.get_chat_domain_service();
    
    // Domain Service处理复杂的业务规则
    let chat = chat_domain_service
      .create_chat_with_member_validation(input)
      .await?;
    
    // Application Service处理跨聚合协调
    self.notify_members_about_new_chat(&chat).await;
    self.update_workspace_chat_count(chat.workspace_id).await;
    
    Ok(chat)
  }
}

// ❌ 错误：在Application Service中混合业务逻辑
impl ChatApplicationService {
  pub async fn create_chat_bad_example(
    &self,
    input: CreateChatInput,
  ) -> Result<Chat, AppError> {
    // ❌ 业务规则验证应该在Domain Service中
    if input.name.len() > 128 {
      return Err(AppError::Validation("Name too long".to_string()));
    }
    
    // ❌ 复杂的权限检查应该在Domain中
    if input.chat_type == ChatType::PrivateChannel {
      let user_role = self.get_user_role(input.created_by).await?;
      if user_role != UserRole::Admin {
        return Err(AppError::PermissionDenied("Admin required".to_string()));
      }
    }
    
    // ❌ 直接调用Repository，跳过了Domain层
    self.chat_repository.create(input).await.map_err(Into::into)
  }
}
```

### 2. Repository抽象原则
```rust
// ✅ 正确：通过Repository trait抽象数据访问
#[async_trait]
pub trait MessageRepository: Send + Sync {
  async fn create(&self, message: CreateMessage) -> Result<Message, CoreError>;
  async fn find_by_id(&self, id: MessageId) -> Result<Option<Message>, CoreError>;
  async fn list_by_chat(&self, chat_id: ChatId, limit: i32) -> Result<Vec<Message>, CoreError>;
  async fn update_content(&self, id: MessageId, new_content: &str) -> Result<Message, CoreError>;
  async fn delete(&self, id: MessageId) -> Result<bool, CoreError>;
}

impl MessagingApplicationService {
  pub async fn get_messages(
    &self,
    chat_id: ChatId,
    limit: i32,
  ) -> Result<Vec<Message>, AppError> {
    // ✅ 通过抽象接口访问，不依赖具体实现
    self.message_repository
      .list_by_chat(chat_id, limit)
      .await
      .map_err(AppError::from)
  }
}

// ❌ 错误：直接使用具体的数据库操作
impl MessagingApplicationService {
  pub async fn get_messages_bad_example(
    &self,
    chat_id: i64,
    limit: i32,
  ) -> Result<Vec<Message>, AppError> {
    // ❌ 直接SQL操作，违反了分层架构
    let messages = sqlx::query_as!(
      Message,
      "SELECT * FROM messages WHERE chat_id = $1 LIMIT $2",
      chat_id,
      limit
    )
    .fetch_all(&*self.pool)
    .await?;
    
    Ok(messages)
  }
}
```

### 3. 领域模型使用原则
```rust
// ✅ 正确：使用强类型ID保证类型安全
impl UserApplicationService {
  pub async fn get_user_chats(
    &self,
    user_id: UserId,    // 强类型，不会混淆
    workspace_id: WorkspaceId, // 强类型，编译时检查
  ) -> Result<Vec<ChatSidebar>, AppError> {
    // 类型安全的调用
    self.chat_repository
      .list_by_user_and_workspace(user_id, workspace_id)
      .await
      .map_err(AppError::from)
  }
}

// ❌ 错误：使用原始类型，容易出错
impl UserApplicationService {
  pub async fn get_user_chats_bad_example(
    &self,
    user_id: i64,      // 原始类型，容易与其他ID混淆
    workspace_id: i64, // 原始类型，可能传错参数
  ) -> Result<Vec<ChatSidebar>, AppError> {
    // ❌ 参数顺序错误，运行时才能发现
    self.chat_repository
      .list_by_user_and_workspace(workspace_id, user_id) // 参数顺序错了！
      .await
      .map_err(AppError::from)
  }
}
```

## 🔍 Error Handling in Domains

### 1. Domain错误分层
```rust
// Domain层错误
#[derive(Debug, thiserror::Error)]
pub enum ChatDomainError {
  #[error("Chat not found: {chat_id}")]
  ChatNotFound { chat_id: i64 },
  
  #[error("User {user_id} is not a member of chat {chat_id}")]
  NotMember { user_id: i64, chat_id: i64 },
  
  #[error("User {user_id} does not have permission to {action} in chat {chat_id}")]
  PermissionDenied { user_id: i64, chat_id: i64, action: String },
  
  #[error("Chat name '{name}' is invalid: {reason}")]
  InvalidChatName { name: String, reason: String },
  
  #[error("Chat has reached maximum member limit of {limit}")]
  MemberLimitExceeded { limit: usize },
}

// 转换为Application层错误
impl From<ChatDomainError> for AppError {
  fn from(err: ChatDomainError) -> Self {
    match err {
      ChatDomainError::ChatNotFound { chat_id } => {
        AppError::NotFound(vec![format!("Chat {}", chat_id)])
      },
      ChatDomainError::NotMember { .. } | 
      ChatDomainError::PermissionDenied { .. } => {
        AppError::PermissionDenied(err.to_string())
      },
      ChatDomainError::InvalidChatName { .. } => {
        AppError::Validation(err.to_string())
      },
      ChatDomainError::MemberLimitExceeded { .. } => {
        AppError::BusinessRuleViolation(err.to_string())
      },
    }
  }
}
```

### 2. Service中错误处理
```rust
impl ChatApplicationService {
  pub async fn update_chat_name(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    new_name: String,
  ) -> Result<Chat, AppError> {
    // Domain Service调用，自动转换错误
    let chat_domain_service = self.get_chat_domain_service();
    
    let updated_chat = chat_domain_service
      .update_chat_name(chat_id.into(), user_id.into(), new_name)
      .await
      .map_err(|e| {
        // 记录Domain层错误
        tracing::error!("Chat name update failed: {}", e);
        AppError::from(e)
      })?;
    
    // 处理成功后的副作用
    self.invalidate_chat_caches(&[user_id]).await;
    
    Ok(updated_chat)
  }
}
```

## 📖 Service Templates

### Template 1: 标准Domain Service调用
```rust
impl ApplicationService {
  pub async fn domain_operation(
    &self,
    input: DomainInput,
  ) -> Result<DomainResult, AppError> {
    // 1. 获取Domain Service
    let domain_service = self.get_domain_service();
    
    // 2. 调用Domain Service（包含业务逻辑）
    let result = domain_service
      .operation(input)
      .await
      .map_err(|e| {
        tracing::error!("Domain operation failed: {}", e);
        AppError::from(e)
      })?;
    
    // 3. 处理应用层副作用
    self.handle_side_effects(&result).await;
    
    // 4. 返回结果
    Ok(result)
  }
  
  fn get_domain_service(&self) -> DomainServiceImpl {
    DomainServiceImpl::new(
      self.repository.clone(),
      self.config.clone(),
    )
  }
  
  async fn handle_side_effects(&self, result: &DomainResult) {
    // 缓存失效
    self.invalidate_caches(result).await;
    
    // 事件发布
    if let Err(e) = self.publish_domain_event(result).await {
      tracing::warn!("Failed to publish domain event: {}", e);
    }
  }
}
```

### Template 2: 跨聚合协调
```rust
impl ApplicationService {
  pub async fn cross_aggregate_operation(
    &self,
    input: CrossAggregateInput,
  ) -> Result<OperationResult, AppError> {
    // 1. 获取多个Domain Service
    let chat_service = self.get_chat_domain_service();
    let user_service = self.get_user_domain_service();
    let messaging_service = self.get_messaging_domain_service();
    
    // 2. 协调多个聚合的操作
    // 创建聊天
    let chat = chat_service
      .create_chat(input.chat_data)
      .await?;
    
    // 添加初始成员
    for user_id in input.initial_members {
      user_service
        .validate_user_exists(user_id)
        .await?;
    }
    
    // 发送欢迎消息
    let welcome_message = messaging_service
      .send_system_message(chat.id, "Welcome to the chat!")
      .await?;
    
    // 3. 构建结果
    Ok(OperationResult {
      chat,
      welcome_message,
      member_count: input.initial_members.len(),
    })
  }
}
```

---

## 总结

这个Domains架构设计确保了：
- 🎯 **业务逻辑集中**: Domain Service承载核心业务规则
- 🔒 **数据访问抽象**: Repository模式解耦数据访问
- ⚡ **类型安全**: 强类型ID系统防止错误
- 🛡️ **错误处理**: 分层错误处理和转换
- 📚 **易于测试**: Domain Service易于单元测试

遵循这个指南，Service层将能够正确使用Domain层的业务逻辑和数据抽象！🎉 