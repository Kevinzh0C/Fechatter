//! # Application Services Layer
//!
//! This layer is responsible for:
//! - Use Case Orchestration  
//! - Cross-Domain Service Coordination
//! - Transaction Boundary Management
//! - Cache Strategy Execution
//! - Authorization and Validation
//!
//! ## Architecture Principles
//! - Single Responsibility: Each application service focuses on one business domain
//! - Dependency Inversion: Depend on abstract interfaces, not concrete implementations
//! - Interface Segregation: Lean use case interfaces, avoid bloated services

// ============================================================================
// CORE APPLICATION SERVICES (实际存在的模块)
// ============================================================================

/// Authentication Application Service - 认证服务
pub mod auth_app_service;

/// Chat Application Service - 聊天领域服务
pub mod chat_app_service;

/// User Application Service - 用户管理服务
pub mod user_app_service;

/// Message Application Service - 消息领域服务 (新增)
pub mod message_app_service;

/// Notification Application Service - 通知服务
pub mod notification_app_service;

/// Workspace Application Service - 工作空间服务 (新增)
pub mod workspace_app_service;

/// Cache Strategy Service - 缓存策略服务
pub mod cache_strategy_service;

/// Application Event Publisher - 应用事件发布
pub mod application_event_publisher;

/// Indexer Sync Service - 索引同步服务
pub mod indexer_sync_service;

/// Message Stream Service - 消息流服务
pub mod message_stream;

/// Optimized Service Architecture - 优化的服务架构
pub mod optimized_service_architecture;

/// Search Application Service - 搜索应用服务
pub mod search_app_service;

// ============================================================================
// ADAPTER SERVICES - 适配器服务
// ============================================================================
// 适配器模式：将现有AppState方法桥接为新的Service接口

/// AppState适配器服务
pub mod adapters;

// ============================================================================
// PUBLIC EXPORTS - 公共导出
// ============================================================================

// Application Service Traits
pub use cache_strategy_service::CacheStrategyService;
pub use chat_app_service::{ChatDetailView, ChatServiceTrait, CreateChatInput};
pub use user_app_service::{
  ChangePasswordInput, UpdateUserInput, UserProfileView, UserServiceTrait,
};

// Message Application Service Exports (新增)
pub use message_app_service::{
  AppStateMessageServiceAdapter, MessageApplicationService, MessageApplicationServiceTrait,
  MessageView, create_message_application_service, create_message_service_adapter,
};

// Workspace Application Service Exports (新增)
pub use workspace_app_service::{
  AppStateWorkspaceServiceAdapter, InviteUserCommand, UpdateWorkspaceCommand, UserSummaryView,
  WorkspaceApplicationService, WorkspaceApplicationServiceTrait, WorkspaceView,
  create_workspace_application_service,
};

// Adapter Services and Traits - consolidated imports
pub use adapters::{
  AppStateChatServiceAdapter, AppStateNotificationServiceAdapter, AppStateUserServiceAdapter,
  NotificationServiceTrait,
};

// Re-export event publisher types
pub use application_event_publisher::{
  ApplicationEvent, ApplicationEventPublisher, ChatEvent, MessageEvent, NotificationEvent,
  UserEvent,
};

// Search Application Service Exports (新增)
pub use search_app_service::{
  MessageSearchResults, SearchApplicationService, SearchApplicationServiceTrait, SearchPage,
  SearchableMessage, create_search_application_service,
  create_search_application_service_from_config,
};

use crate::services::service_provider::ServiceProvider;
use fechatter_core::error::CoreError;
use std::sync::Arc;

// Simple no-op cache implementation for development
struct NoOpCache;

#[async_trait::async_trait]
impl fechatter_core::contracts::CacheService for NoOpCache {
  async fn get_bytes(&self, _key: &str) -> Result<Option<Vec<u8>>, CoreError> {
    Ok(None)
  }

  async fn set_bytes(&self, _key: &str, _value: Vec<u8>, _ttl: u64) -> Result<(), CoreError> {
    Ok(())
  }

  async fn delete(&self, _key: &str) -> Result<(), CoreError> {
    Ok(())
  }

  async fn exists(&self, _key: &str) -> Result<bool, CoreError> {
    Ok(false)
  }

  async fn delete_pattern(&self, _pattern: &str) -> Result<u64, CoreError> {
    Ok(0)
  }
}

