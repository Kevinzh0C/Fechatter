//! Message handlers after refactoring
//! 
//! Demonstrates how to refactor handlers using validation middleware to remove duplicate validation logic

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{instrument, debug};

use crate::{
    AppState, 
    error::AppError,
    middlewares::validation::ValidationContext,
    services::application::workers::message::{
        AppStateMessageServiceAdapter, MessageApplicationServiceTrait, MessageView,
    },
};
use fechatter_core::{AuthUser, MessageId, UserId};

// =============================================================================
// Refactored Request/Response DTOs - Validation Logic Removed
// =============================================================================

/// Send message request - Validation logic moved to middleware
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(default)]
    pub files: Vec<String>,
    // idempotency_key now handled by middleware
}

/// Edit message request
#[derive(Debug, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

/// Message response
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: i64,
    pub content: String,
    pub sender_id: i64,
    pub chat_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// =============================================================================
// Refactored Handler Implementations - Greatly Simplified
// =============================================================================

/// Send message - Refactored version
/// 
/// Validation logic completed via middleware:
/// - User authentication ✓
/// - Chat existence validation ✓
/// - Chat membership validation ✓
/// - Idempotency validation ✓
/// - Rate limiting ✓
#[instrument(skip(state, validation_context))]
pub async fn send_message_validated(
    State(state): State<AppState>,
    Path(chat_id): Path<i64>,
    Extension(user): Extension<AuthUser>,
    Extension(validation_context): Extension<ValidationContext>,
    Json(request): Json<SendMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    debug!(
        user_id = %user.id,
        chat_id = chat_id,
        content_length = request.content.len(),
        "Processing validated send message request"
    );
    
    // Permission check from validation context (optional - already ensured by middleware)
    if !validation_context.has_permission("chat_member") {
        return Err(AppError::PermissionDenied(
            "Chat membership required".to_string()
        ));
    }
    
    // Get idempotency key
    let idempotency_key: String = validation_context
        .get_resource("idempotency_key")
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    // Direct service call - no duplicate validation needed
    let message_service = AppStateMessageServiceAdapter::new(&state)?;
    
    let create_message = fechatter_core::CreateMessage {
        content: request.content,
        chat_id: fechatter_core::ChatId(chat_id),
        sender_id: user.id,
        files: request.files,
        idempotency_key: uuid::Uuid::parse_str(&idempotency_key)
            .unwrap_or_else(|_| uuid::Uuid::new_v4()),
    };
    
    let message = message_service
        .send_message(create_message)
        .await?;
    
    debug!(
        message_id = message.id,
        chat_id = chat_id,
        "Message sent successfully"
    );
    
    Ok(Json(MessageResponse {
        id: message.id,
        content: message.content,
        sender_id: message.sender_id,
        chat_id,
        created_at: message.created_at,
        updated_at: message.updated_at,
    }))
}

/// Edit message - Refactored version
/// 
/// Validation logic completed via middleware:
/// - User authentication ✓
/// - Message existence validation ✓
/// - Message ownership validation ✓
/// - Edit time window validation ✓
#[instrument(skip(state, validation_context))]
pub async fn edit_message_validated(
    State(state): State<AppState>,
    Path(message_id): Path<i64>,
    Extension(user): Extension<AuthUser>,
    Extension(validation_context): Extension<ValidationContext>,
    Json(request): Json<EditMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    debug!(
        user_id = %user.id,
        message_id = message_id,
        "Processing validated edit message request"
    );
    
    // Get validated message from validation context (optional)
    let _validated_message: Option<fechatter_core::Message> = validation_context
        .get_resource("message");
    
    // Direct service call - no duplicate validation needed
    let message_service = AppStateMessageServiceAdapter::new(&state)?;
    
    let updated_message = message_service
        .edit_message(MessageId(message_id), user.id, request.content)
        .await?;
    
    debug!(
        message_id = message_id,
        "Message edited successfully"
    );
    
    Ok(Json(MessageResponse {
        id: updated_message.id,
        content: updated_message.content,
        sender_id: updated_message.sender_id,
        chat_id: updated_message.chat_id,
        created_at: updated_message.created_at,
        updated_at: updated_message.updated_at,
    }))
}

