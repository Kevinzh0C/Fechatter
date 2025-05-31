pub mod config;
pub mod domains;
pub mod error;
pub mod handlers;
pub mod middlewares;
pub mod openapi;
pub mod services;
pub mod tests;
pub mod utils;
use std::{fmt, ops::Deref, sync::Arc};

use axum::{
  Router,
  routing::{get, patch, post},
};
pub use config::AppConfig;
use dashmap::DashMap;
use fechatter_core::chat::ChatSidebar;
use fechatter_core::error::CoreError;
use fechatter_core::models::jwt::TokenManager;
use fechatter_core::models::jwt::TokenService;
use fechatter_core::{
  contracts::UserRepository,
  middlewares::{
    ActualAuthServiceProvider, TokenVerifier as CoreTokenVerifier, WithServiceProvider,
    WithTokenManager as CoreWithTokenManager,
  },
  models::jwt::UserClaims,
};
use openapi::OpenApiRouter;
use sqlx::PgPool;
use tokio::fs;
use tokio::time::Instant;

use crate::error::{AppError, ErrorOutput};
use crate::services::EventPublisher;
use crate::services::application::{ApplicationServiceProvider, auth_app_service};

pub use error::{AppError as ErrorAppError, ErrorOutput as ErrorOutputType};

use handlers::*;
use middlewares::RouterExt;

use crate::handlers::{
  // Chat member handlers
  add_chat_members_batch_handler,
  // Chat handlers
  create_chat_handler,
  delete_chat_handler,
  // Message handlers
  file_handler,
  fix_file_storage_handler,
  get_search_suggestions,
  global_search_messages,
  // Health handlers
  health_check,
  // Workspace handlers
  list_all_workspace_users_handler,
  list_chat_members_handler,
  list_chats_handler,
  list_messages_handler,
  // Auth handlers
  logout_all_handler,
  logout_handler,
  refresh_token_handler,
  reindex_chat_messages,
  // Search handlers
  search_messages_in_chat,
  send_message_handler,
  signin_handler,
  signup_handler,
  simple_health_check,
  transfer_chat_ownership_handler,
  update_chat_handler,
  upload_handler,
};

use crate::handlers::chat_members::remove_chat_member_handler;

// Define the cache trait locally
#[allow(unused)]
trait WithCache<K, V> {
  fn get_from_cache(&self, key: &K) -> Option<V>;
  fn insert_into_cache(&self, key: K, value: V, ttl_seconds: u64);
  fn remove_from_cache(&self, key: &K);
}

// Define the pool trait locally
#[allow(unused)]
trait WithDbPool {
  fn db_pool(&self) -> &PgPool;
}

/// AppState - Focused on state management and service coordination only
#[derive(Debug, Clone)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
  pub(crate) config: AppConfig,
  pub(crate) service_provider: crate::services::service_provider::ServiceProvider,
  pub(crate) application_services: ApplicationServiceProvider,
  pub(crate) chat_list_cache: DashMap<i64, (Arc<Vec<ChatSidebar>>, Instant)>,
  pub(crate) event_publisher: Option<EventPublisher>,
}

impl CoreTokenVerifier for AppState {
  type Claims = UserClaims;
  type Error = CoreError;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    <TokenManager as fechatter_core::middlewares::TokenVerifier>::verify_token(
      self.inner.service_provider.token_manager(),
      token,
    )
  }
}

impl CoreWithTokenManager for AppState {
  type TokenManagerType = TokenManager;

  fn token_manager(&self) -> &Self::TokenManagerType {
    self.inner.service_provider.token_manager()
  }
}

impl WithServiceProvider for AppState {
  type ServiceProviderType = crate::services::service_provider::ServiceProvider;

  fn service_provider(&self) -> &Self::ServiceProviderType {
    &self.inner.service_provider
  }
}

impl WithDbPool for AppState {
  fn db_pool(&self) -> &PgPool {
    self.inner.service_provider.pool()
  }
}

// Implement ActualAuthServiceProvider for AppState
impl ActualAuthServiceProvider for AppState {
  type AuthService = auth_app_service::AuthService;

  fn create_service(&self) -> Self::AuthService {
    // Delegate to the application services
    self
      .inner
      .application_services
      .auth_service()
      .as_ref()
      .clone()
  }
}

impl WithCache<i64, (Arc<Vec<ChatSidebar>>, Instant)> for AppState {
  fn get_from_cache(&self, key: &i64) -> Option<(Arc<Vec<ChatSidebar>>, Instant)> {
    if let Some(entry) = self.inner.chat_list_cache.get(key) {
      let (chats, created_at) = &*entry;
      return Some((Arc::clone(chats), *created_at));
    }
    None
  }

