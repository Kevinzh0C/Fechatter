//! Realtime event handlers (typing, presence, read receipts)
//!
//! These handlers manage ephemeral state and broadcast events to notify-server

use crate::{AppError, AppState};
use axum::{
    extract::{Path, State},
    response::Json,
    Extension,
};
use chrono::Utc;
use fechatter_core::AuthUser;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct PresenceUpdate {
    pub status: String, // "online", "away", "offline"
}

/// Start typing indicator
pub async fn start_typing(
    Extension(state): Extension<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // 1. Validate chat membership
    let chat_service = state.application_services().chat_application_service();
    chat_service
        .ensure_user_is_chat_member(auth.id.into(), chat_id)
        .await?;

    // 2. Update typing state using TypingIndicatorService
    let typing_service = state.application_services().typing_indicator_service();
    typing_service
        .set_typing(chat_id, auth.id.into(), auth.fullname.clone(), true)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to set typing state: {}", e)))?;

    // 3. Publish typing event through message service
    let message_service = state.application_services().message_service();
    message_service
        .start_typing(
            fechatter_core::ChatId(chat_id),
            fechatter_core::UserId(auth.id.into()),
            auth.fullname.clone(),
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to publish typing event: {}", e)))?;

    Ok(Json(json!({
        "status": "ok"
    })))
}

/// Stop typing indicator
pub async fn stop_typing(
    Extension(state): Extension<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // 1. Validate chat membership
    let chat_service = state.application_services().chat_application_service();
    chat_service
        .ensure_user_is_chat_member(auth.id.into(), chat_id)
        .await?;

    // 2. Update typing state
    let typing_service = state.application_services().typing_indicator_service();
    typing_service
        .set_typing(
            chat_id,
            auth.id.into(),
            String::new(), // Name not needed for stop
            false,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to clear typing state: {}", e)))?;

    // 3. Publish typing stopped event
    let message_service = state.application_services().message_service();
    message_service
        .stop_typing(
            fechatter_core::ChatId(chat_id),
            fechatter_core::UserId(auth.id.into()),
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to publish typing event: {}", e)))?;

    Ok(Json(json!({
        "status": "ok"
    })))
}

/// Mark message as read
pub async fn mark_message_read(
    Extension(state): Extension<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((chat_id, message_id)): Path<(i64, i64)>,
) -> Result<Json<Value>, AppError> {
    // Validate chat membership
    let chat_service = state.application_services().chat_application_service();
    chat_service
        .ensure_user_is_chat_member(auth.id.into(), chat_id)
        .await?;

    // Publish read receipt event through message service
    let app_message_service = state.application_services().message_service();
    app_message_service
        .mark_message_read(
            fechatter_core::MessageId(message_id),
            fechatter_core::ChatId(chat_id),
            fechatter_core::UserId(auth.id.into()),
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
    Extension(state): Extension<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<PresenceUpdate>,
) -> Result<Json<Value>, AppError> {
    // 1. Validate status
    let valid_statuses = ["online", "away", "offline"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid status. Must be one of: {:?}",
            valid_statuses
        )));
    }

    // 2. Publish presence event through message service (no local state management)
    let message_service = state.application_services().message_service();
    message_service
        .update_user_presence(
            fechatter_core::UserId(auth.id.into()),
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
    Extension(state): Extension<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // Validate chat membership
    let chat_service = state.application_services().chat_application_service();
    chat_service
        .ensure_user_is_chat_member(auth.id.into(), chat_id)
        .await?;

    // Get typing users from typing service
    let typing_service = state.application_services().typing_indicator_service();
    let typing_users = typing_service.get_typing_users(chat_id).await;

    // Filter out self and format response
    let auth_user_id: i64 = auth.id.into();
    let typing_data: Vec<_> = typing_users
        .into_iter()
        .filter(|u| u.user_id != auth_user_id)
        .map(|u| {
            json!({
                "user_id": u.user_id,
                "user_name": u.user_name,
                "started_at": u.started_at.elapsed().as_secs()
            })
        })
        .collect();

    Ok(Json(json!({
        "chat_id": chat_id,
        "typing_users": typing_data
    })))
}

/// Get read receipts for a message
pub async fn get_message_receipts(
    Extension(state): Extension<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(message_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // Skip message and chat validation for now
    // TODO: Add proper message access validation

    // Get read receipts from message service
    // Note: This requires implementing get_read_receipts in the domain service
    // For now, we'll return empty until the domain service is updated
    let receipts = Vec::<Value>::new();

    // TODO: When domain service supports get_read_receipts:
    // let receipts = message_service
    //     .get_read_receipts(message_id)
    //     .await
    //     .map_err(|e| AppError::Internal(e.to_string()))?
    //     .into_iter()
    //     .map(|(user_id, read_at)| json!({
    //         "user_id": user_id,
    //         "read_at": read_at.to_rfc3339()
    //     }))
    //     .collect();

    Ok(Json(json!({
        "message_id": message_id,
        "receipts": receipts
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_update_deserialization() {
        let json = r#"{"status": "online"}"#;
        let req: PresenceUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, "online");
    }
}
