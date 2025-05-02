mod config;
mod error;
mod handlers;

mod models;

mod test_utils;

use std::sync::Arc;
use std::{fmt, ops::Deref};

use axum::{
  Router,
  routing::{get, patch, post},
};
pub use config::AppConfig;
use dashmap::DashMap;
use fechatter_core::utils::jwt::{TokenManager, UserClaims};
use sqlx::PgPool;
use tokio::fs;
use tokio::time::Instant;

use crate::error::{AppError, ErrorOutput};
pub use error::{AppError as ErrorAppError, ErrorOutput as ErrorOutputType};
use fechatter_core::models::chat::ChatSidebar;
pub use fechatter_core::{CreateUser, SigninUser, User};
use handlers::*;

// Define the trait locally since it's not in fechatter_core
#[allow(unused)]
trait TokenVerifier {
  type Error;
  type Claims;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error>;
}

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

// Define the token manager trait locally
#[allow(unused)]
trait WithTokenManager {
  fn token_manager(&self) -> &TokenManager;
}
use services::ServiceProvider;
=======
pub use middlewares::{RouterExt, SetAuthLayer, SetLayer, WorkspaceContext};
pub use models::{ChatSidebar, ChatUser, CreateUser, SigninUser, User, UserStatus, Workspace};
pub use services::{AuthServiceTrait, auth_service::AuthService};
pub use utils::*;

use middlewares::{RouterExt, SetLayer};
pub use models::{ChatSidebar, CreateUser, SigninUser, User};
use services::ServiceProvider;

#[derive(Debug, Clone)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
  pub(crate) config: AppConfig,
  pub(crate) token_manager: TokenManager,
  pub(crate) pool: PgPool,
  pub(crate) chat_list_cache: DashMap<i64, (Arc<Vec<ChatSidebar>>, Instant)>,
  pub(crate) service_provider: ServiceProvider,
}

impl TokenVerifier for AppState {
  type Error = AppError;
  type Claims = UserClaims;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    self
      .inner
      .token_manager
      .verify_token(token)
      .map_err(AppError::from)
  }
}

impl WithDbPool for AppState {
  fn db_pool(&self) -> &PgPool {
    &self.inner.pool
  }
}

impl WithTokenManager for AppState {
  fn token_manager(&self) -> &TokenManager {
    &self.inner.token_manager
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

impl TokenVerifier for AppState {
  type Error = AppError;
  type Claims = UserClaims;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    self
      .inner
      .token_manager
      .verify_token(token)
      .map_err(AppError::from)
  }
}

impl WithDbPool for AppState {
  fn db_pool(&self) -> &PgPool {
    &self.inner.pool
  }
}

impl WithTokenManager for AppState {
  fn token_manager(&self) -> &TokenManager {
    &self.inner.token_manager
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
  let public_routes = Router::new()
    .route("/signin", post(signin_handler))
    .route("/signup", post(signup_handler))
    .route(
      "/refresh",
      post(|state, cookies, headers, auth_user| {
        refresh_token_handler(state, cookies, headers, auth_user)
      }),
    )
    .with_token_refresh(&state);

  // Basic auth routes - only requires authentication
  let auth_routes = Router::new()
    .route("/upload", post(upload_handler))
    .route("/files/{ws_id}/{*path}", get(file_handler))
    .route("/fix-files/{ws_id}", post(fix_file_storage_handler))
    .route("/users", get(list_all_workspace_users_handler))
    .route("/logout", post(logout_handler))
    .route("/logout_all", post(logout_all_handler))
    .with_token_refresh(&state)
    .with_auth(&state);

  // Chat create routes - need workspace context
  let chat_create_routes = Router::new()
    .route("/chat", post(create_chat_handler))
    .route("/chat", get(list_chats_handler))
    .with_workspace(&state)
    .with_auth(&state)
    .with_token_refresh(&state);

  // Chat manage routes - need chat membership verification
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
    .with_state(state.clone())
    .with_state(state.clone());
    .with_middlewares(state.clone())
    .with_token_refresh()
    .with_auth()
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
<<<<<<< HEAD
    .with_state(state);
=======
    .layer(axum::extract::Extension(state))
    .set_layer();
>>>>>>> eef9927 (fix: resolve linting and type errors)

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
    let token_manager = TokenManager::from_config(&config.auth)?;
    let pool = create_pool(&config.server.db_url).await?;
    let chat_list_cache = DashMap::new();
    let service_provider = ServiceProvider::new(pool.clone(), token_manager.clone());

    let state = AppStateInner {
      config,
      token_manager,
      pool,
      chat_list_cache,
      service_provider,
    };

    Ok(Self {
      inner: Arc::new(state),
    })
  }
}

#[cfg(test)]
impl AppState {
  pub async fn test_new() -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
    use sqlx_db_tester::TestPg;

    let config = AppConfig::load().expect("Failed to load config");
    fs::create_dir_all(&config.server.base_dir)
      .await
      .map_err(|e| AppError::IOError(e))?;
    let token_manager = TokenManager::from_config(&config.auth)?;
    let post = config.server.db_url.rfind('/').expect("invalid db_url");
    let server_url = &config.server.db_url[..post];
    let tdb = TestPg::new(
      server_url.to_string(),
      std::path::Path::new("../migrations"),
    );

    let pool = tdb.get_pool().await;
    let chat_list_cache = DashMap::new();
    let service_provider = ServiceProvider::new(pool.clone(), token_manager.clone());
    let state = AppStateInner {
      config,
      token_manager,
      pool,
      chat_list_cache,
      service_provider,
    };

    Ok((
      tdb,
      Self {
        inner: Arc::new(state),
      },
    ))
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
