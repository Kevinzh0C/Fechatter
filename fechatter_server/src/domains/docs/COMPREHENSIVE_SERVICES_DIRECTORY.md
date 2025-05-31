# Fechatter 全面服务层目录清单
> **核心理念**：通过清晰的Service分层，实现Handler专注业务协调，API专注数据传输

## 📋 服务层架构概览

### 🏗️ 4层Service架构设计

```rust
// Layer 1: Core Services (fechatter_core/src/services/) 
基础契约服务 → Repository接口 → 核心业务规则

// Layer 2: Domain Services (fechatter_server/src/domains/)
领域服务 → 业务逻辑封装 → 领域事件 → 聚合根管理

// Layer 3: Application Services (fechatter_server/src/services/application/)
用例编排 → 跨域协调 → 事务边界 → 缓存策略

// Layer 4: Infrastructure Services (fechatter_server/src/services/infrastructure/)
外部集成 → 技术实现 → 第三方API → 存储抽象
```

---

## 🎯 1. Core Services 基础服务层 (`fechatter_core/src/services/`)

### 1.1 核心契约服务
**职责**: 定义业务契约，提供基础服务接口

| 服务名称 | 文件 | 核心职责 | Handler支持能力 |
|---------|------|----------|---------------|
| **认证服务** | `auth_service.rs` | JWT管理、身份验证、权限检查 | ✅ 为Handler提供认证抽象 |
| **工作空间服务** | `workspace_service.rs` | 工作空间管理、成员管理 | ✅ 为Handler提供工作空间操作 |
| **服务提供者** | `service_provider.rs` | 依赖注入、服务定位、生命周期管理 | ✅ 为Handler提供统一服务访问 |

#### 1.1.1 认证服务抽象
```rust
// 🎯 Handler解耦的认证支持
pub trait AuthServiceTrait {
    // 基础认证操作
    async fn authenticate_user(&self, credentials: &Credentials) -> Result<AuthResult>;
    async fn verify_token(&self, token: &str) -> Result<TokenClaims>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthTokens>;
    
    // 权限检查
    async fn check_permission(&self, user_id: UserId, resource: Resource) -> Result<bool>;
    async fn get_user_permissions(&self, user_id: UserId) -> Result<Vec<Permission>>;
}
```

#### 1.1.2 服务提供者抽象
```rust
// 🎯 Handler统一的服务访问
pub trait ServiceProvider {
    fn auth_service(&self) -> &dyn AuthServiceTrait;
    fn workspace_service(&self) -> &dyn WorkspaceServiceTrait;
    fn messaging_service(&self) -> &dyn MessagingServiceTrait;
    fn notification_service(&self) -> &dyn NotificationServiceTrait;
}
```

---

## 🚀 2. Domain Services 领域服务层 (`fechatter_server/src/domains/`)

### 2.1 核心业务域服务
**职责**: 封装业务逻辑，管理领域对象，处理业务规则

| 业务域 | 目录 | 领域服务 | Handler解耦效果 |
|-------|------|---------|---------------|
| **消息传递** | `messaging/` | MessageDomainService | Handler专注消息操作协调 |
| **聊天管理** | `chat/` | ChatDomainService | Handler专注聊天室管理协调 |
| **用户管理** | `user/` | UserDomainService | Handler专注用户操作协调 |
| **工作空间** | `workspace/` | WorkspaceDomainService | Handler专注工作空间协调 |
| **通知系统** | `notification/` | NotificationDomainService | Handler专注通知协调 |

#### 2.1.1 消息传递域服务
```rust
// 🎯 从Handler抽象的消息业务逻辑
pub struct MessageDomainService {
    repository: Arc<dyn MessageRepository>,
    event_publisher: Arc<dyn DomainEventPublisher>,
}

impl MessageDomainService {
    // 核心业务逻辑
    pub async fn send_message(&self, command: SendMessageCommand) -> DomainResult<MessageAggregate>;
    pub async fn edit_message(&self, command: EditMessageCommand) -> DomainResult<MessageAggregate>;
    pub async fn delete_message(&self, command: DeleteMessageCommand) -> DomainResult<()>;
    
    // 业务规则验证
    pub async fn validate_message_permissions(&self, user_id: UserId, message_id: MessageId) -> DomainResult<()>;
    pub async fn validate_chat_access(&self, user_id: UserId, chat_id: ChatId) -> DomainResult<()>;
    
    // 领域事件处理
    async fn publish_message_sent_event(&self, message: &MessageAggregate);
    async fn publish_message_edited_event(&self, message: &MessageAggregate);
}
```