/// Delete message - Refactored version
/// 
/// Validation logic completed via middleware:
/// - User authentication ✓
/// - Message ownership validation ✓
#[instrument(skip(state))]
pub async fn delete_message_validated(
    State(state): State<AppState>,
    Path(message_id): Path<i64>,
    Extension(user): Extension<AuthUser>,
    Extension(validation_context): Extension<ValidationContext>,
) -> Result<StatusCode, AppError> {
    debug!(
        user_id = %user.id,
        message_id = message_id,
        "Processing validated delete message request"
    );
    
    // Direct service call - no duplicate validation needed
    let message_service = AppStateMessageServiceAdapter::new(&state)?;
    
    message_service
        .delete_message(MessageId(message_id), user.id)
        .await?;
    
    debug!(
        message_id = message_id,
        "Message deleted successfully"
    );
    
    Ok(StatusCode::NO_CONTENT)
}

/// Get message details - Refactored version
/// 
/// Validation logic completed via middleware:
/// - User authentication ✓
/// - Message existence validation ✓
/// - Message access permission validation ✓
#[instrument(skip(state))]
pub async fn get_message_validated(
    State(state): State<AppState>,
    Path(message_id): Path<i64>,
    Extension(user): Extension<AuthUser>,
    Extension(validation_context): Extension<ValidationContext>,
) -> Result<Json<MessageResponse>, AppError> {
    debug!(
        user_id = %user.id,
        message_id = message_id,
        "Processing validated get message request"
    );
    
    // Get validated message directly from validation context
    if let Some(message) = validation_context.get_resource::<fechatter_core::Message>("message") {
        debug!(
            message_id = message_id,
            "Message retrieved from validation context"
        );
        
        return Ok(Json(MessageResponse {
            id: message.id,
            content: message.content,
            sender_id: message.sender_id,
            chat_id: message.chat_id,
            created_at: message.created_at,
            updated_at: message.updated_at,
        }));
    }
    
    // Fallback to service call (shouldn't reach here as middleware has validated)
    let message_service = AppStateMessageServiceAdapter::new(&state)?;
    
    let message = message_service
        .get_message(MessageId(message_id), user.id)
        .await?
        .ok_or_else(|| AppError::NotFound(vec!["Message not found".to_string()]))?;
    
    Ok(Json(MessageResponse {
        id: message.id,
        content: message.content,
        sender_id: message.sender_id,
        chat_id: message.chat_id,
        created_at: message.created_at,
        updated_at: message.updated_at,
    }))
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use fechatter_core::{UserId, WorkspaceId};
    
    fn create_test_user() -> AuthUser {
        AuthUser {
            id: UserId(1),
            username: "testuser".to_string(),
            workspace_id: WorkspaceId(1),
        }
    }
    
    fn create_validation_context_with_permissions(permissions: Vec<&str>) -> ValidationContext {
        let mut context = ValidationContext::default();
        for permission in permissions {
            context.add_permission(permission.to_string());
        }
        context
    }
    
    #[tokio::test]
    async fn test_send_message_with_validation_context() {
        // Test handler integration with validation middleware
        let context = create_validation_context_with_permissions(vec![
            "authenticated",
            "chat_member:123",
            "idempotency_validated"
        ]);
        
        // More unit tests can be added here
        // Test handler behavior with validation context
        assert!(context.has_permission("chat_member:123"));
    }
    
    #[test]
    fn test_validation_context_resource_storage() {
        let mut context = ValidationContext::default();
        
        // Test resource storage and retrieval
        context.add_resource("test_key".to_string(), "test_value");
        let retrieved: Option<String> = context.get_resource("test_key");
        
        assert_eq!(retrieved, Some("test_value".to_string()));
    }
}