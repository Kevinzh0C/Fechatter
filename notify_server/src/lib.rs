mod config;
mod error;
mod nats_subscriber;
mod notify;
mod sse;

use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use axum::{
  Router,
  middleware::from_fn_with_state,
  response::{Html, IntoResponse},
  routing::get,
};
pub use config::AppConfig;
use dashmap::DashMap;
use error::NotifyError;
use fechatter_core::{
  ErrorMapper, TokenManager, TokenVerifier, UserClaims, middlewares::verify_token_middleware,
};

pub use notify::*;
use sse::sse_handler;
use tokio::sync::broadcast;

type UserMap = Arc<DashMap<i64, broadcast::Sender<Arc<NotifyEvent>>>>;

#[derive(Clone)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

#[derive(Clone)]
pub struct AppStateInner {
  config: AppConfig,
  users: UserMap,
  token_manager: TokenManager,
}

const INDEX_HTML: &str = include_str!("../index.html");

pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
  let state = AppState::new(config).expect("Failed to create app state");

  // Choose notification mechanism based on configuration
  if state.config.messaging.enabled {
    tracing::info!("Using NATS for messaging");
    nats_subscriber::setup_nats_subscriber(state.clone()).await?;
  } else {
    tracing::info!("Using PostgreSQL NOTIFY for messaging");
    notify::set_up_pg_listener(state.clone()).await?;
  }

  let app = Router::new()
    .route("/events", get(sse_handler))
    .layer(from_fn_with_state(
      state.clone(),
      verify_token_middleware::<AppState>,
    ))
    .route("/", get(index_handler))
    .with_state(state.clone());

  Ok(app)
}

async fn index_handler() -> impl IntoResponse {
  Html(INDEX_HTML)
}

impl TokenVerifier for AppState {
  type Claims = UserClaims;
  type Error = NotifyError;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    self
      .inner
      .token_manager
      .verify_token(token)
      .map_err(NotifyError::map_error)
  }
}

impl Deref for AppState {
  type Target = AppStateInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl AppState {
  pub fn new(config: AppConfig) -> Result<Self, anyhow::Error> {
    let users = Arc::new(DashMap::new());
    let token_manager = TokenManager::new(&config.auth)?;

    Ok(Self {
      inner: Arc::new(AppStateInner {
        config,
        users,
        token_manager,
      }),
    })
  }
}
