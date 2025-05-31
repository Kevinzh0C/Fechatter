use crate::api::v1::messages::MessageResponse;
// TODO: SendMessageRequest should be re-exported or moved to correct location
// use crate::dtos::models::requests::message::SendMessageRequest;
use fechatter_core::models::Message;

/// Message mapper - handles DTO conversions for messages
///
/// Note: This mapper is transitioning to new DTOs architecture
/// New architecture uses core::Converter trait for type-safe conversions
pub struct MessageMapper;

impl MessageMapper {
  /// Deprecated - SendMessageRequest module temporarily unavailable
  #[deprecated(note = "Use new RequestDto::to_domain() method instead")]
  pub fn request_to_domain<T>(
    _request: &T,
  ) -> Result<fechatter_core::models::CreateMessage, String> {
    Err(
      "SendMessageRequest module temporarily unavailable, please use new DTOs conversion framework"
        .to_string(),
    )
  }

  /// Convert domain Message model to MessageResponse DTO
  pub fn domain_to_response(message: &Message) -> MessageResponse {
    MessageResponse {
      id: message.id.into(),
      chat_id: message.chat_id.into(),
      sender_id: message.sender_id.into(),
      content: message.content.clone(),
      files: message.files.clone(),
      created_at: message.created_at,
      reply_to: message.reply_to.map(|id| id.into()),
      mentions: message
        .mentions
        .as_ref()
        .map(|mentions| mentions.iter().map(|user_id| (*user_id).into()).collect()),
    }
  }
}
