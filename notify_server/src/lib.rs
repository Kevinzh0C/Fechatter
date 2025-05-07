pub mod app_state;
mod sse;

use anyhow::Result;
use app_state::NotifyState;
use axum::{
  Router,
  response::{Html, IntoResponse},
  routing::get,
};
use fechatter_core::models::{Chat, ChatMember, Message};
use futures::StreamExt;
use sqlx::postgres::PgListener;
use sse::sse_handler;
use tracing::info;

const INDEX_HTML: &str = include_str!("../index.html");

pub enum Event {
  NewChat(Chat),
  AddToChat(ChatMember),
  RemoveFromChat(ChatMember),
  NewMessage(Message),
}


pub enum Event {
  NewChat(Chat),
  AddToChat(ChatMember),
  RemoveFromChat(ChatMember),
  NewMessage(Message),
}

pub fn get_router(state: NotifyState) -> Router {
  let state_clone = state.clone();
  Router::new()
    .route("/", get(index_handler))
    .route(
      "/events",
      get(move |req| sse_handler(state_clone.clone(), req)),
    )
    .with_state(state)
}

async fn index_handler() -> impl IntoResponse {
  Html(INDEX_HTML)
}

pub async fn set_up_pg_listener() -> Result<()> {
  let mut listener =
    PgListener::connect("postgres://postgres:postgres@localhost:5432/fechatter").await?;

  listener.listen("chat_updated").await?;
  listener.listen("message_created").await?;

  let mut stream = listener.into_stream();

  tokio::spawn(async move {
    while let Some(Ok(notification)) = stream.next().await {
      info!("event: {:?}", notification);
    }
  });

  Ok(())
}
