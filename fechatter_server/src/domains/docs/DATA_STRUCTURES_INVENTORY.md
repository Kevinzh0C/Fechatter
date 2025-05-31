# Fechatter 数据结构目录清单

## 概述
本文档列出了Fechatter项目中所有已定义的数据结构，包括DTOs、Domain实体和Core模型。

---

## 1. fechatter_core 核心数据结构

### 1.1 基础模型 (`fechatter_core/src/models/`)

#### **用户相关**
```rust
// models/user.rs
pub struct User {
  pub id: UserId,
  pub fullname: String,
  pub email: String,
  pub password_hash: Option<String>,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
  pub workspace_id: WorkspaceId,
}

pub struct AuthUser {
  pub id: UserId,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
  pub workspace_id: WorkspaceId,
}

pub struct ChatUser {
  pub id: UserId,
  pub fullname: String,
  pub email: String,
}

pub enum UserStatus {
  Suspended,
  Active,
}
```

#### **JWT和认证相关**
```rust
// models/jwt.rs
pub struct UserClaims {
  pub id: UserId,
  pub workspace_id: WorkspaceId,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
}

pub struct RefreshToken {
  pub id: i64,
  pub user_id: UserId,
  pub token_hash: String,
  pub expires_at: DateTime<Utc>,
  pub issued_at: DateTime<Utc>,
  pub revoked: bool,
  pub replaced_by: Option<String>,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
  pub absolute_expires_at: DateTime<Utc>,
}

pub struct AuthTokens {
  pub access_token: String,
  pub refresh_token: RefreshTokenData,
}

pub struct RefreshTokenData {
  pub token: String,
  pub expires_at: DateTime<Utc>,
  pub absolute_expires_at: DateTime<Utc>,
}
```

#### **消息相关**
```rust
// models/message.rs
pub struct Message {
  pub id: MessageId,
  pub chat_id: ChatId,
  pub sender_id: UserId,
  pub content: String,
  pub files: Option<Vec<String>>,
  pub created_at: DateTime<Utc>,
  pub idempotency_key: Option<String>,
  pub reply_to: Option<MessageId>,
  pub mentions: Option<Vec<i64>>,
}

pub struct CreateMessage {
  pub content: String,
  pub files: Vec<String>,
  pub idempotency_key: Option<String>,
  pub reply_to: Option<i64>,
  pub mentions: Option<Vec<i64>>,
}

pub struct ListMessages {
  pub limit: i32,
  pub last_id: Option<i64>,
}

pub struct SearchMessages {
  pub query: String,
  pub chat_id: Option<i64>,
  pub sender_id: Option<i64>,
  pub start_date: Option<DateTime<Utc>>,
  pub end_date: Option<DateTime<Utc>>,
  pub has_files: Option<bool>,
  pub limit: Option<i32>,
  pub offset: Option<i32>,
}

pub struct SearchResult {
  pub messages: Vec<Message>,
  pub total_count: i64,
  pub has_more: bool,
}
```

#### **聊天相关**
```rust
// models/chat.rs
pub struct Chat {
  pub id: ChatId,
  pub name: Option<String>,
  pub chat_type: ChatType,
  pub created_by: UserId,
  pub created_at: DateTime<Utc>,
  pub workspace_id: WorkspaceId,
}

pub struct CreateChat {
  pub name: Option<String>,
  pub members: Vec<i64>,
  pub chat_type: ChatType,
}

pub struct UpdateChat {
  pub name: Option<String>,
  pub description: Option<String>,
}

pub enum ChatType {
  Single,
  Group,
  PrivateChannel,
  PublicChannel,
}

pub struct ChatSidebar {
  pub id: ChatId,
  pub name: Option<String>,
  pub chat_type: ChatType,
  pub created_at: DateTime<Utc>,
  pub members: Vec<ChatUser>,
  pub last_message: Option<Message>,
  pub unread_count: i64,
}
```

#### **工作空间相关**
```rust
// models/workspace.rs
pub struct Workspace {
  pub id: WorkspaceId,
  pub name: String,
  pub owner_id: UserId,
  pub created_at: DateTime<Utc>,
}
```

