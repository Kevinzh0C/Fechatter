pub mod auth;
pub mod cache_stats;
pub mod chat;
pub mod chat_members;
pub mod files;
pub mod health;
pub mod login_monitor;
pub mod messages;
pub mod production_health;
pub mod realtime;
pub mod search;
pub mod users;
pub mod workspaces;

// Re-export all handlers for convenience
pub use auth::*;
pub use chat::*;
pub use chat_members::*;
pub use files::*;
pub use health::*;
pub use login_monitor::*;
pub use messages::*;
pub use realtime::*;
pub use search::*;
pub use users::*;
pub use workspaces::*;

use axum::response::IntoResponse;

pub async fn index_handler() -> impl IntoResponse {
  "index"
}
