use async_nats::{Message, Subscriber};
use chrono::Utc;
use futures::StreamExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::{
    analytics::types::NotifyEventHelper,
    error::NotifyError,
    state::app_state::ConnectionUpdate,
    state::AppState,
};
use fechatter_core::{ChatId, UserId};

/// Event processor for handling incoming NATS events
pub struct EventProcessor {
    nats_subscriber: Subscriber,
    state: Arc<AppState>,
}

impl EventProcessor {
    /// Create a new event processor
    pub async fn new(
        nats_subscriber: Subscriber,
        state: Arc<AppState>,
    ) -> Result<Self, NotifyError> {
        Ok(Self {
            nats_subscriber,
            state,
        })
    }

    /// Start processing events
    pub async fn start(mut self) -> Result<(), NotifyError> {
        info!("Starting event processor");

        while let Some(message) = self.nats_subscriber.next().await {
            if let Err(e) = self.process_message(message).await {
                error!("Failed to process message: {}", e);
            }
        }

        warn!("Event processor stopped");
        Ok(())
    }

    /// Process a single NATS message
    async fn process_message(&self, message: Message) -> Result<(), NotifyError> {
        let subject = &message.subject;
        let payload_size = message.payload.len();
        
        // Add INFO level logging for event reception
        info!("📨 [NOTIFY] Received NATS event from subject: {} (size: {} bytes)", subject, payload_size);

        // Parse message payload
        let payload: Value = match serde_json::from_slice(&message.payload) {
            Ok(payload) => {
                info!("✅ [NOTIFY] Successfully parsed JSON payload from: {}", subject);
                payload
            }
            Err(e) => {
                error!("❌ [NOTIFY] Failed to parse JSON from {}: {}", subject, e);
                return Err(NotifyError::InvalidJson(format!("Failed to parse JSON: {}", e)));
            }
        };

        // Route based on subject
        match subject.as_str() {
            s if s.starts_with("fechatter.chat.") => {
                info!("🗨️ [NOTIFY] Processing chat event from: {}", s);
                self.handle_chat_event(payload).await?;
            }
            s if s.starts_with("fechatter.user.") => {
                info!("👤 [NOTIFY] Processing user event from: {}", s);
                self.handle_user_event(payload).await?;
            }
            s if s.starts_with("fechatter.message.") => {
                info!("💬 [NOTIFY] Processing message event from: {}", s);
                self.handle_message_event(payload).await?;
            }
            s if s.starts_with("fechatter.realtime.") => {
                info!("⚡ [NOTIFY] Processing realtime event from: {}", s);
                self.handle_realtime_event(payload).await?;
            }
            _ => {
                warn!("⚠️ [NOTIFY] Unhandled subject: {}", subject);
            }
        }

        info!("✅ [NOTIFY] Successfully processed event from: {}", subject);
        Ok(())
    }

    /// Handle chat-related events
    async fn handle_chat_event(&self, payload: Value) -> Result<(), NotifyError> {
        debug!("Processing chat event: {:?}", payload);

        // Extract event details
        let event_type = payload
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let chat_id = payload
            .get("chat_id")
            .and_then(|v| v.as_i64())
            .map(ChatId);

        let user_id = payload
            .get("user_id")
            .and_then(|v| v.as_i64())
            .map(UserId);

        match event_type {
            "member_added" => {
                if let (Some(chat_id), Some(user_id)) = (chat_id, user_id) {
                    self.handle_member_added(chat_id, user_id).await?;
                }
            }
            "member_removed" => {
                if let (Some(chat_id), Some(user_id)) = (chat_id, user_id) {
                    self.handle_member_removed(chat_id, user_id).await?;
                }
            }
            _ => {
                debug!("Unhandled chat event type: {}", event_type);
            }
        }

        Ok(())
    }

    /// Handle user-related events
    async fn handle_user_event(&self, payload: Value) -> Result<(), NotifyError> {
        debug!("Processing user event: {:?}", payload);

        let event_type = payload
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let user_id = payload
            .get("user_id")
            .and_then(|v| v.as_i64())
            .map(UserId);

        match event_type {
            "status_changed" => {
                if let Some(user_id) = user_id {
                    let status = payload
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    self.handle_user_status_changed(user_id, status).await?;
                }
            }
            _ => {
                debug!("Unhandled user event type: {}", event_type);
            }
        }

        Ok(())
    }

