//! # Messages Handlers - Production Implementation
//!
//! **Responsibility**: HTTP message processing, delegated to application services
//! **Principle**: Clean design, single responsibility

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use validator::Validate;

use crate::dtos::core::ApiResponse;
use crate::dtos::models::requests::message::{EditMessageRequest, SendMessageRequest};
use crate::services::application::workers::message::MessageView;
use crate::{AppError, AppState};
use fechatter_core::{AuthUser, ChatId, CreateMessage, ListMessages, MessageId, UserId};

// =============================================================================
// LOCAL DTOs - Local data transfer objects
// =============================================================================

/// Message List Query DTO
#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub before: Option<i64>,
}

fn default_limit() -> i64 {
    50
}

/// Sender Response DTO
#[derive(Debug, Serialize)]
pub struct SenderResponse {
    pub id: i64,
    pub fullname: String,
    pub username: Option<String>,
    pub email: Option<String>,
}

/// Message Response DTO
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub sender: Option<SenderResponse>, // Added sender information
    pub content: String,
    pub files: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// =============================================================================
// DTO CONVERSIONS
// =============================================================================

impl From<SendMessageRequest> for CreateMessage {
    fn from(request: SendMessageRequest) -> Self {
        Self {
            content: request.content,
            files: Some(request.files.unwrap_or_default()),
            idempotency_key: Some(request.idempotency_key.unwrap_or_else(uuid::Uuid::now_v7)),
        }
    }
}

impl From<ListMessagesQuery> for ListMessages {
    fn from(query: ListMessagesQuery) -> Self {
        Self {
            limit: query.limit,
            last_id: query.before,
        }
    }
}

impl From<MessageView> for MessageResponse {
    fn from(view: MessageView) -> Self {
        Self {
            id: view.id,
            chat_id: view.chat_id,
            sender_id: view.sender_id,
            sender: view.sender.map(|s| SenderResponse {
                id: s.id,
                fullname: s.fullname,
                username: s.username,
                email: s.email,
            }),
            content: view.content,
            files: view.files.unwrap_or_default(),
            created_at: view.created_at,
        }
    }
}

// =============================================================================
// HANDLERS
// =============================================================================

/// Send Message Handler
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn send_message_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
    Json(request): Json<SendMessageRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, AppError> {
    // VALIDATION: Validate request data
    request
        .validate()
        .map_err(|e| AppError::InvalidInput(format!("Message validation failed: {}", e)))?;

    // Additional business logic validation
    if request.content.trim().is_empty() && request.files.as_ref().map_or(true, |f| f.is_empty()) {
        return Err(AppError::InvalidInput(
            "Message must contain either text content or files".to_string(),
        ));
    }

    let create_message = CreateMessage::from(request.clone());
    let message_service = state.application_services().message_service();

    let message_view = message_service
        .send_message(UserId::from(user.id), ChatId::from(chat_id), create_message)
        .await?;

    // Track analytics event for message sent using unified analytics publisher
    if let Some(analytics_publisher) = state.analytics_publisher() {
        use crate::services::infrastructure::event::AnalyticsTracking;

        let message_size = message_view.content.len() as i32;
        let message_type = if message_view.files.as_ref().map_or(false, |f| !f.is_empty()) {
            "file".to_string()
        } else {
            "text".to_string()
        };

        if let Err(e) = analytics_publisher
            .track_message_sent(
                user.id.to_string(),
                chat_id.to_string(),
                message_type,
                message_size,
            )
            .await
        {
            tracing::warn!("Failed to track message analytics event: {}", e);
        }
    }

    // ========================================================================
    // NEW: notify_server SSE Integration - Complete Message Content
    // ========================================================================

    // Publish to notify_server for real-time SSE broadcasting
    if let Some(enhanced_publisher) = state.enhanced_event_publisher() {
        // Convert message to complete data for notify_server
        let complete_message = crate::services::infrastructure::event::message_to_complete_data(
            &fechatter_core::Message {
                id: fechatter_core::MessageId::from(message_view.id),
                chat_id: fechatter_core::ChatId::from(chat_id),
                sender_id: fechatter_core::UserId::from(user.id),
                content: message_view.content.clone(),
                files: message_view.files.clone(),
                created_at: message_view.created_at,
                idempotency_key: request.idempotency_key,
            },
            user.fullname.clone(),
        );

        if let Err(e) = enhanced_publisher
            .publish_complete_message_for_sse(complete_message, user.workspace_id.into())
            .await
        {
            tracing::warn!("Failed to publish message to notify_server for SSE: {}", e);
        } else {
            tracing::info!("SUBSCRIPTION: Successfully published complete message to notify_server for SSE broadcasting");
        }
    }

    // ========================================================================
    // LEGACY: Unified Event Publishing - Production-grade NATS Integration
    // ========================================================================

    // Publish unified event using enhanced event publisher
    if let Some(event_publisher) = get_unified_event_publisher(&state) {
        // Get additional context for the event
        let sender_name = user.fullname.clone();
        let chat_name = get_chat_name(&state, ChatId::from(chat_id))
            .await
            .unwrap_or_else(|_| format!("Chat {}", chat_id));
        let workspace_id = user.workspace_id.into();

        // Convert message_view to Message for the enhanced publisher
        let message = fechatter_core::Message {
            id: fechatter_core::MessageId::from(message_view.id),
            chat_id: fechatter_core::ChatId::from(chat_id),
            sender_id: fechatter_core::UserId::from(user.id),
            content: message_view.content.clone(),
            files: message_view.files.clone(),
            created_at: message_view.created_at,
            idempotency_key: request.idempotency_key,
        };

        // Get chat members (simplified - in production, this should come from chat service)
        let chat_members = vec![fechatter_core::UserId::from(user.id)]; // TODO: Get actual members

        if let Err(e) = event_publisher
            .publish_unified_message_created(
                &message,
                &chat_members,
                sender_name,
                chat_name,
                workspace_id,
            )
            .await
        {
            tracing::warn!("Failed to publish unified message created event: {}", e);
        } else {
            tracing::info!(
                "SUBSCRIPTION: Successfully published unified message created event for message {}",
                message_view.id
            );
        }
    }

    let response = MessageResponse::from(message_view);
    Ok(Json(ApiResponse::success(
        response,
        "message_sent".to_string(),
    )))
}

