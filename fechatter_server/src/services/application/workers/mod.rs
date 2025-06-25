//! # Workers Module - 工作者
//!
//! **职责**: 执行具体业务逻辑的工作者
//! **特点**: 专业化、高效执行、各司其职

pub mod auth;
pub mod chat;
pub mod message;
pub mod profile;
pub mod search;
pub mod workspace;

// 重新导出工作者 - 简化版本，无traits
pub use auth::AuthUserService;
pub use chat::{ChatDetailView, ChatService, CreateChatInput};
pub use message::{MessageApplicationService, MessageView};
pub use search::{MessageSearchResults, SearchApplicationService};
pub use workspace::{
  InviteUserCommand, UpdateWorkspaceCommand, UserSummaryView, WorkspaceService, WorkspaceView,
};