    /// Handle message-related events
    async fn handle_message_event(&self, payload: Value) -> Result<(), NotifyError> {
        debug!("Processing message event: {:?}", payload);

        let event_type = payload
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match event_type {
            "new_message" => {
                self.handle_new_message(payload).await?;
            }
            _ => {
                debug!("Unhandled message event type: {}", event_type);
            }
        }

        Ok(())
    }

    /// Handle member added to chat
    async fn handle_member_added(&self, chat_id: ChatId, user_id: UserId) -> Result<(), NotifyError> {
        info!("User {} added to chat {}", user_id.0, chat_id.0);

        // Update chat membership cache
        self.state.add_user_to_chat(user_id, chat_id).await;

        // Send notification to user
        let notification = json!({
            "type": "chat_member_added",
            "chat_id": chat_id.0,
            "user_id": user_id.0,
            "timestamp": Utc::now()
        });

        self.state
            .send_notification_to_user(user_id, notification)
            .await?;

        // Track analytics event
        let publisher = self.state.analytics_publisher();
        if publisher.is_enabled() {
            let event = NotifyEventHelper::notification_received(
                user_id,
                "chat_member_added".to_string(),
                "system".to_string(),
                true,
                None,
            );
            publisher.publish(event);
        }

        Ok(())
    }

    /// Handle member removed from chat
    async fn handle_member_removed(&self, chat_id: ChatId, user_id: UserId) -> Result<(), NotifyError> {
        info!("User {} removed from chat {}", user_id.0, chat_id.0);

        // Update chat membership cache
        self.state.remove_user_from_chat(user_id, chat_id).await;

        // Send notification to user
        let notification = json!({
            "type": "chat_member_removed",
            "chat_id": chat_id.0,
            "user_id": user_id.0,
            "timestamp": Utc::now()
        });

        self.state
            .send_notification_to_user(user_id, notification)
            .await?;

        // Track analytics event
        let publisher = self.state.analytics_publisher();
        if publisher.is_enabled() {
            let event = NotifyEventHelper::notification_received(
                user_id,
                "chat_member_removed".to_string(),
                "system".to_string(),
                true,
                None,
            );
            publisher.publish(event);
        }

        Ok(())
    }

    /// Handle user status change
    async fn handle_user_status_changed(&self, user_id: UserId, status: &str) -> Result<(), NotifyError> {
        info!("User {} status changed to: {}", user_id.0, status);

        // Update user status
        match status {
            "online" => {
                self.state.update_connection_status(user_id, ConnectionUpdate::Connected).await;
            }
            "offline" => {
                self.state.update_connection_status(user_id, ConnectionUpdate::Disconnected).await;
            }
            _ => {
                debug!("Unknown status: {}", status);
            }
        }

        Ok(())
    }

    /// Handle new message
    async fn handle_new_message(&self, payload: Value) -> Result<(), NotifyError> {
        let chat_id = payload
            .get("chat_id")
            .and_then(|v| v.as_i64())
            .map(ChatId);

        let sender_id = payload
            .get("sender_id")
            .and_then(|v| v.as_i64())
            .map(UserId);

        let content = payload
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if let (Some(chat_id), Some(sender_id)) = (chat_id, sender_id) {
            info!(
                "New message in chat {} from user {}: {}",
                chat_id.0, sender_id.0, content
            );

            // Get chat members
            let members = self.state.get_chat_members(chat_id).await.unwrap_or_default();

            // Send notification to all members except sender
            for member in members {
                if member != sender_id {
                    let notification = json!({
                        "type": "new_message",
                        "chat_id": chat_id.0,
                        "sender_id": sender_id.0,
                        "content": content,
                        "timestamp": Utc::now()
                    });

                    if let Err(e) = self
                        .state
                        .send_notification_to_user(member, notification)
                        .await
                    {
                        warn!("Failed to send notification to user {}: {}", member.0, e);
                    }

                    // Track analytics event for notification delivery
                    let publisher = self.state.analytics_publisher();
                    if publisher.is_enabled() {
                        let event = NotifyEventHelper::notification_received(
                            member,
                            "new_message".to_string(),
                            "chat".to_string(),
                            true,
                            None,
                        );                   publisher.publish(event);
                    }
                }
            }
        }

     
        Ok(())
    }

