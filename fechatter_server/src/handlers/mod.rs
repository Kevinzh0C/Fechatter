pub mod auth;
mod chat;
mod chat_member;
mod health;
pub mod messages;
mod workspace;

pub use auth::*;
pub use chat::*;
pub use chat_member::*;
pub use health::*;
pub use messages::*;
pub use workspace::*;

use axum::response::IntoResponse;

pub use messages::search_messages;

pub async fn index_handler() -> impl IntoResponse {
  "index"
}
