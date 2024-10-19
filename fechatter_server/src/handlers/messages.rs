// use axum::{
//   Json,
//   extract::{Extension, Path, State},
//   http::StatusCode,
//   response::IntoResponse,
// };
// use serde::{Deserialize, Serialize};
// use tracing::info;

// use crate::{
//   AppError, AppState,
//   models::{AuthUser, create_message, is_member_in_chat, list_messages},
// };

// #[derive(Debug, Deserialize)]
// pub struct MessageRequest {
//   content: String,
// }

// #[derive(Debug, Serialize)]
// pub struct MessageResponse {
//   id: i64,
//   content: String,
//   sender_id: i64,
//   created_at: chrono::DateTime<chrono::Utc>,
// }

// /// 发送消息到指定聊天
// pub(crate) async fn send_message_handler(
//   State(state): State<AppState>,
//   Extension(user): Extension<AuthUser>,
//   Path(chat_id): Path<i64>,
//   Json(request): Json<MessageRequest>,
// ) -> Result<impl IntoResponse, AppError> {
//   info!("User {} sending message to chat {}", user.id, chat_id);

//   // 验证用户是否是聊天成员
//   let is_member = is_member_in_chat(&state.pool, user.id, chat_id).await?;

//   if !is_member {
//     return Err(AppError::Unauthorized("您不是该聊天的成员".into()));
//   }

//   // 验证消息内容不为空
//   if request.content.trim().is_empty() {
//     return Err(AppError::BadRequest("消息内容不能为空".into()));
//   }

//   // 创建消息
//   let message = create_message(&state.pool, chat_id, user.id, request.content).await?;

//   // 返回响应
//   let response = MessageResponse {
//     id: message.id,
//     content: message.content,
//     sender_id: message.sender_id,
//     created_at: chrono::DateTime::from_naive_utc_and_offset(message.created_at, chrono::Utc),
//   };

//   Ok((StatusCode::CREATED, Json(response)))
// }

// /// 获取指定聊天的所有消息
// pub(crate) async fn list_message_handler(
//   State(state): State<AppState>,
//   Extension(user): Extension<AuthUser>,
//   Path(chat_id): Path<i64>,
// ) -> Result<impl IntoResponse, AppError> {
//   info!("User {} listing messages for chat {}", user.id, chat_id);

//   // 验证用户是否是聊天成员
//   let is_member = is_member_in_chat(&state.pool, user.id, chat_id).await?;

//   if !is_member {
//     return Err(AppError::Unauthorized("您不是该聊天的成员".into()));
//   }

//   // 查询该聊天的消息
//   let messages = list_messages(&state.pool, chat_id).await?;

//   // 转换为响应格式
//   let responses: Vec<MessageResponse> = messages
//     .into_iter()
//     .map(|msg| MessageResponse {
//       id: msg.id,
//       content: msg.content,
//       sender_id: msg.sender_id,
//       created_at: chrono::DateTime::from_naive_utc_and_offset(msg.created_at, chrono::Utc),
//     })
//     .collect();

//   Ok(Json(responses))
// }
