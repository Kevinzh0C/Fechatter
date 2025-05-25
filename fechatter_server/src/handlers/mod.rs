pub mod auth;
pub mod chat;
pub mod chat_member;
pub mod messages;
pub mod workspace;

use axum::response::IntoResponse;

pub use messages::search_messages;

pub async fn index_handler() -> impl IntoResponse {
  "index"
}
