pub mod contracts;
pub mod error;
pub mod middlewares;
pub mod models;
pub mod services;
pub mod utils;

// Re-export core types and traits
pub use contracts::{
  AIService, AuthContext, CacheService, ChatMessage, Document, EventService, SearchQuery,
  SearchResult, SearchService, Sentiment,
};
pub use error::{CoreError, ErrorMapper};
pub use middlewares::{
  ActualAuthServiceProvider, SetLayer, TokenVerifier, WithServiceProvider, WithTokenManager,
};
pub use models::user::AuthUser;
pub use models::{
  AuthTokens,
  // Chat related models
  Chat,
  ChatId,
  ChatMember,
  ChatType,
  ChatUser,
  CreateChat,
  // Message related models
  CreateMessage,
  CreateUser,
  ListMessages,
  Message,
  MessageId,
  SearchMessages,
  SearchableMessage,
  SigninUser,
  UpdateChat,
  User,
  UserClaims,
  UserId,
  UserStatus,
  Workspace,
  WorkspaceId,
};
pub use services::AuthService;

// Re-export time management
pub use models::time_management::TimeManager;

// Re-export vector database types
pub use models::vector_db::{
  MessageChunk, MessageEmbedding, MessageVectorRepository, MetadataFilter, VectorDatabase,
  VectorQuery, VectorSearchResult,
};

// Re-export JWT and auth related types for backward compatibility
pub use models::jwt;
pub use models::jwt::{
  LogoutService, RefreshTokenService, SigninService, SignupService, TokenManager, TokenService,
};

// Chat module re-exports
pub mod chat {
  pub use crate::models::{Chat, ChatMember, ChatType};

  // ChatSidebar type for UI
  use crate::models::{ChatId, UserId};
  use chrono::{DateTime, Utc};
  use serde::{Deserialize, Serialize};
  use utoipa::ToSchema;

  #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
  pub struct ChatSidebar {
    pub id: ChatId,
    pub name: String,
    pub chat_type: ChatType,
    pub last_message: Option<String>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub unread_count: i32,
    pub members_count: i32,
    pub created_by: UserId,
  }
}
