use crate::{AppError, AppState, error::ErrorOutput, models::AuthUser};
use axum::{
  extract::{Extension, Path, Query, State},
  http::StatusCode,
  response::{IntoResponse, Json},
};
use fechatter_core::{CreateMessage, ListMessages, Message};
use tracing::info;

/// 发送消息
#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/messages",
    params(
        ("chat_id" = i64, Path, description = "Chat ID")
    ),
    request_body = CreateMessage,
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 201, description = "Message sent successfully", body = Message),
        (status = 400, description = "Invalid input", body = ErrorOutput),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat not found", body = ErrorOutput)
    ),
    tag = "messages"
)]
pub async fn send_message_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(message): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
  info!(
    "User {} sending message to chat {} with idempotency_key {}",
    user.id, chat_id, message.idempotency_key
  );

  // Convert core CreateMessage to server CreateMessage
  let message = state
    .create_message(message, chat_id.into(), user.id.into())
    .await?;

  Ok((StatusCode::CREATED, Json(message)))
}

/// 获取消息列表
#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/messages",
    params(
        ("chat_id" = i64, Path, description = "Chat ID")
    ),
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Messages retrieved successfully", body = Vec<Message>),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat not found", body = ErrorOutput)
    ),
    tag = "messages"
)]
pub async fn list_messages_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Query(query): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
  info!("User {} listing messages for chat {}", user.id, chat_id);

  let messages: Vec<crate::models::Message> = state.list_messages(query, chat_id.into()).await?;

  Ok((StatusCode::OK, Json(messages)))
}
