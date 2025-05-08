mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod tests;
mod utils;

mod services;
use std::sync::Arc;
use std::{fmt, ops::Deref};

use axum::{
  Router,
  routing::{get, patch, post},
};
pub use config::AppConfig;
use dashmap::DashMap;
use fechatter_core::chat::ChatSidebar;
use fechatter_core::error::CoreError;
use fechatter_core::models::jwt::TokenManager;
use fechatter_core::service_provider::ServiceProvider;
use fechatter_core::{
  SigninUser,
  middlewares::{
    ActualAuthServiceProvider, TokenVerifier as CoreTokenVerifier, WithServiceProvider,
    WithTokenManager as CoreWithTokenManager,
  },
  models::jwt::{LogoutService, RefreshTokenService, SigninService, SignupService, UserClaims},
};
use sqlx::PgPool;
use tokio::fs;
use tokio::time::Instant;
use utils::refresh_token::RefreshTokenAdaptor;

use crate::error::{AppError, ErrorOutput};

pub use error::{AppError as ErrorAppError, ErrorOutput as ErrorOutputType};
use fechatter_core::middlewares::custom_builder::RouterExt as CoreRouterExt;

use handlers::*;
use middlewares::RouterExt;

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
  pub(crate) service_provider: ServiceProvider,
  pub(crate) chat_list_cache: DashMap<i64, (Arc<Vec<ChatSidebar>>, Instant)>,
}

impl CoreTokenVerifier for AppState {
  type Claims = UserClaims;
  type Error = CoreError;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    self.inner.service_provider.verify_token(token)
  }
}

impl CoreWithTokenManager for AppState {
  type TokenManagerType = TokenManager;

  fn token_manager(&self) -> &Self::TokenManagerType {
    self.inner.service_provider.token_manager()
  }
}

impl WithServiceProvider for AppState {
  type ServiceProviderType = ServiceProvider;

  fn service_provider(&self) -> &Self::ServiceProviderType {
    &self.inner.service_provider
  }
}

impl WithDbPool for AppState {
  fn db_pool(&self) -> &PgPool {
    self.inner.service_provider.pool()
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

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
  let state = AppState::try_new(config).await?;

  // Public routes - no authentication required but apply token refresh
  // Must use direct approach since the builder enforces auth before refresh
  let public_routes = Router::new()
    .route("/signin", post(signin_handler))
    .route("/signup", post(signup_handler))
    .route(
      "/refresh",
      post(|state, cookies, headers, auth_user| {
        refresh_token_handler(state, cookies, headers, auth_user)
      }),
    )
    .with_refresh(state.clone());

  // Basic auth routes - requires authentication and token refresh
  let auth_routes = Router::new()
    .route("/upload", post(upload_handler))
    .route("/files/{ws_id}/{*path}", get(file_handler))
    .route("/fix-files/{ws_id}", post(fix_file_storage_handler))
    .route("/users", get(list_all_workspace_users_handler))
    .route("/logout", post(logout_handler))
    .route("/logout_all", post(logout_all_handler))
    .with_middlewares(state.clone())
    .with_auth()
    .with_token_refresh()
    .build();

  // Chat create routes - need auth, refresh, and workspace context
  let chat_create_routes = Router::new()
    .route("/chat", post(create_chat_handler))
    .route("/chat", get(list_chats_handler))
    .with_middlewares(state.clone())
    .with_auth()
    .with_token_refresh()
    .with_workspace()
    .build();

  // Chat manage routes - need auth, refresh, workspace, and chat membership
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
    .with_middlewares(state.clone())
    .with_auth()
    .with_token_refresh()
    .with_workspace()
    .with_chat_membership()
    .build();

  // Merge all routes
  let protected_api = Router::new()
    .merge(auth_routes)
    .merge(chat_create_routes)
    .merge(chat_manage_routes);

  let api = Router::new().merge(public_routes).merge(protected_api);

  let app = Router::new()
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

    // Create service provider - centrally manages pool and token_manager
    let service_provider = ServiceProvider::new(pool, token_manager);

    // Create chat list cache
    let chat_list_cache = DashMap::new();

    // Create application state
    let state = AppStateInner {
      config,
      service_provider,
      chat_list_cache,
    };

    Ok(Self {
      inner: Arc::new(state),
    })
  }

  #[cfg(test)]
  pub async fn test_new() -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
    use sqlx_db_tester::TestPg;

    let config = AppConfig::load().expect("Failed to load config");
    fs::create_dir_all(&config.server.base_dir)
      .await
      .map_err(|e| AppError::IOError(e))?;

    let post = config.server.db_url.rfind('/').expect("invalid db_url");
    let server_url = &config.server.db_url[..post];
    let tdb = TestPg::new(
      server_url.to_string(),
      std::path::Path::new("../migrations"),
    );

    // Create test database connection pool
    let pool = tdb.get_pool().await;

    // Create refresh token adapter and token manager
    let refresh_token_repo = Arc::new(RefreshTokenAdaptor::new(Arc::new(pool.clone())));
    let token_manager = TokenManager::from_config(&config.auth, refresh_token_repo)?;

    // Create service provider - centrally manages pool and token_manager
    let service_provider = ServiceProvider::new(pool, token_manager);

    // Create chat list cache
    let chat_list_cache = DashMap::new();

    // Create application state
    let state = AppStateInner {
      config,
      service_provider,
      chat_list_cache,
    };

    Ok((
      tdb,
      Self {
        inner: Arc::new(state),
      },
    ))
  }

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
    self
      .inner
      .service_provider
      .create_service()
      .signup(payload, auth_context)
      .await
  }

  pub async fn signin(
    &self,
    payload: &fechatter_core::SigninUser,
    auth_context: Option<fechatter_core::services::AuthContext>,
  ) -> Result<Option<fechatter_core::AuthTokens>, fechatter_core::error::CoreError> {
    self
      .inner
      .service_provider
      .create_service()
      .signin(payload, auth_context)
      .await
  }

  pub async fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<fechatter_core::services::AuthContext>,
  ) -> Result<fechatter_core::AuthTokens, fechatter_core::error::CoreError> {
    self
      .inner
      .service_provider
      .create_service()
      .refresh_token(refresh_token, auth_context)
      .await
  }

  pub async fn logout(&self, refresh_token: &str) -> Result<(), fechatter_core::error::CoreError> {
    self
      .inner
      .service_provider
      .create_service()
      .logout(refresh_token)
      .await
  }

  pub async fn logout_all(&self, user_id: i64) -> Result<(), fechatter_core::error::CoreError> {
    self
      .inner
      .service_provider
      .create_service()
      .logout_all(user_id)
      .await
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