#### 2.1.2 聊天管理域服务
```rust
// 🎯 从Handler抽象的聊天业务逻辑
pub struct ChatDomainService {
    repository: Arc<dyn ChatRepository>,
    member_repository: Arc<dyn ChatMemberRepository>,
}

impl ChatDomainService {
    // 聊天室管理
    pub async fn create_chat(&self, command: CreateChatCommand) -> DomainResult<ChatAggregate>;
    pub async fn add_member(&self, command: AddMemberCommand) -> DomainResult<()>;
    pub async fn remove_member(&self, command: RemoveMemberCommand) -> DomainResult<()>;
    
    // 权限检查
    pub async fn check_member_permissions(&self, user_id: UserId, chat_id: ChatId) -> DomainResult<ChatPermissions>;
    pub async fn validate_admin_permissions(&self, user_id: UserId, chat_id: ChatId) -> DomainResult<()>;
}
```

#### 2.1.3 用户管理域服务
```rust
// 🎯 从Handler抽象的用户业务逻辑
pub struct UserDomainService {
    repository: Arc<dyn UserRepository>,
    workspace_service: Arc<dyn WorkspaceServiceTrait>,
}

impl UserDomainService {
    // 用户生命周期管理
    pub async fn create_user(&self, command: CreateUserCommand) -> DomainResult<UserAggregate>;
    pub async fn update_profile(&self, command: UpdateProfileCommand) -> DomainResult<UserAggregate>;
    pub async fn deactivate_user(&self, command: DeactivateUserCommand) -> DomainResult<()>;
    
    // 用户关系管理
    pub async fn add_to_workspace(&self, user_id: UserId, workspace_id: WorkspaceId) -> DomainResult<()>;
    pub async fn validate_user_workspace_access(&self, user_id: UserId, workspace_id: WorkspaceId) -> DomainResult<()>;
}
```

### 2.2 领域事件处理
**职责**: 管理领域事件的发布和处理

| 事件类型 | 文件 | 事件职责 | 跨域协调能力 |
|---------|------|---------|-------------|
| **消息事件** | `messaging/events.rs` | 消息创建、编辑、删除事件 | ✅ 触发通知、索引更新 |
| **聊天事件** | `chat/events.rs` | 聊天室创建、成员变更事件 | ✅ 触发权限更新、通知 |
| **用户事件** | `user/events.rs` | 用户注册、状态变更事件 | ✅ 触发欢迎消息、统计 |

---

## 🔧 3. Application Services 应用服务层 (`fechatter_server/src/services/application/`)

### 3.1 核心应用服务
**职责**: 用例编排、跨域协调、事务管理

| 应用服务 | 文件 | 用例职责 | Handler简化程度 |
|---------|------|---------|---------------|
| **用户应用服务** | `user_app_service.rs` | 用户用例编排、缓存管理 | ✅ Handler仅需调用单一方法 |
| **认证应用服务** | `auth_app_service.rs` | 认证用例编排、会话管理 | ✅ Handler仅需调用认证方法 |
| **聊天应用服务** | `chat_app_service.rs` | 聊天用例编排、权限协调 | ✅ Handler专注API层协调 |
| **通知应用服务** | `notification_app_service.rs` | 通知用例编排、推送管理 | ✅ Handler专注通知触发 |

