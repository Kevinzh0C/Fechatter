mod auth;
mod chat;
mod messages;

pub(crate) use auth::*;
use axum::response::IntoResponse;
pub(crate) use chat::*;
pub(crate) use messages::*;

pub(crate) async fn index_handler() -> impl IntoResponse {
  "index"
}