/// List Messages Handler
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn list_messages_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<ApiResponse<Vec<MessageResponse>>>, AppError> {
    let list_query = ListMessages::from(query);

    // Use service layer instead of direct database access
    let message_service = state.application_services().message_service();

    let messages = message_service
        .list_messages(
            fechatter_core::UserId::from(user.id),
            fechatter_core::ChatId::from(chat_id),
            list_query,
        )
        .await?;

    let responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(|message_view| MessageResponse {
            id: message_view.id,
            chat_id: message_view.chat_id,
            sender_id: message_view.sender_id,
            sender: message_view.sender.map(|s| SenderResponse {
                id: s.id,
                fullname: s.fullname,
                username: s.username,
                email: s.email,
            }),
            content: message_view.content,
            files: message_view.files.unwrap_or_default(),
            created_at: message_view.created_at,
        })
        .collect();

    Ok(Json(ApiResponse::success(
        responses,
        "messages_listed".to_string(),
    )))
}

/// Edit Message Handler
#[instrument(skip(state), fields(message_id = %message_id, user_id = %user.id))]
pub async fn edit_message_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((chat_id, message_id)): Path<(i64, i64)>,
    Json(request): Json<EditMessageRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let message_service = state.application_services().message_service();

    // Get original content for the event
    let original_content = get_message_content(&state, MessageId::from(message_id))
        .await
        .unwrap_or_else(|_| "".to_string());

    // Clone content for later use in event
    let new_content = request.content.clone();

    message_service
        .domain_service()
        .edit_message(message_id, request.content, i64::from(user.id))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // ========================================================================
    // NEW: notify_server SSE Integration for Message Edit
    // ========================================================================

    // Publish to notify_server for real-time SSE broadcasting
    if let Some(enhanced_publisher) = state.enhanced_event_publisher() {
        // Create complete message data for edited message
        let complete_message = crate::services::infrastructure::event::CompleteMessageData {
            id: message_id,
            chat_id,
            sender_id: user.id.into(),
            sender_name: user.fullname.clone(),
            content: new_content.clone(),
            files: vec![],                  // TODO: Get actual files from database
            created_at: chrono::Utc::now(), // TODO: Get actual created_at from database
            idempotency_key: None,
        };

        if let Err(e) = enhanced_publisher
            .publish_message_edited_for_sse(complete_message, user.workspace_id.into())
            .await
        {
            tracing::warn!(
                "Failed to publish edited message to notify_server for SSE: {}",
                e
            );
        } else {
            tracing::info!(
                "Successfully published edited message to notify_server for SSE broadcasting"
            );
        }
    }

    // ========================================================================
    // LEGACY: Unified Event Publishing for Message Edit
    // ========================================================================

    if let Some(event_publisher) = get_unified_event_publisher(&state) {
        let workspace_id = user.workspace_id.into();
        let editor_name = user.fullname.clone();
        let chat_name = get_chat_name(&state, ChatId::from(chat_id))
            .await
            .unwrap_or_else(|_| format!("Chat {}", chat_id));

        // Create a message structure for the edited message
        let message = fechatter_core::Message {
            id: fechatter_core::MessageId::from(message_id),
            chat_id: fechatter_core::ChatId::from(chat_id),
            sender_id: fechatter_core::UserId::from(user.id),
            content: new_content.clone(),
            files: None,                    // TODO: Get actual files from database
            created_at: chrono::Utc::now(), // TODO: Get actual created_at from database
            idempotency_key: None,
        };

        if let Err(e) = event_publisher
            .publish_unified_message_edited(&message, editor_name, chat_name, workspace_id)
            .await
        {
            tracing::warn!("Failed to publish unified message edited event: {}", e);
        } else {
            tracing::info!(
                "Successfully published unified message edited event for message {}",
                message_id
            );
        }
    }

    Ok(Json(ApiResponse::success((), "message_edited".to_string())))
}