#### 3.1.1 用户应用服务
```rust
// 🎯 Handler的用户操作简化接口
#[async_trait]
pub trait UserServiceTrait {
    // 用例：用户资料管理
    async fn get_user_profile(&self, user_id: i64) -> Result<UserProfileView>;
    async fn update_user_profile(&self, user_id: i64, input: UpdateUserInput) -> Result<UserProfileView>;
    async fn change_password(&self, user_id: i64, input: ChangePasswordInput) -> Result<()>;
    
    // 用例：用户状态管理  
    async fn update_user_status(&self, user_id: i64, status: UserStatus) -> Result<()>;
    async fn validate_users_exist(&self, user_ids: Vec<i64>) -> Result<()>;
    
    // 用例：工作空间用户管理
    async fn find_users_by_workspace(&self, workspace_id: i64) -> Result<Vec<UserProfileView>>;
}
```

#### 3.1.2 聊天应用服务
```rust
// 🎯 Handler的聊天操作简化接口
#[async_trait]
pub trait ChatServiceTrait {
    // 用例：聊天室管理
    async fn create_chat(&self, input: CreateChatInput) -> Result<ChatView>;
    async fn get_chat_details(&self, chat_id: i64, user_id: i64) -> Result<ChatDetailView>;
    async fn update_chat_settings(&self, chat_id: i64, input: UpdateChatInput) -> Result<ChatView>;
    
    // 用例：成员管理
    async fn add_chat_member(&self, chat_id: i64, user_id: i64, inviter_id: i64) -> Result<()>;
    async fn remove_chat_member(&self, chat_id: i64, user_id: i64, remover_id: i64) -> Result<()>;
    async fn get_chat_members(&self, chat_id: i64) -> Result<Vec<ChatMemberView>>;
    
    // 用例：权限管理
    async fn check_chat_access(&self, user_id: i64, chat_id: i64) -> Result<ChatAccessLevel>;
    async fn update_member_role(&self, chat_id: i64, user_id: i64, role: ChatRole) -> Result<()>;
}
```

### 3.2 支持应用服务
**职责**: 提供横切关注点支持

| 支持服务 | 文件 | 支持职责 | Handler解耦效果 |
|---------|------|---------|---------------|
| **消息流服务** | `message_stream.rs` | 实时消息处理、WebSocket管理 | ✅ Handler专注业务，流处理透明 |
| **索引同步服务** | `indexer_sync_service.rs` | 搜索索引管理、数据同步 | ✅ Handler无需关心搜索索引 |
| **缓存策略服务** | `cache_strategy_service.rs` | 统一缓存管理、失效策略 | ✅ Handler无需关心缓存逻辑 |
| **事件发布服务** | `application_event_publisher.rs` | 应用事件发布、异步处理 | ✅ Handler仅需触发事件 |

#### 3.2.1 缓存策略服务
```rust
// 🎯 统一的缓存抽象，Handler无需关心缓存实现
pub struct CacheStrategyService {
    redis_cache: Option<Arc<RedisCacheService>>,
    memory_cache: Arc<MemoryCache>,
}

impl CacheStrategyService {
    // 统一缓存接口
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>;
    pub async fn set<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<()>;
    pub async fn invalidate(&self, pattern: &str) -> Result<()>;
    
    // 业务特定缓存方法
    pub async fn cache_user_profile(&self, user_id: i64, profile: &UserProfileView);
    pub async fn cache_chat_members(&self, chat_id: i64, members: &[ChatMemberView]);
    pub async fn invalidate_user_caches(&self, user_id: i64);
    
    // 缓存键生成
    pub fn user_profile_key(user_id: i64) -> String;
    pub fn chat_members_key(chat_id: i64) -> String;
    pub fn user_permissions_key(user_id: i64, workspace_id: i64) -> String;
}
```

#### 3.2.2 事件发布服务
```rust
// 🎯 简化的事件发布接口
pub struct ApplicationEventPublisher {
    nats_publisher: Option<Arc<NatsEventPublisher>>,
    local_publisher: Arc<LocalEventBus>,
}

impl ApplicationEventPublisher {
    // 异步事件发布
    pub async fn publish_async(&self, event: ApplicationEvent) -> Result<()>;
    pub async fn publish_domain_event(&self, event: DomainEvent) -> Result<()>;
    
    // 同步事件发布
    pub fn publish_sync(&self, event: ApplicationEvent) -> Result<()>;
    
    // 批量事件发布
    pub async fn publish_batch(&self, events: Vec<ApplicationEvent>) -> Result<()>;
}

// 应用事件类型
#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    User(UserEvent),
    Chat(ChatEvent),
    Message(MessageEvent),
    Notification(NotificationEvent),
}
```

