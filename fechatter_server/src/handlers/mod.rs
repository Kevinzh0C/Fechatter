pub mod auth;
pub mod chat;
pub mod chat_member;
pub mod messages;
pub mod workspace;

use axum::response::IntoResponse;

pub use auth::*;
pub use chat::*;
pub use chat_member::*;
pub use messages::*;
pub use workspace::*;

pub async fn index_handler() -> impl IntoResponse {
  "index"
}