#### **强类型ID**
```rust
// models/ids.rs
pub struct UserId(pub i64);
pub struct ChatId(pub i64);
pub struct MessageId(pub i64);
pub struct WorkspaceId(pub i64);
```

#### **请求/响应模型**
```rust
// models/mod.rs
pub struct CreateUser {
  pub fullname: String,
  pub email: String,
  pub workspace: String,
  pub password: String,
}

pub struct SigninUser {
  pub email: String,
  pub password: String,
}
```

### 1.2 错误类型 (`fechatter_core/src/error.rs`)
```rust
pub enum CoreError {
  Authentication(String),
  InternalError(String),
  ValidationError(String),
  NotFound(String),
}
```

---

## 2. fechatter_server DTOs 数据传输对象

### 2.1 请求DTOs (`src/dtos/models/requests/`)

#### **认证请求**
```rust
// requests/auth.rs
pub struct LoginRequest {
  pub email: String,
  pub password: String,
}

pub struct RegisterRequest {
  pub fullname: String,
  pub email: String,
  pub password: String,
  pub workspace_name: String,
}

pub struct RefreshTokenRequest {
  pub refresh_token: String,
}
```

#### **消息请求**
```rust
// requests/message.rs
pub struct SendMessageRequest {
  pub content: String,
  pub files: Option<Vec<String>>,
  pub idempotency_key: Option<Uuid>,
  pub reply_to: Option<i64>,
  pub mentions: Option<Vec<i64>>,
}

pub struct EditMessageRequest {
  pub content: String,
}

pub struct DeleteMessageRequest {
  pub reason: Option<String>,
  pub delete_for_everyone: Option<bool>,
}

pub struct ListMessagesRequest {
  pub limit: Option<i32>,
  pub before_id: Option<i64>,
  pub after_id: Option<i64>,
  pub before_timestamp: Option<DateTime<Utc>>,
  pub after_timestamp: Option<DateTime<Utc>>,
}

pub struct SearchMessagesRequest {
  pub query: String,
  pub chat_id: Option<i64>,
  pub sender_id: Option<i64>,
  pub start_date: Option<DateTime<Utc>>,
  pub end_date: Option<DateTime<Utc>>,
  pub has_files: Option<bool>,
  pub limit: Option<i32>,
  pub offset: Option<i32>,
}

pub struct MarkMessagesReadRequest {
  pub message_ids: Vec<i64>,
}

pub struct MessageReactionRequest {
  pub emoji: String,
  pub add: bool,
}

pub struct ForwardMessageRequest {
  pub target_chat_ids: Vec<i64>,
  pub comment: Option<String>,
}

pub struct PinMessageRequest {
  pub pin: bool,
  pub reason: Option<String>,
}

pub struct BatchDeleteMessagesRequest {
  pub message_ids: Vec<i64>,
  pub reason: Option<String>,
  pub delete_for_everyone: Option<bool>,
}

pub struct MessageStatsRequest {
  pub start_date: DateTime<Utc>,
  pub end_date: DateTime<Utc>,
  pub group_by: Option<String>,
}

pub struct ExportMessagesRequest {
  pub start_date: Option<DateTime<Utc>>,
  pub end_date: Option<DateTime<Utc>>,
  pub format: Option<String>,
  pub include_files: Option<bool>,
}
```

#### **聊天请求**
```rust
// requests/chat.rs
pub struct CreateChatRequest {
  pub name: Option<String>,
  pub chat_type: String,
  pub members: Vec<i64>,
  pub description: Option<String>,
}

pub struct UpdateChatRequest {
  pub name: Option<String>,
  pub description: Option<String>,
}

pub struct AddMembersRequest {
  pub user_ids: Vec<i64>,
}

pub struct RemoveMembersRequest {
  pub user_ids: Vec<i64>,
  pub reason: Option<String>,
}

pub struct TransferOwnershipRequest {
  pub new_owner_id: i64,
}
```

