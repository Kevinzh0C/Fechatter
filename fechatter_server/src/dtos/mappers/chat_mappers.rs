// TODO: CreateChatRequest module temporarily unavailable
// use crate::dtos::requests::chat::CreateChatRequest;
use crate::dtos::models::responses::chat::ChatDetailDto;
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

  /// Convert domain Chat model to response DTO
  pub fn domain_to_response(chat: &Chat) -> ChatDetailDto {
    ChatDetailDto {
      id: chat.id.into(),
      name: chat.name.clone(),
      chat_type: chat.chat_type,
      description: Some(chat.description.clone()),
      created_by: chat.created_by.into(),
      workspace_id: Some(chat.workspace_id.into()),
      member_count: chat.chat_members.len() as i32,
      created_at: chat.created_at,
      updated_at: Some(chat.updated_at),
      is_archived: false, // Default value
      is_public: match chat.chat_type {
        fechatter_core::ChatType::PublicChannel => true,
        _ => false,
      },
    }
  }
}
