use chrono::{DateTime, Utc};
use fechatter_core::{error::CoreError, ChatId, UserId};
use serde::{Deserialize, Serialize};

/// Notification entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub user_id: UserId,
    pub notification_type: NotificationType,
    pub title: String,
    pub content: NotificationContent,
    pub priority: NotificationPriority,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

impl Notification {
    /// Create mention notification
    pub fn create_mention(
        user_id: i64,
        mentioned_by_name: &str,
        message_preview: &str,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Self, CoreError> {
        Ok(Self {
            id: 0, // Will be set by repository
            user_id: UserId::new(user_id),
            notification_type: NotificationType::Mention,
            title: format!("Mentioned by {}", mentioned_by_name),
            content: NotificationContent {
                message: message_preview.to_string(),
                chat_id: Some(ChatId::new(chat_id)),
                sender_id: None,
                metadata: Some(message_id.to_string()),
            },
            priority: NotificationPriority::Normal,
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        })
    }

    /// Create direct message notification
    pub fn create_direct_message(
        recipient_id: i64,
        sender_name: &str,
        message_preview: &str,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Self, CoreError> {
        Ok(Self {
            id: 0, // Will be set by repository
            user_id: UserId::new(recipient_id),
            notification_type: NotificationType::DirectMessage,
            title: format!("Message from {}", sender_name),
            content: NotificationContent {
                message: message_preview.to_string(),
                chat_id: Some(ChatId::new(chat_id)),
                sender_id: None,
                metadata: Some(message_id.to_string()),
            },
            priority: NotificationPriority::Normal,
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        })
    }

    /// Mark notification as read
    pub fn mark_as_read(&mut self) -> Result<(), CoreError> {
        if !self.is_read {
            self.is_read = true;
            self.read_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Check if notification is expired (older than 30 days)
    pub fn is_expired(&self) -> bool {
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
        self.created_at < thirty_days_ago
    }

    /// Check if notification can be deleted
    pub fn can_be_deleted(&self) -> bool {
        // Business rule: read notifications or expired notifications can be deleted
        self.is_read || self.is_expired()
    }

    /// Apply user preferences to notification
    pub fn apply_user_preferences(&mut self, _preferences: &UserNotificationPreferences) {
        // TODO: Implement preference application logic
        // For now, just keep the notification as-is
    }
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Mention,
    DirectMessage,
    ChatInvite,
    WorkspaceInvite,
    System,
}

/// Notification content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationContent {
    pub message: String,
    pub chat_id: Option<ChatId>,
    pub sender_id: Option<UserId>,
    pub metadata: Option<String>,
}

/// Notification priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationPreferences {
    pub user_id: UserId,
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub mention_notifications: bool,
    pub direct_message_notifications: bool,
    pub chat_invite_notifications: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserNotificationPreferences {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            user_id: UserId::new(0), // Will be overridden
            email_notifications: true,
            push_notifications: true,
            mention_notifications: true,
            direct_message_notifications: true,
            chat_invite_notifications: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Notification channel
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationChannel {
    WebSocket,
    Email,
    Push,
    Database,
}

/// Notification channel trait - fix dyn compatibility
pub trait NotificationChannelTrait: Send + Sync {
    /// Send notification through this channel
    fn send_notification(
        &self,
        notification: Notification,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;

    /// Get the channel type
    fn channel_type(&self) -> NotificationChannel;
}