#### **用户请求**
```rust
// requests/user.rs
pub struct UpdateUserProfileRequest {
  pub fullname: Option<String>,
  pub email: Option<String>,
}

pub struct ChangePasswordRequest {
  pub current_password: String,
  pub new_password: String,
}

pub struct UpdateUserSettingsRequest {
  pub timezone: Option<String>,
  pub language: Option<String>,
  pub notification_settings: Option<serde_json::Value>,
}

pub struct SearchUsersRequest {
  pub query: String,
  pub workspace_id: Option<i64>,
  pub limit: Option<i32>,
  pub offset: Option<i32>,
}
```

#### **工作空间请求**
```rust
// requests/workspace.rs
pub struct CreateWorkspaceRequest {
  pub name: String,
  pub description: Option<String>,
}

pub struct UpdateWorkspaceRequest {
  pub name: Option<String>,
  pub description: Option<String>,
}

pub struct InviteUserRequest {
  pub email: String,
  pub role: Option<String>,
}

pub struct UpdateMemberRoleRequest {
  pub user_id: i64,
  pub role: String,
}
```

### 2.2 响应DTOs (`src/dtos/models/responses/`)

#### **通用响应**
```rust
// responses/common.rs
pub struct ApiResponse<T> {
  pub success: bool,
  pub message: Option<String>,
  pub data: Option<T>,
  pub timestamp: DateTime<Utc>,
}

pub struct PaginatedResponse<T> {
  pub items: Vec<T>,
  pub total: i64,
  pub page: i32,
  pub per_page: i32,
  pub total_pages: i32,
  pub has_next: bool,
  pub has_prev: bool,
}

pub struct OperationResponse {
  pub success: bool,
  pub message: String,
  pub affected_rows: Option<i64>,
  pub operation_id: Option<String>,
}

pub struct ErrorResponse {
  pub error_code: String,
  pub error_message: String,
  pub details: Option<serde_json::Value>,
  pub timestamp: DateTime<Utc>,
  pub request_id: Option<String>,
}

pub struct StatsResponse {
  pub start_date: DateTime<Utc>,
  pub end_date: DateTime<Utc>,
  pub metrics: serde_json::Value,
  pub generated_at: DateTime<Utc>,
}

pub struct HealthResponse {
  pub status: String,
  pub version: String,
  pub uptime_seconds: u64,
  pub services: Vec<ServiceHealth>,
  pub timestamp: DateTime<Utc>,
}

pub struct ServiceHealth {
  pub name: String,
  pub status: String,
  pub response_time_ms: Option<u64>,
  pub message: Option<String>,
}
```

#### **认证响应**
```rust
// responses/auth.rs
pub struct LoginResponse {
  pub access_token: String,
  pub refresh_token: String,
  pub expires_in: i64,
  pub user: UserResponse,
}

pub struct UserResponse {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub workspace_id: i64,
  pub created_at: DateTime<Utc>,
}

pub struct RefreshTokenResponse {
  pub access_token: String,
  pub expires_in: i64,
}
```

#### **消息响应**
```rust
// responses/message.rs
pub struct MessageResponse {
  pub id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Option<Vec<String>>,
  pub created_at: DateTime<Utc>,
  pub reply_to: Option<i64>,
  pub mentions: Option<Vec<i64>>,
  pub is_edited: bool,
  pub idempotency_key: Option<String>,
}

pub struct MessageCreateResponse {
  pub success: bool,
  pub message: String,
  pub data: MessageResponse,
}

pub struct MessageOperationResponse {
  pub success: bool,
  pub message: String,
  pub affected_rows: Option<i64>,
  pub timestamp: DateTime<Utc>,
}
```

### 2.3 核心框架 (`src/dtos/core/`)

#### **转换框架**
```rust
// core/conversion.rs
pub trait RequestDto {
  type DomainModel;
  fn to_domain(&self) -> Result<Self::DomainModel, ConversionError>;
}

pub trait ResponseDto {
  type DomainModel;
  fn from_domain(domain: &Self::DomainModel) -> Result<Self, ConversionError>;
  fn from_domain_collection(domains: &[Self::DomainModel]) -> Result<Vec<Self>, ConversionError>;
}

pub enum ConversionError {
  ValidationFailed(String),
  TypeMismatch(String),
  MissingField(String),
  InvalidFormat(String),
}
```

