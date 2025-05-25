pub mod config;
pub mod error;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod openapi;
pub mod services;
pub mod tests;
pub mod utils;
use std::sync::Arc;
use std::{fmt, ops::Deref};

use crate::services::EventPublisher;
use axum::{
  Router,
  routing::{get, patch, post},
};
pub use config::AppConfig;
use dashmap::DashMap;
use fechatter_core::chat::ChatSidebar;
use fechatter_core::error::CoreError;
use fechatter_core::models::jwt::TokenManager;
use fechatter_core::{
  SigninUser,
  middlewares::{
    ActualAuthServiceProvider, TokenVerifier as CoreTokenVerifier, WithServiceProvider,
    WithTokenManager as CoreWithTokenManager,
  },
  models::jwt::{LogoutService, RefreshTokenService, SigninService, SignupService, UserClaims},
};
use openapi::OpenApiRouter;
use sqlx::PgPool;
use tokio::fs;
use tokio::time::Instant;
use utils::refresh_token::RefreshTokenAdaptor;

use crate::error::{AppError, ErrorOutput};

pub use error::{AppError as ErrorAppError, ErrorOutput as ErrorOutputType};
use fechatter_core::middlewares::custom_builder::RouterExt as CoreRouterExt;

use handlers::*;
use middlewares::RouterExt;

use crate::handlers::auth::{
  logout_all_handler, logout_handler, refresh_token_handler, signin_handler, signup_handler,
};
use crate::handlers::chat::{
  create_chat_handler, delete_chat_handler, list_chats_handler, update_chat_handler,
};
use crate::handlers::chat_member::{
  add_chat_members_batch_handler, list_chat_members_handler, remove_chat_member_handler,
  transfer_chat_ownership_handler,
};
use crate::handlers::messages::{
  file_handler, fix_file_storage_handler, list_messages_handler, send_message_handler,
  upload_handler,
};
use crate::handlers::search_messages;
use crate::handlers::workspace::list_all_workspace_users_handler;

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

#[derive(Debug, Clone)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
  pub(crate) config: AppConfig,
  pub(crate) service_provider: crate::services::service_provider::ServiceProvider,
  pub(crate) chat_list_cache: DashMap<i64, (Arc<Vec<ChatSidebar>>, Instant)>,
  pub(crate) event_publisher: Option<EventPublisher>,
}

impl CoreTokenVerifier for AppState {
  type Claims = UserClaims;
  type Error = CoreError;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    self
      .inner
      .service_provider
      .token_manager()
      .verify_token(token)
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

// Implement ActualAuthServiceProvider for AppState to replace the default implementation in core
impl ActualAuthServiceProvider for AppState {
  type AuthService = AuthService;

