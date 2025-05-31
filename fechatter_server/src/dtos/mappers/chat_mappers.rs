// TODO: CreateChatRequest module temporarily unavailable
// use crate::dtos::requests::chat::CreateChatRequest;
use fechatter_core::{Chat, CreateChat};

/// Chat mapper - handles DTO conversions for chats
///
/// Note: This mapper is transitioning to new DTOs architecture
/// New architecture uses core::Converter trait for type-safe conversions
pub struct ChatMapper;

impl ChatMapper {
  /// Deprecated - CreateChatRequest module temporarily unavailable
  #[deprecated(note = "Use new RequestDto::to_domain() method instead")]
  pub fn request_to_domain<T>(
    _request: &T,
    _members: Option<Vec<fechatter_core::UserId>>,
  ) -> Result<CreateChat, String> {
    Err(
      "CreateChatRequest module temporarily unavailable, please use new DTOs conversion framework"
        .to_string(),
    )
  }

  /// Convert domain Chat model to simple response structure
  /// (Note: Original ChatResponse doesn't exist, returning core Chat model here)
  pub fn domain_to_response(chat: &Chat) -> Chat {
    chat.clone()
  }
}
