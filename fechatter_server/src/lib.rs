pub mod config;
pub mod error;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod services;
pub mod utils;
pub mod tests;

use std::sync::Arc;
use std::{fmt, ops::Deref};

use anyhow::Context as _;
use axum::{
  Router,
  routing::{get, patch, post},
};
pub use config::AppConfig;
use dashmap::DashMap;
use services::ServiceProvider;
use sqlx::PgPool;
use tokio::fs;
use tokio::time::Instant;
pub use utils::jwt::TokenManager;

pub use error::{AppError, ErrorOutput};
use handlers::*;
pub use middlewares::{RouterExt, SetLayer, WorkspaceContext};
pub use models::{ChatSidebar, ChatUser, CreateUser, SigninUser, User, UserStatus, Workspace};
pub use services::{AuthServiceTrait, auth_service::AuthService};
pub use utils::*;

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

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
  let state = AppState::try_new(config).await?;

  // Public routes - no authentication required but apply token refresh
  let public_routes = Router::new()
    .route("/signin", post(signin_handler))
    .route("/signup", post(signup_handler))
    .route(
      "/refresh",
      post(|state, cookies, headers| refresh_token_handler(state, cookies, headers)),
    )
    .with_middlewares(state.clone())
    .with_token_refresh()
    .build();

  // Protected routes - authentication required
  let protected_routes = Router::new()
    .route("/upload", post(upload_handler))
    .route("/files/{ws_id}/{*path}", get(file_handler))
    .route("/fix-files/{ws_id}", post(fix_file_storage_handler))
    .route("/users", get(list_all_workspace_users_handler))
    .route(
      "/logout",
      post(|state, cookies, headers, auth_user| logout_handler(state, cookies, headers, auth_user)),
    )
    .route(
      "/logout_all",
      post(|state, cookies, headers, auth_user| {
        logout_all_handler(state, cookies, headers, auth_user)
      }),
    )
    .route(
      "/chat",
      get(list_chats_handler)
        .post(create_chat_handler)
        .delete(delete_chat_handler),
    )
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
    .with_token_refresh()
    .with_auth()
    .build();

  // Routes using workspace middleware
  let workspace_routes = Router::new()
    .route(
      "/workspace/users",
      get(list_workspace_users_with_middleware),
    )
    .with_middlewares(state.clone())
    .with_token_refresh()
    .with_auth()
    .with_workspace()
    .build();

  let api = Router::new()
    .merge(public_routes)
    .merge(protected_routes)
    .merge(workspace_routes);

  let app = Router::new()
    .route("/", get(index_handler))
    .nest("/api", api)
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
  pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
    fs::create_dir_all(&config.server.base_dir)
      .await
      .context("failed to create base dir")?;
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
      .context("failed to create base dir")?;
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
