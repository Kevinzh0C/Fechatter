use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, instrument};

use crate::domains::notification::{
  Notification, NotificationRepository, UserNotificationPreferences,
};
use fechatter_core::{UserId, error::CoreError};

/// Notification Application Service - Use Case Coordination Layer
///
/// Responsibilities:
/// 1. Coordinate Domain objects to execute business use cases
/// 2. Handle transactions and errors
/// 3. Call infrastructure services
pub struct NotificationAppService {
  notification_repo: Arc<dyn NotificationRepository>,
}

impl NotificationAppService {
  pub fn new(notification_repo: Arc<dyn NotificationRepository>) -> Self {
    Self { notification_repo }
  }

  /// Use Case: Send @ mention notification
  #[instrument(skip(self))]
  pub async fn send_mention_notification(
    &self,
    mentioned_user_ids: Vec<i64>,
    mentioned_by_name: &str,
    message_preview: &str,
    chat_id: i64,
    message_id: i64,
  ) -> Result<Vec<i64>, CoreError> {
    if mentioned_user_ids.is_empty() {
      return Ok(Vec::new());
    }

    info!(
      mentioned_count = mentioned_user_ids.len(),
      chat_id = chat_id,
      "Processing mention notifications"
    );

    let mut notification_ids = Vec::new();

    for user_id in mentioned_user_ids {
      // Create notification using Domain entity's business method
      let mut notification = Notification::create_mention(
        user_id,
        mentioned_by_name,
        message_preview,
        chat_id,
        message_id,
      )?;

      // Apply user preferences
      let preferences = self.get_user_preferences(user_id).await?;
      notification.apply_user_preferences(&preferences);

      // Save to database
      let notification_id = self
        .notification_repo
        .save_notification(&notification)
        .await?;

      notification_ids.push(notification_id);
    }

    info!(
      sent_count = notification_ids.len(),
      "Mention notifications created"
    );
    Ok(notification_ids)
  }

  /// Use Case: Send direct message notification
  #[instrument(skip(self))]
  pub async fn send_direct_message_notification(
    &self,
    recipient_id: i64,
    sender_id: i64,
    sender_name: &str,
    message_preview: &str,
    chat_id: i64,
    message_id: i64,
  ) -> Result<Option<i64>, CoreError> {
    // Business rule: Don't send notification to self
    if recipient_id == sender_id {
      return Ok(None);
    }

    info!(
      recipient = recipient_id,
      sender = sender_id,
      "Processing DM notification"
    );

    // Create notification using Domain entity
    let mut notification = Notification::create_direct_message(
      recipient_id,
      sender_name,
      message_preview,
      chat_id,
      message_id,
    )?;

    // Apply user preferences
    let preferences = self.get_user_preferences(recipient_id).await?;
    notification.apply_user_preferences(&preferences);

    // Save
    let notification_id = self
      .notification_repo
      .save_notification(&notification)
      .await?;

    info!(notification_id, "DM notification created");
    Ok(Some(notification_id))
  }

  /// Use Case: Mark notification as read
  #[instrument(skip(self))]
  pub async fn mark_notification_as_read(
    &self,
    notification_id: i64,
    user_id: i64,
  ) -> Result<(), CoreError> {
    // Get notification from repository
    let mut notification = self
      .notification_repo
      .find_by_id_and_user(notification_id, user_id)
      .await?
      .ok_or_else(|| CoreError::NotFound("Notification not found".into()))?;

    // Use Domain entity's business method
    notification.mark_as_read()?;

    // Update to database
    self
      .notification_repo
      .update_notification(&notification)
      .await?;

    info!(notification_id, user_id, "Notification marked as read");
    Ok(())
  }

  /// Use Case: Get user's unread notifications
  #[instrument(skip(self))]
  pub async fn get_unread_notifications(
    &self,
    user_id: i64,
    limit: i32,
  ) -> Result<Vec<Notification>, CoreError> {
    let notifications = self
      .notification_repo
      .get_unread_by_user(user_id, limit)
      .await?;

    // Filter out expired notifications
    let active_notifications: Vec<Notification> = notifications
      .into_iter()
      .filter(|n| !n.is_expired())
      .collect();

    Ok(active_notifications)
  }

  /// Use Case: Delete notification
  #[instrument(skip(self))]
  pub async fn delete_notification(
    &self,
    notification_id: i64,
    user_id: i64,
  ) -> Result<(), CoreError> {
    // Get notification
    let notification = self
      .notification_repo
      .find_by_id_and_user(notification_id, user_id)
      .await?
      .ok_or_else(|| CoreError::NotFound("Notification not found".into()))?;

    // Check business rules
    if !notification.can_be_deleted() {
      return Err(CoreError::ValidationError(
        "This notification cannot be deleted".into(),
      ));
    }

    // Execute deletion
    self
      .notification_repo
      .delete_notification(notification_id, user_id)
      .await?;

    info!(notification_id, user_id, "Notification deleted");
    Ok(())
  }

  // Private helper method
  async fn get_user_preferences(
    &self,
    user_id: i64,
  ) -> Result<UserNotificationPreferences, CoreError> {
    // TODO: Get real preferences from user settings repository
    // Return default settings for now
    Ok(UserNotificationPreferences {
      user_id: UserId::new(user_id),
      ..Default::default()
    })
  }
}

/// Type alias for backward compatibility
pub type NotificationService = NotificationAppService;