---

## 🛠️ 4. Infrastructure Services 基础设施服务层 (`fechatter_server/src/services/infrastructure/`)

### 4.1 存储基础设施
**职责**: 数据持久化、存储抽象

| 存储服务 | 目录/文件 | 基础设施职责 | Handler透明化 |
|---------|----------|-------------|-------------|
| **关系数据库** | `storage/` | PostgreSQL数据持久化 | ✅ Handler通过Repository访问 |
| **缓存存储** | `cache/` | Redis缓存、内存缓存 | ✅ Handler通过CacheService访问 |
| **向量数据库** | `vector_db/` | AI向量存储、相似度搜索 | ✅ Handler通过VectorService访问 |
| **文件存储** | `storage/` | 文件上传、静态资源管理 | ✅ Handler通过StorageService访问 |

#### 4.1.1 缓存基础设施
```rust
// 🎯 缓存抽象接口
#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn expire(&self, key: &str, ttl: Duration) -> Result<()>;
}

// Redis缓存实现
pub struct RedisCacheService {
    client: redis::Client,
    connection_pool: Arc<Mutex<redis::Connection>>,
}

// 内存缓存实现
pub struct MemoryCacheService {
    cache: Arc<DashMap<String, (String, Instant)>>,
    default_ttl: Duration,
}
```

#### 4.1.2 向量数据库抽象
```rust
// 🎯 AI向量搜索抽象
#[async_trait]
pub trait VectorDatabase: Send + Sync {
    async fn store_embedding(&self, id: &str, embedding: Vec<f32>, metadata: HashMap<String, String>) -> Result<()>;
    async fn search_similar(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<VectorSearchResult>>;
    async fn delete_embedding(&self, id: &str) -> Result<()>;
}

// Pinecone实现
pub struct PineconeClient {
    client: reqwest::Client,
    api_key: String,
    environment: String,
    index_name: String,
}

// PgVector实现
pub struct PgVectorDatabase {
    pool: PgPool,
    table_name: String,
}
```

### 4.2 外部集成基础设施
**职责**: 第三方服务集成、API封装

| 集成服务 | 目录/文件 | 外部服务 | Handler解耦效果 |
|---------|----------|---------|---------------|
| **搜索服务** | `search/` | Meilisearch全文搜索 | ✅ Handler通过SearchService访问 |
| **通知推送** | `notification/` | WebSocket、邮件、短信 | ✅ Handler通过NotificationService |
| **消息队列** | `messaging/` | NATS、Redis Streams | ✅ Handler通过MessageQueue访问 |
| **AI服务** | `ai/` | OpenAI、Hugging Face等 | ✅ Handler通过AIService访问 |

#### 4.2.1 搜索服务抽象
```rust
// 🎯 全文搜索抽象
#[async_trait]
pub trait SearchService: Send + Sync {
    async fn index_document(&self, index: &str, id: &str, document: Value) -> Result<()>;
    async fn search(&self, index: &str, query: &SearchQuery) -> Result<SearchResults>;
    async fn delete_document(&self, index: &str, id: &str) -> Result<()>;
    async fn create_index(&self, index: &str, settings: IndexSettings) -> Result<()>;
}

pub struct MeilisearchClient {
    client: meilisearch_sdk::Client,
    host: String,
    api_key: Option<String>,
}
```

#### 4.2.2 AI服务抽象
```rust
// 🎯 AI服务统一接口
#[async_trait]
pub trait AIService: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
    async fn generate_text(&self, prompt: &str, options: GenerationOptions) -> Result<String>;
    async fn moderate_content(&self, content: &str) -> Result<ModerationResult>;
}

// OpenAI实现
pub struct OpenAIService {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

// Hugging Face实现
pub struct HuggingFaceService {
    client: reqwest::Client,
    api_key: String,
    model_endpoint: String,
}
```