  fn insert_into_cache(
    &self,
    key: i64,
    value: (Arc<Vec<ChatSidebar>>, Instant),
    _ttl_seconds: u64,
  ) {
    self.inner.chat_list_cache.insert(key, value);
  }

  fn remove_from_cache(&self, key: &i64) {
    self.inner.chat_list_cache.remove(key);
  }
}

// ============================================================================
// AppState - Clean Interface for State Management and Service Access
// ============================================================================

impl AppState {
  // ========================================================================
  // Infrastructure Access Methods
  // ========================================================================

  pub fn event_publisher(&self) -> Option<&EventPublisher> {
    self.inner.event_publisher.as_ref()
  }

  pub fn nats_client(&self) -> Option<&async_nats::Client> {
    self
      .inner
      .event_publisher
      .as_ref()
      .map(|ep| ep.nats_client())
  }

  pub fn search_service(&self) -> Option<&crate::services::SearchService> {
    self.inner.service_provider.search_service()
  }

  pub fn is_search_enabled(&self) -> bool {
    self
      .search_service()
      .map(|service| service.is_enabled())
      .unwrap_or(false)
  }

  #[inline]
  pub fn pool(&self) -> &PgPool {
    self.inner.service_provider.pool()
  }

  #[inline]
  pub fn token_manager(&self) -> &TokenManager {
    self.inner.service_provider.token_manager()
  }

  // ========================================================================
  // Application Service Access - Delegate to ApplicationServiceProvider
  // ========================================================================

  /// Get application services - All business logic is delegated here
  pub fn application_services(&self) -> &ApplicationServiceProvider {
    &self.inner.application_services
  }

  // ========================================================================
  // High-level Business Operations - Simple Delegation
  // ========================================================================

  /// Create user - Delegate to application services
  pub async fn create_user(
    &self,
    payload: &fechatter_core::CreateUser,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    self
      .application_services()
      .create_user(payload, auth_context)
      .await
  }

  /// Sign up user - Delegate to application services (alias for create_user)
  pub async fn signup(
    &self,
    payload: &fechatter_core::CreateUser,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    self
      .application_services()
      .create_user(payload, auth_context)
      .await
  }

  /// Sign in user - Delegate to application services
  pub async fn signin(
    &self,
    payload: &fechatter_core::SigninUser,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<Option<fechatter_core::AuthTokens>, fechatter_core::error::CoreError> {
    self
      .application_services()
      .signin_user(payload, auth_context)
      .await
  }

  /// Refresh token - Delegate to application services
  pub async fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    self
      .application_services()
      .refresh_token(refresh_token, auth_context)
      .await
  }

  /// Logout user - Delegate to application services
  pub async fn logout(&self, refresh_token: &str) -> Result<(), fechatter_core::error::CoreError> {
    self.application_services().logout_user(refresh_token).await
  }

  /// Logout all sessions - Delegate to application services
  pub async fn logout_all(
    &self,
    user_id: fechatter_core::UserId,
  ) -> Result<(), fechatter_core::error::CoreError> {
    self
      .application_services()
      .logout_all_sessions(user_id)
      .await
  }

  /// Create new chat - Delegate to application services
  pub async fn create_new_chat(
    &self,
    chat_type: fechatter_core::ChatType,
    name: Option<String>,
    description: Option<String>,
    creator_id: fechatter_core::UserId,
    initial_members: Vec<fechatter_core::UserId>,
  ) -> Result<fechatter_core::Chat, fechatter_core::error::CoreError> {
    self
      .application_services()
      .create_new_chat(
        chat_type,
        name.unwrap_or_default(),
        description,
        creator_id,
        initial_members,
      )
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
  }

  /// Create message - Delegate to application services
  pub async fn create_message(
    &self,
    payload: fechatter_core::CreateMessage,
    chat_id: fechatter_core::ChatId,
    sender_id: fechatter_core::UserId,
  ) -> Result<fechatter_core::Message, fechatter_core::error::CoreError> {
    self
      .application_services()
      .create_message(payload, chat_id, sender_id)
      .await
  }

  /// Find user by ID - Delegate to application services
  pub async fn find_user_by_id(
    &self,
    user_id: i64,
  ) -> Result<Option<fechatter_core::User>, AppError> {
    self
      .application_services()
      .find_user_by_id(fechatter_core::UserId(user_id))
      .await
      .map_err(|e| AppError::InvalidInput(format!("Failed to find user: {}", e)))
  }

