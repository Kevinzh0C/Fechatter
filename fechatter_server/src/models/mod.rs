pub mod chat;
pub mod chat_member;
pub mod file;
pub mod message;
pub mod user;
pub mod workspace;

use serde::{Deserialize, Serialize};

pub use chat::ChatSidebar as ServerChatSidebar;
pub use chat_member::CreateChatMember as ServerCreateChatMember;
pub use message::{ServerCreateMessage, ServerListMessage};
pub use chat::{CreateChat,UpdateChat};
pub use chat_member::*;
pub use message::*;

pub use fechatter_core::{
  AuthUser, Chat, ChatMember, ChatType, CreateUser, Message, SigninUser, User, UserStatus,
  Workspace,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
  pub workspace_id: i64,
  pub ext: String, // extract from the uploaded filename
  pub hash: String,
}