#### **验证框架**
```rust
// core/validation.rs
pub trait BaseDto {
  fn dto_type() -> &'static str;
  fn validate(&self) -> Result<(), DtoValidationError>;
}

pub struct DtoValidationError {
  pub error_type: ValidationErrorType,
  pub message: String,
  pub field: Option<String>,
}

pub enum ValidationErrorType {
  Required,
  Format,
  Range,
  Length,
  Custom,
}
```

#### **分页框架**
```rust
// core/pagination.rs
pub struct PaginationRequest {
  pub page: u32,
  pub page_size: u32,
  pub sort_by: Option<String>,
  pub sort_order: Option<SortOrder>,
}

pub enum SortOrder {
  Asc,
  Desc,
}

pub struct CursorPagination {
  pub cursor: Option<String>,
  pub limit: u32,
}
```

---

## 3. fechatter_server Domains 领域实体

### 3.1 消息领域 (`src/domains/messaging/`)

#### **实体和聚合**
```rust
// messaging/entities.rs
pub struct MessageAggregate {
  pub message: Message,
}

pub struct MessageValidationRules {
  pub max_content_length: usize,
  pub max_file_count: usize,
  pub allowed_file_types: Vec<String>,
  pub edit_time_limit: chrono::Duration,
}

pub struct MessageValidator {
  rules: MessageValidationRules,
}

pub struct MessageStats {
  pub total_messages: i64,
  pub messages_with_files: i64,
  pub reply_messages: i64,
  pub messages_with_mentions: i64,
  pub average_message_length: f64,
}
```

#### **领域服务**
```rust
// messaging/messaging_domain.rs
pub trait MessageDomainService: Send + Sync {
  async fn send_message(&self, message: CreateMessage, chat_id: i64, user_id: i64) -> Result<Message, CoreError>;
  async fn get_message(&self, id: i64) -> Result<Option<Message>, CoreError>;
  async fn list_messages(&self, input: ListMessages, chat_id: i64, user_id: i64) -> Result<Vec<Message>, CoreError>;
  async fn edit_message(&self, id: i64, content: String, editor_id: i64) -> Result<Message, CoreError>;
  async fn delete_message(&self, id: i64, user_id: i64) -> Result<(), CoreError>;
  async fn get_messages_count(&self, chat_id: i64) -> Result<i64, CoreError>;
}

pub struct MessageConfig {
  pub cache_enabled: bool,
  pub cache_ttl: u64,
  pub max_content_length: usize,
  pub max_file_count: usize,
}

pub struct MessageDomainServiceImpl {
  repository: Arc<MessageRepository>,
  config: MessageConfig,
}
```

#### **事件**
```rust
// messaging/events.rs
pub struct MessageSent {
  pub message_id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content_preview: String,
  pub timestamp: DateTime<Utc>,
}

pub struct MessageEdited {
  pub message_id: i64,
  pub chat_id: i64,
  pub editor_id: i64,
  pub old_content: String,
  pub new_content: String,
  pub timestamp: DateTime<Utc>,
}

pub struct MessageDeleted {
  pub message_id: i64,
  pub chat_id: i64,
  pub deleted_by: i64,
  pub timestamp: DateTime<Utc>,
}
```

### 3.2 聊天领域 (`src/domains/chat/`)

#### **实体和聚合**
```rust
// chat/entities.rs
pub struct ChatAggregate {
  pub chat: Chat,
  pub members: Vec<ChatMember>,
  pub settings: ChatSettings,
}

pub struct ChatMember {
  pub id: i64,
  pub chat_id: i64,
  pub user_id: i64,
  pub role: ChatRole,
  pub joined_at: DateTime<Utc>,
  pub is_active: bool,
}

pub enum ChatRole {
  Owner,
  Admin,
  Member,
}

pub struct ChatSettings {
  pub allow_member_invite: bool,
  pub message_retention_days: Option<i32>,
  pub allow_file_sharing: bool,
  pub max_members: Option<i32>,
}

pub struct ChatPermissions {
  pub can_send_messages: bool,
  pub can_add_members: bool,
  pub can_remove_members: bool,
  pub can_edit_settings: bool,
  pub can_delete_chat: bool,
}

pub struct ChatStatistics {
  pub total_messages: i64,
  pub total_members: i64,
  pub active_members_today: i64,
  pub last_activity: Option<DateTime<Utc>>,
}
```