/// Unified Application Service Provider - Aggregates all application services
pub struct ApplicationServiceProvider {
  service_provider: Arc<ServiceProvider>,
  /// Authentication service
  auth_service: Arc<auth_app_service::AuthService>,
  /// Chat service  
  chat_service: Arc<dyn ChatServiceTrait + Send + Sync>,
  /// Message service
  message_service: Arc<MessageApplicationService>,
  /// Search service
  search_service: Arc<SearchApplicationService>,
  /// Cache strategy service
  cache_service: Arc<CacheStrategyService>,
  /// Event publisher
  event_publisher: Arc<ApplicationEventPublisher>,
}

impl ApplicationServiceProvider {
  pub fn new(service_provider: Arc<ServiceProvider>) -> Result<Self, CoreError> {
    // Create no-op cache service
    let cache_service = Arc::new(CacheStrategyService::new(Arc::new(NoOpCache)));

    // Create event publisher
    let event_publisher = Arc::new(ApplicationEventPublisher::new());

    // Create user repository for auth service
    let user_repository = Arc::new(crate::domains::user::repository::UserRepositoryImpl::new(
      service_provider.pool().clone(),
    ));

    // Create token service for auth service
    let token_service: Box<dyn fechatter_core::TokenService + Send + Sync + 'static> =
      Box::new(crate::services::service_provider::ServerTokenService::new(
        service_provider.token_manager().clone(),
      ));

    // Create refresh token repository
    let refresh_token_repository: Box<
      dyn fechatter_core::models::jwt::RefreshTokenRepository + Send + Sync + 'static,
    > = Box::new(crate::domains::auth::RefreshTokenAdaptor::new(
      service_provider.pool().clone(),
    ));

    // Create auth service with correct parameters
    let auth_service = Arc::new(auth_app_service::AuthService::new(
      user_repository,
      token_service,
      refresh_token_repository,
      event_publisher.clone(),
    ));

    // Create chat repository using existing implementation
    let chat_repository = Arc::new(crate::domains::chat::repository::ChatRepositoryImpl::new(
      Arc::new(service_provider.pool().clone()),
    ));

    // Create chat service
    let chat_service: Arc<dyn ChatServiceTrait + Send + Sync> = Arc::new(ChatService::new(
      chat_repository,
      cache_service.clone(),
      event_publisher.clone(),
    ));

    // Create message repository using existing implementation
    let message_repository = Arc::new(
      crate::domains::messaging::repository::MessageRepositoryImpl::new(Arc::new(
        service_provider.pool().clone(),
      )),
    );

    // Create message domain service
    let message_domain_service =
      crate::domains::messaging::messaging_domain::MessageDomainServiceImpl::new(
        message_repository,
        crate::domains::messaging::messaging_domain::MessageConfig::default(),
      );

    // Create message stream service (placeholder)
    let message_stream_service =
      Arc::new(crate::services::application::message_stream::MessageStreamService::new());

    // Create message service with correct parameters
    let message_service = Arc::new(MessageApplicationService::new(
      message_domain_service,
      message_stream_service,
      event_publisher.clone(),
    ));

    // Create search service if available
    let search_service = if let Some(core_search_service) = service_provider.search_service() {
      Arc::new(SearchApplicationService::new(
        Arc::new(core_search_service.clone()), // Clone the search service
        event_publisher.clone(),
        search_app_service::SearchConfig::default(),
      ))
    } else {
      // Create a no-op search service if search is disabled
      Arc::new(SearchApplicationService::new(
        Arc::new(NoOpSearchService),
        event_publisher.clone(),
        search_app_service::SearchConfig {
          enabled: false,
          ..Default::default()
        },
      ))
    };

    Ok(Self {
      service_provider,
      auth_service,
      chat_service,
      message_service,
      search_service,
      cache_service,
      event_publisher,
    })
  }

  // Access methods
  pub fn auth_service(&self) -> Arc<auth_app_service::AuthService> {
    self.auth_service.clone()
  }

  pub fn chat_service(&self) -> Arc<dyn ChatServiceTrait + Send + Sync> {
    self.chat_service.clone()
  }

  pub fn message_service(&self) -> Arc<MessageApplicationService> {
    self.message_service.clone()
  }

  pub fn search_service(&self) -> Arc<SearchApplicationService> {
    self.search_service.clone()
  }

  pub fn cache_service(&self) -> Arc<CacheStrategyService> {
    self.cache_service.clone()
  }

  pub fn event_publisher(&self) -> Arc<ApplicationEventPublisher> {
    self.event_publisher.clone()
  }