### 4.3 事件基础设施
**职责**: 事件驱动架构支持

| 事件服务 | 目录/文件 | 事件职责 | 异步处理能力 |
|---------|----------|---------|-------------|
| **事件发布** | `event/` | NATS事件发布、订阅管理 | ✅ Handler触发后异步处理 |
| **消息传递** | `messaging/` | 实时消息传递、WebSocket | ✅ Handler专注业务逻辑 |

---

## 🚀 5. Service组合模式与Handler简化

### 5.1 Service注入模式
```rust
// ✅ Handler通过统一ServiceProvider访问所有服务
pub async fn create_message_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<ApiResponse<MessageResponse>, AppError> {
    
    // 1. 获取服务 - 通过统一provider
    let messaging_service = state.service_provider().messaging_service();
    let chat_service = state.service_provider().chat_service();
    
    // 2. 验证权限 - 通过domain service
    chat_service.validate_member_access(user.id.into(), chat_id.into()).await?;
    
    // 3. 执行业务逻辑 - 通过application service
    let message = messaging_service.send_message(
        user.id.into(),
        chat_id.into(), 
        payload.content
    ).await?;
    
    // 4. 构建响应 - 简单转换
    Ok(ApiResponse::success(MessageResponse::from(message)))
}
```

### 5.2 分层服务调用模式
```rust
// Handler Layer: 协调层
async fn create_chat_handler(...) -> Result<ChatResponse> {
    let chat_service = state.application_services().chat_service();
    let result = chat_service.create_chat(input).await?;
    Ok(ChatResponse::from(result))
}

// Application Service Layer: 用例编排层
impl ChatApplicationService {
    async fn create_chat(&self, input: CreateChatInput) -> Result<ChatView> {
        // 跨域协调
        let chat = self.chat_domain_service.create_chat(command).await?;
        self.notification_service.notify_chat_created(&chat).await?;
        self.cache_service.cache_chat(&chat).await;
        Ok(ChatView::from(chat))
    }
}

// Domain Service Layer: 业务逻辑层  
impl ChatDomainService {
    async fn create_chat(&self, command: CreateChatCommand) -> DomainResult<ChatAggregate> {
        // 业务规则验证
        self.validate_chat_creation(&command)?;
        // 聚合创建
        let chat = ChatAggregate::create(command)?;
        // 持久化
        self.repository.save(&chat).await?;
        // 领域事件
        self.publish_chat_created_event(&chat).await;
        Ok(chat)
    }
}

// Infrastructure Service Layer: 技术实现层
impl PostgresChatRepository {
    async fn save(&self, chat: &ChatAggregate) -> Result<()> {
        // 数据库操作
        sqlx::query!(...).execute(&self.pool).await?;
        Ok(())
    }
}
```

---

## 🎯 6. Service职责矩阵表

### 6.1 各层Service职责分工

| 服务层 | 主要职责 | Handler简化程度 | 可测试性 | 可复用性 |
|-------|---------|---------------|---------|---------|
| **Core Services** | 基础契约定义、Repository接口 | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Domain Services** | 业务逻辑封装、领域规则验证 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Application Services** | 用例编排、跨域协调 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Infrastructure Services** | 技术实现、外部集成 | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

### 6.2 业务场景Service选择指南

| 业务场景 | 推荐Service层 | 典型用法 | Handler复杂度 |
|---------|-------------|----------|-------------|
| **单一实体CRUD** | Domain Service | `user_domain_service.create_user()` | 极简 |
| **跨域业务用例** | Application Service | `chat_app_service.create_chat_with_notification()` | 简单 |
| **复杂业务流程** | Application Service + Domain Services | 组合调用多个服务 | 中等 |
| **外部系统集成** | Infrastructure Service | `ai_service.generate_embedding()` | 简单 |
| **实时事件处理** | Event Infrastructure + Application Services | 事件驱动架构 | 极简 |