  /// Generate new tokens for user - Delegate to application services
  pub async fn generate_new_tokens_for_user(
    &self,
    user_id: i64,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    self
      .application_services()
      .generate_new_tokens_for_user(user_id, auth_context)
      .await
  }

  // ========================================================================
  // Temporary/Legacy Methods - TODO: Move to appropriate services
  // ========================================================================

  /// Simple user access check - TODO: Move to authorization service
  pub async fn user_can_access_chat(&self, user_id: i64, chat_id: i64) -> Result<bool, AppError> {
    // TODO: Implement proper authorization check through authorization service
    let _ = (user_id, chat_id);
    Ok(true)
  }

  /// Delete chat by ID - TODO: Move to chat service
  pub async fn delete_chat_by_id(
    &self,
    chat_id: fechatter_core::ChatId,
    user_id: fechatter_core::UserId,
  ) -> Result<bool, fechatter_core::error::CoreError> {
    // TODO: Implement through chat service
    let _ = (chat_id, user_id);
    Ok(true)
  }

  /// Add chat members - TODO: Move to chat service
  pub async fn add_chat_members(
    &self,
    chat_id: fechatter_core::ChatId,
    user_id: fechatter_core::UserId,
    member_ids: Vec<fechatter_core::UserId>,
  ) -> Result<Vec<fechatter_core::ChatMember>, fechatter_core::error::CoreError> {
    // TODO: Implement through chat service
    let _ = (chat_id, user_id, member_ids);
    Ok(vec![])
  }

  /// Remove chat members - TODO: Move to chat service
  pub async fn remove_chat_members(
    &self,
    chat_id: fechatter_core::ChatId,
    user_id: fechatter_core::UserId,
    member_ids: Vec<fechatter_core::UserId>,
  ) -> Result<bool, fechatter_core::error::CoreError> {
    // TODO: Implement through chat service
    let _ = (chat_id, user_id, member_ids);
    Ok(true)
  }

  /// Transfer chat ownership - TODO: Move to chat service
  pub async fn transfer_chat_ownership(
    &self,
    chat_id: fechatter_core::ChatId,
    from_user_id: fechatter_core::UserId,
    to_user_id: fechatter_core::UserId,
  ) -> Result<bool, fechatter_core::error::CoreError> {
    // TODO: Implement through chat service
    let _ = (chat_id, from_user_id, to_user_id);
    Ok(true)
  }
}

impl Deref for AppState {
  type Target = AppStateInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl fmt::Debug for AppStateInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "AppStateInner {{ config: {:?}, token_manager: <hidden> }}",
      self.config
    )
  }
}

// Ensure service module and AuthService are exported
pub use crate::services::application::auth_app_service::AuthService;

// Re-export key handlers for external use
pub use handlers::*;

// Re-export domains for direct access (no models layer)
pub use domains::*;

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
  // Public routes - only need token refresh middleware
  let public_routes = Router::new()
    .route("/signin", post(signin_handler))
    .route("/signup", post(signup_handler))
    .route(
      "/refresh",
      post(|state, cookies, headers, auth_user| {
        refresh_token_handler(state, cookies, headers, auth_user)
      }),
    )
    .with_middlewares(state.clone())
    .build();

  // Basic auth routes - need Auth and Refresh middleware
  // Execution order: Auth -> Refresh -> Infrastructure middleware -> Handler
  let auth_routes = Router::new()
    .route("/upload", post(upload_handler))
    .route("/files/{ws_id}/{*path}", get(file_handler))
    .route("/fix-files/{ws_id}", post(fix_file_storage_handler))
    .route("/users", get(list_all_workspace_users_handler))
    .route("/logout", post(logout_handler))
    .route("/logout_all", post(logout_all_handler))
    .with_middlewares(state.clone())
    .with_auth_refresh() // Use helper method to add Auth and Refresh at once
    .build();

  // Search routes - need Auth and Refresh middleware
  let search_routes = Router::new()
    .route("/search/messages", get(global_search_messages))
    .route("/search/suggestions", get(get_search_suggestions))
    .route(
      "/chat/{chat_id}/messages/search",
      post(search_messages_in_chat),
    )
    .route("/admin/chat/{chat_id}/reindex", post(reindex_chat_messages))
    .with_middlewares(state.clone())
    .with_auth_refresh()
    .build();

  // Chat create routes - need Auth, Refresh and Workspace middleware
  // Execution order: Auth -> Refresh -> Workspace -> Infrastructure middleware -> Handler
  let chat_create_routes = Router::new()
    .route("/chat", post(create_chat_handler))
    .with_middlewares(state.clone())
    .with_auth_refresh_workspace() // Use helper method to add Auth, Refresh, and Workspace at once
    .build();

  // Chat management routes - need Auth, Refresh, Workspace and Chat middleware
  // Execution order: Auth -> Refresh -> Workspace -> Chat -> Infrastructure middleware -> Handler
  let chat_management_routes = Router::new()
    .route("/chat/{chat_id}/members", get(list_chat_members_handler))
    .route("/chat/{chat_id}/members", post(add_chat_members_batch_handler))
    .route("/chat/{chat_id}", patch(update_chat_handler))
    .route("/chat/{chat_id}", delete(delete_chat_handler))
    .route(
      "/chat/{chat_id}/transfer",
      post(transfer_chat_ownership_handler),
    )
    .route("/chats", get(list_chats_handler))
    .route("/chat/{chat_id}/messages", get(list_messages_handler))
    .route("/chat/{chat_id}/messages", post(send_message_handler))
    .with_middlewares(state.clone())
    .with_auth_refresh_workspace_chat() // Use helper method to add all middlewares
    .build();

  // Health check routes (no middleware needed)
  let health_routes = Router::new()
    .route("/health", get(health_check))
    .route("/health/simple", get(simple_health_check));

  // Root route
  let root_routes = Router::new().route("/", get(index_handler));

  // Combine all routes
  let app = Router::new()
    .merge(public_routes)
    .merge(auth_routes)
    .merge(search_routes)
    .merge(chat_create_routes)
    .merge(chat_management_routes)
    .merge(health_routes)
    .merge(root_routes)
    .with_state(state)
    .set_layer();

  Ok(app)
}