/// Delete Message Handler
#[instrument(skip(state), fields(message_id = %message_id, user_id = %user.id))]
pub async fn delete_message_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((chat_id, message_id)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
    let message_service = state.application_services().message_service();

    // Delete the message
    message_service
        .domain_service()
        .delete_message(message_id.into(), user.id.into())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // ========================================================================
    // NEW: notify_server SSE Integration for Message Delete
    // ========================================================================

    // Publish to notify_server for real-time SSE broadcasting
    if let Some(enhanced_publisher) = state.enhanced_event_publisher() {
        if let Err(e) = enhanced_publisher
            .publish_message_deleted_for_sse(
                message_id,
                chat_id,
                user.id.into(),
                user.fullname.clone(),
                user.workspace_id.into(),
            )
            .await
        {
            tracing::warn!(
                "Failed to publish deleted message to notify_server for SSE: {}",
                e
            );
        } else {
            tracing::info!(
                "Successfully published deleted message to notify_server for SSE broadcasting"
            );
        }
    }

    // ========================================================================
    // LEGACY: Unified Event Publishing for Message Delete
    // ========================================================================

    if let Some(event_publisher) = get_unified_event_publisher(&state) {
        let workspace_id = user.workspace_id.into();
        let deleted_by_name = user.fullname.clone();
        let chat_name = get_chat_name(&state, ChatId::from(chat_id))
            .await
            .unwrap_or_else(|_| format!("Chat {}", chat_id));

        // Create a message structure for the deleted message
        let message = fechatter_core::Message {
            id: fechatter_core::MessageId::from(message_id),
            chat_id: fechatter_core::ChatId::from(chat_id),
            sender_id: fechatter_core::UserId::from(user.id),
            content: "".to_string(), // Content is not relevant for delete events
            files: None,             // TODO: Get actual files from database if needed
            created_at: chrono::Utc::now(), // TODO: Get actual created_at from database
            idempotency_key: None,
        };

        if let Err(e) = event_publisher
            .publish_unified_message_deleted(&message, deleted_by_name, chat_name, workspace_id)
            .await
        {
            tracing::warn!("Failed to publish unified message deleted event: {}", e);
        } else {
            tracing::info!(
                "Successfully published unified message deleted event for message {}",
                message_id
            );
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// READ/UNREAD STATUS HANDLERS
// =============================================================================

/// Mark messages as read request
#[derive(Debug, Deserialize)]
pub struct MarkReadRequest {
    pub message_ids: Vec<i64>,
}

/// Unread count response
#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub chat_id: i64,
    pub unread_count: i64,
}

/// Mark messages as read
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn mark_messages_read_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
    Json(request): Json<MarkReadRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let message_service = state.application_services().message_service();

    // Mark messages as read in database
    message_service
        .domain_service()
        .mark_messages_read_batch(&request.message_ids, user.id.into())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // ========================================================================
    // NEW: notify_server SSE Integration for Read Receipts
    // ========================================================================

    // Publish to notify_server for real-time SSE broadcasting
    if let Some(enhanced_publisher) = state.enhanced_event_publisher() {
        if let Err(e) = enhanced_publisher
            .publish_read_receipts_for_sse(chat_id, i64::from(user.id), request.message_ids.clone())
            .await
        {
            tracing::warn!(
                "Failed to publish read receipts to notify_server for SSE: {}",
                e
            );
        } else {
            tracing::info!(
                "Successfully published read receipts to notify_server for SSE broadcasting"
            );
        }
    }

    // ========================================================================
    // LEGACY: Unified Event Publishing for Read Receipts
    // ========================================================================

    if let Some(event_publisher) = get_unified_event_publisher(&state) {
        let workspace_id = user.workspace_id.into();
        let message_ids: Vec<MessageId> = request
            .message_ids
            .iter()
            .map(|&id| MessageId::from(id))
            .collect();

        if let Err(e) = event_publisher
            .publish_unified_message_read_receipt(
                &ChatId::from(chat_id),
                &UserId::from(user.id),
                message_ids,
                workspace_id,
            )
            .await
        {
            tracing::warn!("Failed to publish unified read receipt event: {}", e);
        } else {
            tracing::info!(
                "Successfully published unified read receipt event for {} messages in chat {}",
                request.message_ids.len(),
                chat_id
            );
        }
    }

    Ok(Json(ApiResponse::success(
        (),
        format!("marked_{}_messages_as_read", request.message_ids.len()),
    )))
}

/// Get unread message count
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn get_unread_count_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
) -> Result<Json<ApiResponse<UnreadCountResponse>>, AppError> {
    let message_service = state.application_services().message_service();

    let unread_count = message_service
        .domain_service()
        .get_unread_count(chat_id, user.id.into())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(
        UnreadCountResponse {
            chat_id,
            unread_count,
        },
        "unread_count_retrieved".to_string(),
    )))
}

