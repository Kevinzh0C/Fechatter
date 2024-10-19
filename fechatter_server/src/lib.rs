mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use std::sync::Arc;
use std::{fmt, ops::Deref};

use axum::{
  Router,
  routing::{get, patch, post},
};
pub use config::AppConfig;
use dashmap::DashMap;
use sqlx::PgPool;
use tokio::time::Instant;
use utils::jwt::TokenManager;

pub use error::{AppError, ErrorOutput};
use handlers::*;
use middlewares::{SetAuthLayer, SetLayer};
pub use models::{ChatSidebar, CreateUser, SigninUser, User};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
  inner: Arc<AppStateInner>,
}

pub(crate) struct AppStateInner {
  pub(crate) config: AppConfig,
  pub(crate) token_manager: TokenManager,
  pub(crate) pool: PgPool,
  pub(crate) chat_list_cache: DashMap<i64, (Arc<Vec<ChatSidebar>>, Instant)>,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
  let state = AppState::try_new(config).await?;

  // Public routes - no authentication required
  let public_routes = Router::new()
    .route("/signin", post(signin_handler))
    .route("/signup", post(signup_handler))
    .with_state(state.clone());

  // Protected routes - authentication required
  let protected_routes = Router::new()
    .route("/chat", get(list_chats_handler).post(create_chat_handler).delete(delete_chat_handler))
    .route(
      "/chat/{id}",
      patch(update_chat_handler)
        .delete(delete_chat_handler)
        // .post(send_message_handler),
    )
    .route(
      "/chat/{id}/members",
      get(list_chat_members_handler)
        .post(add_chat_members_batch_handler)
        .delete(remove_chat_member_handler),
    )
    .route(
      "/chat/{id}/members/{member_id}",
      patch(transfer_chat_ownership_handler)
    )
    // .route("/chat/{id}/messages", get(list_message_handler))
    .set_auth_layer(state.clone())
    .with_state(state.clone());

  let api = Router::new().merge(public_routes).merge(protected_routes);

  // Create main app with all middleware
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
    let token_manager = TokenManager::from_config(&config.auth)?;
    let pool = create_pool(&config.server.db_url).await?;
    let chat_list_cache = DashMap::new();

    let state = AppStateInner {
      config,
      token_manager,
      pool,
      chat_list_cache,
    };

    Ok(Self {
      inner: Arc::new(state),
    })
  }
}

#[cfg(test)]
impl AppState {
  pub async fn test_new(config: AppConfig) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
    use sqlx_db_tester::TestPg;

    let token_manager = TokenManager::from_config(&config.auth)?;
    let post = config.server.db_url.rfind('/').expect("invalid db_url");
    let server_url = &config.server.db_url[..post];
    let tdb = TestPg::new(
      server_url.to_string(),
      std::path::Path::new("../migrations"),
    );

    let pool = tdb.get_pool().await;
    let chat_list_cache = DashMap::new();
    let state = AppStateInner {
      config,
      token_manager,
      pool,
      chat_list_cache,
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