  fn create_service(&self) -> Self::AuthService {
    // Create service instance directly
    tracing::trace!("Creating new AuthService instance from AppState");

    let user_repository = Box::new(crate::models::user::FechatterUserRepository::new(Arc::new(
      self.inner.service_provider.pool().clone(),
    )));

    let token_service: Box<dyn fechatter_core::TokenService + Send + Sync + 'static> =
      Box::new(crate::services::ServerTokenService::new(Arc::new(
        self.inner.service_provider.token_manager().clone(),
      )));

    let refresh_token_repository = Box::new(crate::utils::refresh_token::RefreshTokenAdaptor::new(
      Arc::new(self.inner.service_provider.pool().clone()),
    ));

    // Create and return new AuthService instance
    // Although new instances are created each time, all components are shared through Arc
    AuthService::new(user_repository, token_service, refresh_token_repository)
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

// Add method to get EventPublisher
impl AppState {
  pub fn event_publisher(&self) -> Option<&EventPublisher> {
    self.inner.event_publisher.as_ref()
  }

  /// Get the search service if available
  pub fn search_service(&self) -> Option<&crate::services::SearchService> {
    self.inner.service_provider.search_service()
  }

  /// Check if user can access a specific chat
  pub async fn user_can_access_chat(&self, user_id: i64, chat_id: i64) -> Result<bool, AppError> {
    match self.ensure_user_is_chat_member(chat_id, user_id).await {
      Ok(_) => Ok(true),
      Err(AppError::ChatPermissionError(_)) => Ok(false),
      Err(AppError::NotFound(_)) => Ok(false),
      Err(e) => Err(e),
    }
  }

  /// Chat service functionality - delegate to existing methods
  pub fn chat_service(&self) -> &Self {
    self
  }

  /// Check if search service is available and enabled
  pub fn is_search_enabled(&self) -> bool {
    self
      .search_service()
      .map(|service| service.is_enabled())
      .unwrap_or(false)
  }
}

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

  // Chat create routes - need Auth, Refresh and Workspace middleware
  // Execution order: Auth -> Refresh -> Workspace -> Infrastructure middleware -> Handler
  let chat_create_routes = Router::new()
    .route("/chat", post(create_chat_handler))
    .route("/chat", get(list_chats_handler))
    .with_middlewares(state.clone())
    .with_auth_refresh_workspace() // Use helper method to add Auth, Refresh and Workspace at once
    .build();

  // Chat manage routes - need all business middleware
  // Execution order: Auth -> Refresh -> Workspace -> Chat Membership -> Infrastructure middleware -> Handler
  let chat_manage_routes = Router::new()
    .route(
      "/chat/{id}",
      patch(update_chat_handler).delete(delete_chat_handler),
    )
    .route(
      "/chat/{id}/members",
      get(list_chat_members_handler)
        .post(add_chat_members_batch_handler)
        .delete(remove_chat_member_handler),
    )
    .route(
      "/chat/{id}/members/{member_id}",
      patch(transfer_chat_ownership_handler),
    )
    .route(
      "/chat/{id}/messages",
      get(list_messages_handler).post(send_message_handler),
    )
    .route(
      "/chat/{id}/messages/search",
      post(search_messages),
    )
    .with_middlewares(state.clone())
    .with_all_middlewares() // Use helper method to add all business middleware at once
    .build();

  // Merge all routes
  let protected_api = Router::new()
    .merge(auth_routes)
    .merge(chat_create_routes)
    .merge(chat_manage_routes);

  let api = Router::new().merge(public_routes).merge(protected_api);

  let app = Router::new()
    .openapi()
    .route("/", get(index_handler))
    .nest("/api", api)
    .with_state(state);

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
  pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
    fs::create_dir_all(&config.server.base_dir).await?;

    // Create database connection pool
    let pool = create_pool(&config.server.db_url).await?;

    // Create refresh token adapter and token manager
    let refresh_token_repo = Arc::new(RefreshTokenAdaptor::new(Arc::new(pool.clone())));
    let token_manager = TokenManager::from_config(&config.auth, refresh_token_repo)?;

    // Initialize search service if enabled
    let search_service = if config.search.enabled {
      tracing::info!("Initializing search service");
      match crate::services::SearchService::new(config.search.clone()) {
        Ok(service) => {
          // Initialize indexes
          if let Err(e) = service.initialize_indexes().await {
            tracing::error!("Failed to initialize search indexes: {}", e);
            return Err(AppError::SearchError(format!(
              "Search index initialization failed: {}",
              e
            )));
          }
          tracing::info!("Search service initialized successfully");
          Some(service)
        }
        Err(e) => {
          tracing::error!("Failed to initialize search service: {}", e);
          return Err(AppError::SearchError(format!(
            "Search service initialization failed: {}",
            e
          )));
        }
      }
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

    // Create chat list cache
    let chat_list_cache = DashMap::new();

    // Initialize NATS client and EventPublisher if enabled
    let event_publisher = if config.messaging.enabled {
      tracing::info!("Initializing NATS client for event publishing");
      match async_nats::connect(&config.messaging.nats.url).await {
        Ok(nats_client) => {
          let publisher = EventPublisher::new(nats_client, config.messaging.nats.subjects.clone());
          tracing::info!("NATS event publisher initialized successfully");
          Some(publisher)
        }
        Err(e) => {
          tracing::error!("Failed to connect to NATS: {}", e);
          return Err(AppError::NatsError(format!(
            "NATS connection failed: {}",
            e
          )));
        }
      }
    } else {
      tracing::info!("NATS messaging disabled, using PostgreSQL triggers");
      None
    };

    // Create application state
    let state = AppStateInner {
      config,
      service_provider,
      chat_list_cache,
      event_publisher,
    };

    Ok(Self {
      inner: Arc::new(state),
    })
  }
}

impl AppState {
  #[inline]
  pub fn pool(&self) -> &PgPool {
    self.inner.service_provider.pool()
  }

  #[inline]
  pub fn token_manager(&self) -> &TokenManager {
    self.inner.service_provider.token_manager()
  }

  pub async fn signup(
    &self,
    payload: &fechatter_core::CreateUser,
    auth_context: Option<fechatter_core::services::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    <Self as ActualAuthServiceProvider>::create_service(self)
      .signup(payload, auth_context)
      .await
  }

  pub async fn signin(
    &self,
    payload: &fechatter_core::SigninUser,
    auth_context: Option<fechatter_core::services::AuthContext>,
  ) -> Result<Option<fechatter_core::AuthTokens>, fechatter_core::error::CoreError> {
    <Self as ActualAuthServiceProvider>::create_service(self)
      .signin(payload, auth_context)
      .await
  }

  pub async fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<fechatter_core::services::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    <Self as ActualAuthServiceProvider>::create_service(self)
      .refresh_token(refresh_token, auth_context)
      .await
  }

  pub async fn logout(&self, refresh_token: &str) -> Result<(), fechatter_core::error::CoreError> {
    <Self as ActualAuthServiceProvider>::create_service(self)
      .logout(refresh_token)
      .await
  }

  pub async fn logout_all(&self, user_id: i64) -> Result<(), fechatter_core::error::CoreError> {
    <Self as ActualAuthServiceProvider>::create_service(self)
      .logout_all(user_id)
      .await
  }

  pub async fn generate_new_tokens_for_user(
    &self,
    user_id: i64,
    auth_context: Option<fechatter_core::services::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    // Get the user from database
    let user = self
      .find_user_by_id(user_id)
      .await
      .map_err(|e| {
        fechatter_core::error::CoreError::Internal(format!("Failed to find user: {}", e))
      })?
      .ok_or_else(|| {
        fechatter_core::error::CoreError::NotFound(format!("User with id {} not found", user_id))
      })?;

    // Create UserClaims from the user
    let user_claims = fechatter_core::models::jwt::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname,
      email: user.email,
      status: user.status,
      created_at: user.created_at,
    };

    // Generate new tokens using the token manager directly with fully qualified syntax
    // This calls generate_auth_tokens on TokenManager through the TokenService trait
    <fechatter_core::models::jwt::TokenManager as fechatter_core::TokenService>::generate_auth_tokens(
      self.token_manager(),
      &user_claims,
      auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone()),
      auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone()),
    ).await
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
pub use crate::services::AuthService;