### 3.3 用户领域 (`src/domains/user/`)

#### **实体和聚合**
```rust
// user/entities.rs
pub struct UserAggregate {
  pub user: User,
}

pub struct UserPermissions {
  pub can_create_workspace: bool,
  pub can_invite_users: bool,
  pub can_manage_users: bool,
  pub is_workspace_admin: bool,
}

pub struct UserProfile {
  pub user_id: i64,
  pub display_name: String,
  pub avatar_url: Option<String>,
  pub bio: Option<String>,
  pub timezone: String,
  pub language: String,
}

pub struct UserActivity {
  pub user_id: i64,
  pub last_seen: DateTime<Utc>,
  pub is_online: bool,
  pub status_message: Option<String>,
}
```

### 3.4 工作空间领域 (`src/domains/workspace/`)

#### **实体和聚合**
```rust
// workspace/entities.rs  
pub struct WorkspaceAggregate {
  pub workspace: Workspace,
  pub members: Vec<WorkspaceMember>,
  pub settings: WorkspaceSettings,
}

pub struct WorkspaceMember {
  pub id: i64,
  pub workspace_id: i64,
  pub user_id: i64,
  pub role: WorkspaceRole,
  pub joined_at: DateTime<Utc>,
  pub invited_by: Option<i64>,
}

pub enum WorkspaceRole {
  Owner,
  Admin,
  Member,
  Guest,
}

pub struct WorkspaceSettings {
  pub allow_guest_access: bool,
  pub require_email_verification: bool,
  pub default_chat_retention_days: Option<i32>,
  pub max_file_size_mb: i32,
}

pub struct WorkspaceStatistics {
  pub total_members: i64,
  pub total_chats: i64,
  pub total_messages: i64,
  pub storage_used_bytes: i64,
  pub created_at: DateTime<Utc>,
}
```

### 3.5 通知领域 (`src/domains/notification/`)

#### **实体和聚合**
```rust
// notification/entities.rs
pub struct Notification {
  pub id: i64,
  pub user_id: i64,
  pub notification_type: NotificationType,
  pub title: String,
  pub content: String,
  pub is_read: bool,
  pub created_at: DateTime<Utc>,
  pub metadata: Option<serde_json::Value>,
}

pub enum NotificationType {
  MessageMention,
  ChatInvite,
  WorkspaceInvite,
  SystemAlert,
  Custom,
}

pub struct NotificationSettings {
  pub user_id: i64,
  pub email_notifications: bool,
  pub push_notifications: bool,
  pub mention_notifications: bool,
  pub chat_notifications: bool,
  pub quiet_hours_start: Option<chrono::NaiveTime>,
  pub quiet_hours_end: Option<chrono::NaiveTime>,
}
```

---

## 4. 映射器 (`src/dtos/mappers/`)

```rust
// mappers/message_mappers.rs
pub struct MessageMapper;

// mappers/chat_mappers.rs  
pub struct ChatMapper;

// mappers/user_mappers.rs
pub struct UserMapper;
```

---

## 5. 重构指南

基于以上数据结构清单，重构handlers时应该：

1. **使用现有的ResponseDto** - 如`MessageResponse`、`MessageOperationResponse`
2. **复用RequestDto** - 如`SendMessageRequest`、`ListMessagesRequest`  
3. **调用Domain Services** - 如`MessageDomainService`
4. **使用Core模型** - 如`Message`、`CreateMessage`
5. **应用聚合方法** - 如`MessageAggregate::can_edit()`

这样可以确保重构后的handlers与现有架构完全一致，避免重复定义数据结构。 