/// Get all unread counts for user's chats
#[instrument(skip(state), fields(user_id = %user.id))]
pub async fn get_all_unread_counts_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<UnreadCountResponse>>>, AppError> {
    // Get user's chats
    let chat_service = state.application_services().chat_application_service();
    let chats = chat_service
        .list_user_chats(user.id.into())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Get unread count for each chat
    let message_service = state.application_services().message_service();
    let mut unread_counts = Vec::new();

    for chat in chats {
        let unread_count = message_service
            .domain_service()
            .get_unread_count(chat.id.into(), user.id.into())
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        unread_counts.push(UnreadCountResponse {
            chat_id: chat.id.into(),
            unread_count,
        });
    }

    Ok(Json(ApiResponse::success(
        unread_counts,
        "all_unread_counts_retrieved".to_string(),
    )))
}

// =============================================================================
// HELPER FUNCTIONS - Production utilities
// =============================================================================

/// Get unified event publisher from app state
fn get_unified_event_publisher(
    state: &AppState,
) -> Option<&crate::services::infrastructure::event::DynEventPublisher> {
    state.unified_event_publisher().map(|arc| arc.as_ref())
}

/// Get chat name for event context
async fn get_chat_name(state: &AppState, chat_id: ChatId) -> Result<String, AppError> {
    let chat_service = state.application_services().chat_application_service();

    // Use a temporary user_id for this lookup (ideally this should be passed from the caller)
    // For now, we'll use a placeholder approach since this is a helper function
    match chat_service.get_chat(chat_id.into(), 1).await {
        Ok(Some(chat)) => Ok(chat.name),
        Ok(None) => Ok(format!("Chat {}", chat_id.0)),
        Err(_) => Ok(format!("Chat {}", chat_id.0)),
    }
}

/// Get message content for edit events
async fn get_message_content(state: &AppState, message_id: MessageId) -> Result<String, AppError> {
    // TODO: Implement proper message lookup by ID
    // This would require a message repository method to get message by ID
    // For now, return empty string to avoid compilation errors
    Ok("".to_string())
}