---

## 🔧 7. Service实施最佳实践

### 7.1 依赖注入模式
```rust
// ✅ 统一的Service配置和注入
pub struct ServiceContainer {
    // Core Services
    auth_service: Arc<dyn AuthServiceTrait>,
    workspace_service: Arc<dyn WorkspaceServiceTrait>,
    
    // Domain Services  
    messaging_domain_service: Arc<MessageDomainService>,
    chat_domain_service: Arc<ChatDomainService>,
    user_domain_service: Arc<UserDomainService>,
    
    // Application Services
    user_app_service: Arc<dyn UserServiceTrait>,
    chat_app_service: Arc<dyn ChatServiceTrait>,
    auth_app_service: Arc<dyn AuthServiceTrait>,
    
    // Infrastructure Services
    cache_service: Arc<dyn Cache>,
    search_service: Arc<dyn SearchService>,
    ai_service: Arc<dyn AIService>,
}

impl ServiceContainer {
    pub fn new(config: &AppConfig, pool: PgPool) -> Self {
        // 按依赖顺序构建所有服务
        Self::build_services(config, pool)
    }
}
```

### 7.2 错误处理统一化
```rust
// ✅ 分层错误处理
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    
    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),
    
    #[error("Application error: {0}")]
    Application(String),
}

impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::Domain(domain_err) => AppError::BusinessRule(domain_err.to_string()),
            ServiceError::Infrastructure(infra_err) => AppError::ExternalService(infra_err.to_string()),
            ServiceError::Application(msg) => AppError::InvalidInput(msg),
        }
    }
}
```

### 7.3 缓存策略统一化
```rust
// ✅ Service层统一缓存策略
pub trait CacheableService {
    type Key: ToString;
    type Value: Serialize + DeserializeOwned;
    
    fn cache_key(&self, key: &Self::Key) -> String;
    fn cache_ttl(&self) -> Duration;
    
    async fn get_from_cache(&self, key: &Self::Key) -> Option<Self::Value> {
        let cache_key = self.cache_key(key);
        // 统一缓存逻辑
    }
    
    async fn set_to_cache(&self, key: &Self::Key, value: &Self::Value) {
        let cache_key = self.cache_key(key);
        let ttl = self.cache_ttl();
        // 统一缓存逻辑
    }
}
```

---

## 📈 8. 性能优化策略

### 8.1 Service层性能优化
- **并发处理**: 异步Service接口，支持高并发访问
- **缓存分层**: Core缓存基础数据，Application缓存业务视图
- **批量操作**: Repository层支持批量查询，减少数据库round-trip
- **连接池化**: 数据库、Redis、外部API连接池管理

### 8.2 内存管理优化
- **智能缓存**: 基于访问频率的LRU缓存策略
- **对象池**: 重用昂贵对象（如数据库连接、HTTP客户端）
- **流式处理**: 大数据集流式处理，避免内存溢出

---

## 🎯 总结：Service层的Handler支持价值

### 💪 Handler简化效果
1. **单一调用**: Handler仅需调用单个Application Service方法
2. **无业务逻辑**: Handler专注请求解析和响应构建
3. **统一错误处理**: Service层统一错误转换，Handler无需关心
4. **透明缓存**: Service层透明缓存管理，Handler无感知

### 🚀 开发效率提升
1. **快速开发**: 新Handler开发只需组合现有Services
2. **一致性**: 统一的Service接口和调用模式
3. **可测试性**: Service层单独测试，Handler测试简化
4. **可维护性**: 业务逻辑集中在Service层，易于修改

### 🏗️ 架构清晰度
1. **分层明确**: 每层Service有清晰的职责边界
2. **依赖清晰**: 自上而下的依赖关系，无循环依赖
3. **可扩展性**: 新业务需求通过扩展Service层实现
4. **技术解耦**: Infrastructure层封装技术实现细节

---

*这个Service目录清单为Fechatter项目提供了完整的业务逻辑支持架构，实现了Handler专注协调、API专注传输的理想分工模式。* 