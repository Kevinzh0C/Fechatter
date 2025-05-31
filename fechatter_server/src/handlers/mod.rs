pub mod auth;
pub mod chat;
pub mod chat_members;
pub mod files;
pub mod health;
pub mod messages;
pub mod search;
pub mod users;
pub mod workspaces;

// Re-export all handlers for convenience
pub use auth::*;
pub use chat::*;
pub use chat_members::*;
pub use files::*;
pub use health::*;
pub use messages::*;
pub use search::*;
pub use users::*;
pub use workspaces::*;

use axum::response::IntoResponse;

pub async fn index_handler() -> impl IntoResponse {
  "index"
}
