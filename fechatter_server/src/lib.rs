mod config;
mod error;
mod handlers;
mod models;
mod utils;

use std::sync::Arc;
use std::{fmt, ops::Deref};

use anyhow::Context;
use axum::{
  Router,
  routing::{get, patch, post},
};
pub use config::AppConfig;
pub use error::{AppError, ErrorOutput};
use handlers::*;
pub use models::{CreateUser, SigninUser, User};
use sqlx::PgPool;
use utils::jwt::TokenManager;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
  inner: Arc<AppStateInner>,
}

pub(crate) struct AppStateInner {
  pub(crate) config: AppConfig,
  pub(crate) token_manager: TokenManager,
  pub(crate) pool: PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
  let state = AppState::try_new(config).await?;

  let api = Router::new()
    .route("/signin", post(signin_handler))
    .route("/signup", post(signup_handler))
    .route("/chat", get(list_chat_handler).post(create_chat_handler))
    .route(
      "/chat/{id}",
      patch(update_chat_handler)
        .delete(delete_chat_handler)
        .post(send_message_handler),
    )
    .route("/chat/{id}/messages", get(list_message_handler));

  let app = Router::new()
    .route("/", get(index_handler))
    .nest("/api", api)
    .with_state(state);

  Ok(app)
}

impl AppState {
  pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
    let token_manager = TokenManager::from_config(&config.auth)?;
    let pool = PgPool::connect(&config.server.db_url)
      .await
      .context("connect to db failed")?;

    let state = AppStateInner {
      config,
      token_manager,
      pool,
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
    let tdb = TestPg::new(
      config.server.db_url.to_string(),
      std::path::Path::new("./migrations"),
    );

    let pool = tdb.get_pool().await;

    let state = AppStateInner {
      config,
      token_manager,
      pool,
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
