mod chat;
mod chat_member;
mod file;
mod message;
mod user;
mod workspace;

use serde::{Deserialize, Serialize};

pub use chat::*;
pub use chat_member::*;
pub use message::*;
pub use user::{AuthUser, CreateUser, SigninUser};

pub use fechatter_core::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
  pub workspace_id: i64,
  pub ext: String, // extract from the uploaded filename
  pub hash: String,
}