    /// Handle realtime events (typing, presence, etc.)
    async fn handle_realtime_event(&self, payload: Value) -> Result<(), NotifyError> {
        debug!("Processing realtime event: {:?}", payload);

        // 🔧 CRITICAL FIX: Handle MessageReceived events from fechatter-server
        // Check if this is a MessageReceived event (from fechatter-server enum format)
        if let Some(message_received_data) = payload.get("MessageReceived") {
            info!("📨 [REALTIME] Processing MessageReceived event");
            return self.handle_message_received_realtime(message_received_data).await;
        }

        // Handle standard event_type format
        let event_type = payload
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let chat_id = payload
            .get("chat_id")
            .and_then(|v| v.as_i64())
            .map(ChatId);

        let user_id = payload
            .get("user_id")
            .and_then(|v| v.as_i64())
            .map(UserId);

        match event_type {
            "typing_started" => {
                if let (Some(chat_id), Some(user_id)) = (chat_id, user_id) {
                    info!("⌨️ [NOTIFY] User {} started typing in chat {}", user_id.0, chat_id.0);
                    self.handle_typing_started(chat_id, user_id, payload).await?;
                }
            }
            "typing_stopped" => {
                if let (Some(chat_id), Some(user_id)) = (chat_id, user_id) {
                    info!("⏹️ [NOTIFY] User {} stopped typing in chat {}", user_id.0, chat_id.0);
                    self.handle_typing_stopped(chat_id, user_id).await?;
                }
            }
            "message_read" => {
                if let (Some(chat_id), Some(user_id)) = (chat_id, user_id) {
                    info!("👁️ [NOTIFY] User {} read messages in chat {}", user_id.0, chat_id.0);
                    self.handle_message_read(chat_id, user_id, payload).await?;
                }
            }
            "user_presence" => {
                if let Some(user_id) = user_id {
                    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
                    info!("🟢 [NOTIFY] User {} presence changed to: {}", user_id.0, status);
                    self.handle_user_presence(user_id, status).await?;
                }
            }
            _ => {
                debug!("Unhandled realtime event type: {}", event_type);
            }
        }

        Ok(())
    }

    /// 🔧 CRITICAL FIX: Handle MessageReceived events from fechatter-server
    async fn handle_message_received_realtime(&self, payload: &Value) -> Result<(), NotifyError> {
        let chat_id = payload.get("chat_id").and_then(|v| v.as_i64()).map(ChatId);
        let message = payload.get("message");
        let recipients = payload.get("recipients").and_then(|v| v.as_array());
        
        if let (Some(chat_id), Some(message), Some(recipients)) = (chat_id, message, recipients) {
            info!("📨 [REALTIME] Processing MessageReceived for chat {} with {} recipients", 
                  chat_id.0, recipients.len());
            
            // Extract message details
            let message_id = message.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let sender_id = message.get("sender_id").and_then(|v| v.as_i64()).map(UserId);
            let content = message.get("content").and_then(|v| v.as_str()).unwrap_or("");
            
            info!("📨 [REALTIME] Message {} from user {:?} in chat {}: {}", 
                  message_id, sender_id, chat_id.0, 
                  if content.len() > 50 { format!("{}...", &content[..50]) } else { content.to_string() });
            
            // 🚀 CRITICAL: Send SSE to ALL recipients INCLUDING the sender for message confirmation
            for recipient_value in recipients {
                if let Some(user_id) = recipient_value.as_i64() {
                    let user_id = UserId(user_id);
                    
                    let notification = json!({
                        "type": "new_message",
                        "chat_id": chat_id.0,
                        "message": message,
                        "timestamp": Utc::now(),
                        "realtime_source": "fechatter_server"
                    });
                    
                    if let Err(e) = self.state.send_notification_to_user(user_id, notification).await {
                        warn!("Failed to send SSE to user {}: {}", user_id.0, e);
                    } else {
                        info!("✅ [REALTIME] Sent SSE notification to user {} for message {}", user_id.0, message_id);
                    }
                }
            }
        } else {
            warn!("❌ [REALTIME] Invalid MessageReceived payload: missing required fields");
        }
        
        Ok(())
    }

    /// Handle typing started event
    async fn handle_typing_started(&self, chat_id: ChatId, user_id: UserId, payload: Value) -> Result<(), NotifyError> {
        let user_name = payload.get("user_name").and_then(|v| v.as_str()).unwrap_or("Unknown User");
        
        // Get chat members
        let members = self.state.get_chat_members(chat_id).await.unwrap_or_default();

        // Send typing notification to all members except the typer
        for member in members {
            if member != user_id {
                let notification = json!({
                    "type": "typing_started",
                    "chat_id": chat_id.0,
                    "user_id": user_id.0,
                    "user_name": user_name,
                    "timestamp": Utc::now()
                });

                if let Err(e) = self.state.send_notification_to_user(member, notification).await {
                    warn!("Failed to send typing notification to user {}: {}", member.0, e);
                }
            }
        }

        Ok(())
    }

