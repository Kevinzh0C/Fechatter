//! Fixed Realtime event handlers using the proper event infrastructure
//! 
//! These handlers integrate with the typing service and dual-stream architecture

use crate::{AppError, AppState};
use axum::{
    extract::{Path, State},
    response::Json,
    Extension,
};
use chrono::Utc;
use fechatter_core::AuthUser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct TypingRequest {
    pub chat_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct PresenceUpdate {
    pub status: String, // "online", "away", "offline"
}

/// Start typing indicator - Uses typing service and dual-stream dispatcher
pub async fn start_typing(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<TypingRequest>,
) -> Result<Json<Value>, AppError> {
    // 1. Validate user is member of chat
    let chat_service = state.chat_domain_service();
    let members = chat_service
        .get_chat_members(req.chat_id)
        .await
        .map_err(|e| AppError::BadRequest(vec![e.to_string()]))?;
    
    if !members.contains(&auth.user_id) {
        return Err(AppError::Forbidden("Not a member of this chat".to_string()));
    }
    
    // 2. Update typing state using TypingIndicatorService
    let typing_service = state.application_services().typing_indicator_service();
    typing_service
        .set_typing(
            req.chat_id,
            auth.user_id,
            auth.name.clone().unwrap_or_else(|| format!("User {}", auth.user_id)),
            true,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to set typing state: {}", e)))?;
    
    // 3. Get message service which has the dual-stream dispatcher
    let message_service = state.message_application_service();
    
    // 4. Use the dual-stream dispatcher to publish typing event
    let typing_event = crate::services::application::workers::message::RealtimeEvent::TypingStarted {
        chat_id: req.chat_id,
        user_id: auth.user_id,
        user_name: auth.name.clone().unwrap_or_else(|| format!("User {}", auth.user_id)),
    };
    
    // The message service has access to the dispatcher
    if let Err(e) = message_service.domain_service()
        .send_typing_event(typing_event)
        .await {
        // Log but don't fail - typing is non-critical
        tracing::warn!("Failed to publish typing event: {}", e);
    }
    
    info!("User {} started typing in chat {}", auth.user_id, req.chat_id);
    
    Ok(Json(json!({
        "status": "ok"
    })))
}

/// Stop typing indicator
pub async fn stop_typing(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<TypingRequest>,
) -> Result<Json<Value>, AppError> {
    // 1. Update typing state
    let typing_service = state.application_services().typing_indicator_service();
    typing_service
        .set_typing(
            req.chat_id,
            auth.user_id,
            String::new(), // Name not needed for stop
            false,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to clear typing state: {}", e)))?;
    
    // 2. Publish typing stopped event
    let message_service = state.message_application_service();
    let typing_event = crate::services::application::workers::message::RealtimeEvent::TypingStopped {
        chat_id: req.chat_id,
        user_id: auth.user_id,
    };
    
    if let Err(e) = message_service.domain_service()
        .send_typing_event(typing_event)
        .await {
        tracing::warn!("Failed to publish typing stopped event: {}", e);
    }
    
    info!("User {} stopped typing in chat {}", auth.user_id, req.chat_id);
    
    Ok(Json(json!({
        "status": "ok"
    })))
}

/// Mark message as read
pub async fn mark_message_read(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(message_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // 1. Get message details
    let message_service = state.message_domain_service();
    let message = message_service
        .get_message(message_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(vec!["Message not found".to_string()]))?;
    
    // 2. Verify user is member of chat
    let chat_service = state.chat_domain_service();
    let members = chat_service
        .get_chat_members(message.chat_id.into())
        .await
        .map_err(|e| AppError::BadRequest(vec![e.to_string()]))?;
    
    if !members.contains(&auth.user_id) {
        return Err(AppError::Forbidden("Not a member of this chat".to_string()));
    }
    
    // 3. Mark as read in domain service
    message_service
        .mark_message_read(message_id, auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    // 4. Publish read receipt event
    let app_message_service = state.message_application_service();
    app_message_service
        .mark_message_read(
            fechatter_core::MessageId(message_id),
            fechatter_core::ChatId(message.chat_id.into()),
            fechatter_core::UserId(auth.user_id),
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Json(json!({
        "status": "ok",
        "read_at": Utc::now().to_rfc3339()
    })))
}

/// Update user presence
pub async fn update_presence(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<PresenceUpdate>,
) -> Result<Json<Value>, AppError> {
    // 1. Validate status
    let valid_statuses = ["online", "away", "offline"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return Err(AppError::BadRequest(vec![
            format!("Invalid status. Must be one of: {:?}", valid_statuses)
        ]));
    }
    
    // 2. Update presence using presence service
    let presence_service = state.application_services().presence_service();
    presence_service
        .update_presence(auth.user_id, &req.status)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update presence: {}", e)))?;
    
    // 3. Publish presence event
    let message_service = state.message_application_service();
    message_service
        .update_user_presence(
            fechatter_core::UserId(auth.user_id),
            req.status.clone(),
            Some(Utc::now().to_rfc3339()),
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Json(json!({
        "status": "ok",
        "presence": req.status
    })))
}

/// Get typing users in a chat
pub async fn get_typing_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // Verify user is member
    let chat_service = state.chat_domain_service();
    let members = chat_service
        .get_chat_members(chat_id)
        .await
        .map_err(|e| AppError::BadRequest(vec![e.to_string()]))?;
    
    if !members.contains(&auth.user_id) {
        return Err(AppError::Forbidden("Not a member of this chat".to_string()));
    }
    
    // Get typing users from service
    let typing_service = state.application_services().typing_indicator_service();
    let typing_users = typing_service.get_typing_users(chat_id).await;
    
    // Filter out self and format response
    let typing_data: Vec<_> = typing_users
        .into_iter()
        .filter(|u| u.user_id != auth.user_id)
        .map(|u| json!({
            "user_id": u.user_id,
            "user_name": u.user_name,
            "started_at": u.started_at.elapsed().as_secs()
        }))
        .collect();
    
    Ok(Json(json!({
        "chat_id": chat_id,
        "typing_users": typing_data
    })))
}

/// Get read receipts for a message
pub async fn get_message_receipts(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(message_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // Get message and verify access
    let message_service = state.message_domain_service();
    let message = message_service
        .get_message(message_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(vec!["Message not found".to_string()]))?;
    
    // Get chat members
    let chat_service = state.chat_domain_service();
    let members = chat_service
        .get_chat_members(message.chat_id.into())
        .await
        .map_err(|e| AppError::BadRequest(vec![e.to_string()]))?;
    
    if !members.contains(&auth.user_id) {
        return Err(AppError::Forbidden("Not a member of this chat".to_string()));
    }
    
    // Get read receipts from message service
    let receipts = message_service
        .get_read_receipts(message_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let receipt_data: Vec<_> = receipts
        .into_iter()
        .map(|(user_id, read_at)| json!({
            "user_id": user_id,
            "read_at": read_at.to_rfc3339()
        }))
        .collect();
    
    Ok(Json(json!({
        "message_id": message_id,
        "receipts": receipt_data
    })))
}