  // Delegate high-level operations with proper implementations
  pub async fn create_user(
    &self,
    payload: &fechatter_core::CreateUser,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    self.auth_service.signup(payload, auth_context).await
  }

  pub async fn signin_user(
    &self,
    payload: &fechatter_core::SigninUser,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<Option<fechatter_core::AuthTokens>, fechatter_core::error::CoreError> {
    self.auth_service.signin(payload, auth_context).await
  }

  pub async fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    // Use the auth service's refresh token functionality
    self
      .auth_service
      .refresh_token(refresh_token, auth_context)
      .await
  }

  pub async fn logout_user(
    &self,
    refresh_token: &str,
  ) -> Result<(), fechatter_core::error::CoreError> {
    self.auth_service.logout(refresh_token).await
  }

  pub async fn logout_all_sessions(
    &self,
    user_id: fechatter_core::UserId,
  ) -> Result<(), fechatter_core::error::CoreError> {
    self.auth_service.logout_all(user_id).await
  }

  pub async fn create_new_chat(
    &self,
    chat_type: fechatter_core::ChatType,
    name: String,
    description: Option<String>,
    creator_id: fechatter_core::UserId,
    initial_members: Vec<fechatter_core::UserId>,
  ) -> Result<fechatter_core::Chat, crate::error::AppError> {
    // Clone initial_members to avoid move issues
    let members_for_chat = initial_members.clone();
    let members_for_metadata = initial_members;

    let input = CreateChatInput {
      chat_type,
      name: name.clone(),
      description,
      created_by: creator_id.into(),
      workspace_id: Some(1), // TODO: Get from user context
      initial_members: members_for_chat.into_iter().map(|id| id.into()).collect(),
      members: Some(
        members_for_metadata
          .into_iter()
          .map(|id| id.into())
          .collect(),
      ),
    };

    self.chat_service.create_chat(input).await
  }

  pub async fn create_message(
    &self,
    payload: fechatter_core::CreateMessage,
    chat_id: fechatter_core::ChatId,
    sender_id: fechatter_core::UserId,
  ) -> Result<fechatter_core::Message, fechatter_core::error::CoreError> {
    let message_view = self
      .message_service
      .send_message(sender_id, chat_id, payload)
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))?;

    // Convert MessageView back to Message
    Ok(fechatter_core::Message {
      id: fechatter_core::MessageId(message_view.id),
      chat_id,
      sender_id,
      content: message_view.content,
      files: message_view.files,
      created_at: message_view.created_at,
      idempotency_key: message_view.idempotency_key,
    })
  }

  pub async fn find_user_by_id(
    &self,
    user_id: fechatter_core::UserId,
  ) -> Result<Option<fechatter_core::User>, fechatter_core::error::CoreError> {
    self.auth_service.find_user_by_id(user_id).await
  }

  pub async fn generate_new_tokens_for_user(
    &self,
    user_id: i64,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    // Get user first
    let user = self
      .find_user_by_id(fechatter_core::UserId(user_id))
      .await?
      .ok_or_else(|| {
        fechatter_core::error::CoreError::NotFound(format!("User with id {} not found", user_id))
      })?;

    // Create UserClaims
    let user_claims = fechatter_core::models::jwt::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname,
      email: user.email,
      status: user.status,
      created_at: user.created_at,
    };

    // Generate tokens using the token service
    <fechatter_core::models::jwt::TokenManager as fechatter_core::TokenService>::generate_auth_tokens(
      self.service_provider.token_manager(),
      &user_claims,
      auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone()),
      auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone()),
    ).await
  }
}

// No-op search service for when search is disabled
struct NoOpSearchService;

#[async_trait::async_trait]
impl fechatter_core::contracts::SearchService for NoOpSearchService {
  async fn index_document(
    &self,
    _index: &str,
    _doc: fechatter_core::contracts::Document,
  ) -> Result<(), CoreError> {
    Ok(()) // No-op
  }

  async fn search(
    &self,
    _index: &str,
    _query: fechatter_core::contracts::SearchQuery,
  ) -> Result<fechatter_core::contracts::SearchResult, CoreError> {
    Ok(fechatter_core::contracts::SearchResult {
      hits: vec![],
      total: 0,
      took_ms: 0,
    })
  }

  async fn delete_document(&self, _index: &str, _id: &str) -> Result<(), CoreError> {
    Ok(()) // No-op
  }

  async fn update_document(
    &self,
    _index: &str,
    _id: &str,
    _doc: fechatter_core::contracts::Document,
  ) -> Result<(), CoreError> {
    Ok(()) // No-op
  }
}
