// DTO models and structures
pub mod message_models;
pub mod requests;
pub mod responses;

// Re-export core types from fechatter_core
pub use fechatter_core::{AuthUser, CreateUser};

// Re-export message models
pub use message_models::{GetMessagesInput, MessageView, SendMessageInput};