pub async fn create_pool(db_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
  use sqlx::postgres::PgPoolOptions;
  use std::time::Duration;

  PgPoolOptions::new()
    .max_connections(10)
    .min_connections(1)
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(60))
    .max_lifetime(Duration::from_secs(1800))
    .test_before_acquire(true)
    .connect(db_url)
    .await
}

impl AppState {
  #[cfg(test)]
  pub fn default_for_tests() -> Self {
    // Create minimal test configuration
    let config = AppConfig::minimal_dev_config().expect("Failed to create minimal dev config");

    // Create minimal service provider for tests
    let pool = sqlx::PgPool::connect_lazy("postgresql://test:test@localhost/test").unwrap();
    let refresh_token_repo = Arc::new(crate::domains::auth::RefreshTokenAdaptor::new(Arc::new(
      pool.clone(),
    )));
    let token_manager = TokenManager::from_config(&config.auth, refresh_token_repo).unwrap();
    let service_provider = crate::services::service_provider::ServiceProvider::new_with_search(
      pool,
      token_manager,
      None,
    );

    // Create application service provider
    let application_services =
      ApplicationServiceProvider::new(Arc::new(service_provider.clone())).unwrap();

    Self {
      inner: Arc::new(AppStateInner {
        config,
        service_provider,
        application_services,
        chat_list_cache: DashMap::new(),
        event_publisher: None,
      }),
    }
  }

  pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
    fs::create_dir_all(&config.server.base_dir).await?;

    // Create database connection pool
    let pool = create_pool(&config.server.db_url).await?;

    // Create refresh token adapter and token manager
    let refresh_token_repo = Arc::new(crate::domains::auth::RefreshTokenAdaptor::new(Arc::new(
      pool.clone(),
    )));
    let token_manager = TokenManager::from_config(&config.auth, refresh_token_repo)?;

    // Initialize search service if enabled - simplified
    let search_service = if config.search.enabled {
      tracing::info!("Search service enabled but initialization deferred to service layer");
      // TODO: Implement proper search service initialization in services module
      None
    } else {
      tracing::info!("Search service disabled");
      None
    };

    // Create service provider with search service
    let service_provider = crate::services::service_provider::ServiceProvider::new_with_search(
      pool,
      token_manager,
      search_service,
    );

    // Create application service provider
    let application_services = ApplicationServiceProvider::new(Arc::new(service_provider.clone()))
      .map_err(|e| AppError::Internal(format!("Failed to create application services: {}", e)))?;

    // Create chat list cache
    let chat_list_cache = DashMap::new();

    // Initialize NATS client and EventPublisher if enabled - simplified
    let event_publisher = if config.messaging.enabled {
      tracing::info!("Event publishing enabled but initialization deferred to service layer");
      // TODO: Implement proper event publisher initialization in services module
      None
    } else {
      tracing::info!("Event publishing disabled");
      None
    };

    // Create application state
    let state = AppStateInner {
      config,
      service_provider,
      application_services,
      chat_list_cache,
      event_publisher,
    };

    Ok(Self {
      inner: Arc::new(state),
    })
  }
}