    /// Handle typing stopped event
    async fn handle_typing_stopped(&self, chat_id: ChatId, user_id: UserId) -> Result<(), NotifyError> {
        // Get chat members
        let members = self.state.get_chat_members(chat_id).await.unwrap_or_default();

        // Send typing stopped notification to all members except the typer
        for member in members {
            if member != user_id {
                let notification = json!({
                    "type": "typing_stopped",
                    "chat_id": chat_id.0,
                    "user_id": user_id.0,
                    "timestamp": Utc::now()
                });

                if let Err(e) = self.state.send_notification_to_user(member, notification).await {
                    warn!("Failed to send typing stopped notification to user {}: {}", member.0, e);
                }
            }
        }

        Ok(())
    }

    /// Handle message read event
    async fn handle_message_read(&self, chat_id: ChatId, user_id: UserId, payload: Value) -> Result<(), NotifyError> {
        let message_id = payload.get("message_id").and_then(|v| v.as_i64());
        
        // Get chat members
        let members = self.state.get_chat_members(chat_id).await.unwrap_or_default();

        // Send read receipt to all members except the reader
        for member in members {
            if member != user_id {
                let notification = json!({
                    "type": "message_read",
                    "chat_id": chat_id.0,
                    "user_id": user_id.0,
                    "message_id": message_id,
                    "read_at": Utc::now()
                });

                if let Err(e) = self.state.send_notification_to_user(member, notification).await {
                    warn!("Failed to send read receipt to user {}: {}", member.0, e);
                }
            }
        }

        Ok(())
    }

    /// Handle user presence event
    async fn handle_user_presence(&self, user_id: UserId, status: &str) -> Result<(), NotifyError> {
        // Update user status in state
        match status {
            "online" => {
                self.state.update_connection_status(user_id, ConnectionUpdate::Connected).await;
            }
            "offline" => {
                self.state.update_connection_status(user_id, ConnectionUpdate::Disconnected).await;
            }
            _ => {
                debug!("Unknown presence status: {}", status);
            }
        }

        // Broadcast presence update to relevant users (could be optimized)
        let notification = json!({
            "type": "user_presence",
            "user_id": user_id.0,
            "status": status,
            "timestamp": Utc::now()
        });

        // For now, broadcast to all connected users
        // TODO: Optimize to only send to users who can see this user
        self.state.broadcast_to_all_users(notification).await?;

        Ok(())
    }
}

/// Handle system events (non-user specific)
pub async fn handle_system_event(
    state: Arc<AppState>,
    event_type: &str,
    payload: Value,
) -> Result<(), NotifyError> {
    debug!("Processing system event: {} - {:?}", event_type, payload);

    match event_type {
        "server_maintenance" => {
            // Broadcast maintenance notification to all connected users
            let notification = json!({
                "type": "system_maintenance",
                "message": payload.get("message").and_then(|v| v.as_str()).unwrap_or("Server maintenance scheduled"),
                "timestamp": Utc::now()
            });

            state.broadcast_to_all_users(notification).await?;

            // Track analytics event
            let publisher = state.analytics_publisher();
            if publisher.is_enabled() {
                let event = NotifyEventHelper::notification_received(
                    UserId(0), // System event
                    "system_maintenance".to_string(),
                    "system".to_string(),
                    true,
                    None,
                );
                publisher.publish(event);
            }
        }
        _ => {
            debug!("Unhandled system event type: {}", event_type);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::types::NotifyEventHelper;

    #[test]
    fn test_event_creation() {
        let user_id = UserId(123);
        let event = NotifyEventHelper::user_connected(
            user_id,
            "conn_123".to_string(),
            Some("test-agent".to_string()),
        );

        // Verify the event has the correct structure
        assert!(event.context.is_some());
        assert!(event.event_type.is_some());
    }

    #[test]
    fn test_notification_event_creation() {
        let user_id = UserId(456);
        let event = NotifyEventHelper::notification_received(
            user_id,
            "test_notification".to_string(),
            "test_source".to_string(),
            true,
            Some(100),
        );

        assert!(event.context.is_some());
        if let Some(context) = &event.context {
            assert_eq!(context.user_id, "456");
        }
    }

    #[test]
    fn test_user_disconnected_event() {
        let user_id = UserId(789);
        let event = NotifyEventHelper::user_disconnected(
            user_id,
            "conn_789".to_string(),
            5000,
        );

        assert!(event.context.is_some());
        assert!(event.event_type.is_some());
    }
}