/// Get mentions for a message
#[derive(Debug, Serialize)]
pub struct MentionResponse {
    pub user_id: i64,
    pub username: String,
    pub fullname: String,
    pub mention_type: String,
}

/// Get message mentions
#[instrument(skip(state), fields(message_id = %message_id))]
pub async fn get_message_mentions_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(message_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<MentionResponse>>>, AppError> {
    // Use service layer instead of direct database access
    let message_service = state.application_services().message_service();

    let mentions_data = message_service.get_message_mentions(message_id).await?;

    let mentions: Vec<MentionResponse> = mentions_data
        .into_iter()
        .map(
            |(user_id, username, fullname, mention_type)| MentionResponse {
                user_id,
                username,
                fullname,
                mention_type,
            },
        )
        .collect();

    Ok(Json(ApiResponse::success(
        mentions,
        "message_mentions_retrieved".to_string(),
    )))
}

/// Get unread mentions for current user
#[derive(Debug, Serialize)]
pub struct UnreadMentionResponse {
    pub chat_id: i64,
    pub message_id: i64,
    pub content: String,
    pub sender_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub mention_type: String,
}

/// Get unread mentions for the current user across all chats
#[instrument(skip(state), fields(user_id = %user.id))]
pub async fn get_unread_mentions_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<UnreadMentionResponse>>>, AppError> {
    // Use service layer instead of direct database access
    let message_service = state.application_services().message_service();

    let mentions_data = message_service
        .get_unread_mentions_for_user(i64::from(user.id))
        .await?;

    let mentions: Vec<UnreadMentionResponse> = mentions_data
        .into_iter()
        .map(
            |(chat_id, message_id, content, sender_name, created_at, mention_type)| {
                UnreadMentionResponse {
                    chat_id,
                    message_id,
                    content,
                    sender_name,
                    created_at,
                    mention_type,
                }
            },
        )
        .collect();

    Ok(Json(ApiResponse::success(
        mentions,
        "unread_mentions_retrieved".to_string(),
    )))
}

/// Enhanced read receipts with detailed info
#[derive(Debug, Serialize)]
pub struct DetailedReceiptResponse {
    pub user_id: i64,
    pub username: String,
    pub fullname: String,
    pub status: String, // 'delivered' or 'read'
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Get detailed read receipts for a message
#[instrument(skip(state), fields(message_id = %message_id))]
pub async fn get_detailed_message_receipts_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(message_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<DetailedReceiptResponse>>>, AppError> {
    // Use service layer instead of direct database access
    let message_service = state.application_services().message_service();

    let receipts_data = message_service
        .get_detailed_message_receipts(message_id)
        .await?;

    let receipts: Vec<DetailedReceiptResponse> = receipts_data
        .into_iter()
        .map(
            |(user_id, username, fullname, status, timestamp)| DetailedReceiptResponse {
                user_id,
                username,
                fullname,
                status,
                timestamp,
            },
        )
        .collect();

    Ok(Json(ApiResponse::success(
        receipts,
        "detailed_receipts_retrieved".to_string(),
    )))
}

/// Mark message as read with enhanced tracking
#[instrument(skip(state), fields(chat_id = %chat_id, message_id = %message_id, user_id = %user.id))]
pub async fn mark_message_read_enhanced_handler(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthUser>,
    Path((chat_id, message_id)): Path<(i64, i64)>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    // Use service layer instead of direct database access
    let message_service = state.application_services().message_service();

    message_service
        .mark_message_read_enhanced(i64::from(user.id), chat_id, message_id)
        .await?;

    Ok(Json(ApiResponse::success(
        (),
        "message_marked_read".to_string(),
    )))
}

/// Extract mentioned users from message content
fn extract_mentioned_users(content: &str) -> Vec<UserId> {
    use regex::Regex;

    let mut mentioned_users = Vec::new();

    // Extract @username mentions
    if let Ok(re) = Regex::new(r"@(\w+)") {
        for cap in re.captures_iter(content) {
            if let Some(username) = cap.get(1) {
                // In a real implementation, you'd look up the username in the database
                // For now, we'll just store the pattern
                tracing::debug!("Found mention: @{}", username.as_str());
            }
        }
    }

    mentioned_users
}
