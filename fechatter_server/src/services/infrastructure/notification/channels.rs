// Notification channel implementations - Infrastructure layer

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

// Import from domains since we're now in infrastructure
use crate::config::NotificationConfig;
use crate::domains::notification::{Notification, NotificationChannel, NotificationChannelTrait};

// Email sending dependencies
use super::email_templates::{EmailTemplateData, EmailTemplateService};
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

/// Email notification channel implementation with full SMTP functionality
pub struct EmailChannelImpl {
    smtp_transport: AsyncSmtpTransport<Tokio1Executor>,
    template_service: EmailTemplateService,
    config: NotificationConfig,
    from_mailbox: Mailbox,
}

impl EmailChannelImpl {
    /// Create new email channel with SMTP configuration
    pub fn new(
        config: NotificationConfig,
        base_url: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Build SMTP transport
        let smtp_transport = Self::build_smtp_transport(&config)?;

        // Build template service (simplified)
        let template_service = EmailTemplateService::new_simple(base_url)
            .map_err(|e| format!("Failed to initialize email templates: {}", e))?;

        // Create from mailbox
        let from_mailbox = format!("{} <{}>", config.from_name, config.from_email)
            .parse()
            .map_err(|e| format!("Invalid from email address: {}", e))?;

        Ok(Self {
            smtp_transport,
            template_service,
            config,
            from_mailbox,
        })
    }

    /// Build SMTP transport from configuration
    fn build_smtp_transport(
        config: &NotificationConfig,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, Box<dyn std::error::Error + Send + Sync>> {
        let mut builder = if config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
        };

        // Configure authentication if credentials provided
        if !config.smtp_username.is_empty() && !config.smtp_password.is_empty() {
            let credentials =
                Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());
            builder = builder.credentials(credentials);
        }

        // Configure transport (pool configuration is automatic in lettre 0.11)
        let transport = builder
            .port(config.smtp_port)
            .timeout(Some(Duration::from_secs(30))) // Default timeout
            .build();

        Ok(transport)
    }

    /// Send email with retry logic and error handling
    async fn send_email_internal(&self, notification: &Notification) -> Result<(), String> {
        // Check if email notifications are enabled
        if !self.config.email_enabled {
            tracing::warn!("Email notifications are disabled in configuration");
            return Ok(()); // Return success but don't send
        }

        // Get user information (in real implementation, this would query user service)
        let (user_name, user_email) = self.get_user_info(notification.user_id.0).await?;

        // Render email template
        let template_data = self
            .template_service
            .render_notification_email(notification, &user_name, &user_email)
            .map_err(|e| format!("Template rendering failed: {}", e))?;

        // Build email message
        let email = self.build_email_message(&template_data)?;

        // Send email with retry logic
        self.send_with_retry(email, 3).await
    }

    /// Get user information (placeholder - would integrate with user service)
    async fn get_user_info(&self, user_id: i64) -> Result<(String, String), String> {
        // TODO: Integrate with actual user service
        // For now, return placeholder data
        tracing::warn!("Using placeholder user info for user_id: {}", user_id);
        Ok((
            format!("User{}", user_id),
            format!("user{}@example.com", user_id),
        ))
    }

    /// Build email message from template data
    fn build_email_message(&self, template_data: &EmailTemplateData) -> Result<Message, String> {
        let to_mailbox: Mailbox =
            format!("{} <{}>", template_data.user_name, template_data.user_email)
                .parse()
                .map_err(|e| format!("Invalid recipient email: {}", e))?;

        let mut message_builder = Message::builder()
            .from(self.from_mailbox.clone())
            .to(to_mailbox)
            .subject(&template_data.subject);

        // Add reply-to header if configured
        if let Some(reply_to) = &self.config.from_email.parse::<Mailbox>().ok() {
            message_builder = message_builder.reply_to(reply_to.clone());
        }

        // Build multipart message (text + HTML)
        let multipart = MultiPart::alternative()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(template_data.body_text.clone()),
            )
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(template_data.body_html.clone()),
            );

        let message = message_builder
            .multipart(multipart)
            .map_err(|e| format!("Failed to build email message: {}", e))?;

        Ok(message)
    }

    /// Send email with retry logic
    async fn send_with_retry(&self, message: Message, max_retries: u32) -> Result<(), String> {
        let mut last_error = String::new();

        for attempt in 1..=max_retries {
            match self.smtp_transport.send(message.clone()).await {
                Ok(response) => {
                    tracing::info!(
                        "Email sent successfully on attempt {}: {:?}",
                        attempt,
                        response
                    );
                    return Ok(());
                }
                Err(e) => {
                    last_error = format!("SMTP error on attempt {}: {}", attempt, e);
                    tracing::warn!("{}", last_error);

                    // Check if error is retryable
                    if !Self::is_retryable_error(&e) {
                        return Err(format!("Non-retryable SMTP error: {}", e));
                    }

                    // Wait before retry (exponential backoff)
                    if attempt < max_retries {
                        let delay = Duration::from_secs(2_u64.pow(attempt - 1));
                        tracing::info!("Retrying email send in {:?}", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(format!(
            "Failed to send email after {} attempts: {}",
            max_retries, last_error
        ))
    }

    /// Check if SMTP error is retryable
    fn is_retryable_error(error: &lettre::transport::smtp::Error) -> bool {
        // lettre 0.11 使用字符串描述错误，需要检查错误消息
        let error_string = error.to_string().to_lowercase();

        // 检查是否是网络相关的可重试错误
        error_string.contains("connection")
            || error_string.contains("timeout")
            || error_string.contains("io error")
            || error_string.contains("network")
            || error_string.contains("temporary")
            || error_string.contains("try again")
    }

    /// Health check for SMTP connection
    pub async fn health_check(&self) -> Result<(), String> {
        match self.smtp_transport.test_connection().await {
            Ok(is_connected) => {
                if is_connected {
                    Ok(())
                } else {
                    Err("SMTP connection test failed".to_string())
                }
            }
            Err(e) => Err(format!("SMTP health check failed: {}", e)),
        }
    }
}

impl NotificationChannelTrait for EmailChannelImpl {
    fn send_notification(
        &self,
        notification: Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            tracing::info!(
                "Sending email notification to user {}: {}",
                notification.user_id.0,
                notification.content.message
            );

            // 现在notification是所有权而不是引用，但我们需要一个引用来传递给send_email_internal
            // 所以这里使用&notification
            self.send_email_internal(&notification).await?;

            tracing::info!(
                "Email notification sent successfully to user {}",
                notification.user_id.0
            );

            Ok(())
        })
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Email
    }
}

/// Push notification channel implementation
pub struct PushChannelImpl;

impl NotificationChannelTrait for PushChannelImpl {
    fn send_notification(
        &self,
        notification: Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            // TODO: Implement actual push notification logic
            // - Connect to FCM/APNS
            // - Format push payload
            // - Send push notification

            tracing::info!(
                "Sending push notification to user {}: {}",
                notification.user_id.0,
                notification.content.message
            );

            // Placeholder implementation
            Ok(())
        })
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Push
    }
}

/// In-app notification channel implementation - Primary notification method
pub struct InAppChannelImpl {
    // TODO: Add database connection for storing notifications
    // notification_repo: Arc<dyn NotificationRepository>,
}

impl InAppChannelImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl NotificationChannelTrait for InAppChannelImpl {
    fn send_notification(
        &self,
        notification: Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            // TODO: Implement actual database storage
            // - Connect to notification repository
            // - Store notification in database
            // - Track delivery status

            tracing::info!(
                "Storing in-app notification for user {}: {}",
                notification.user_id.0,
                notification.content.message
            );

            // For now, we'll simulate success
            // In a real implementation:
            // 1. Store notification in database
            // 2. Mark as unread
            // 3. Update user's notification count
            // 4. Trigger real-time event if user is online

            Ok(())
        })
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Database
    }
}

/// WebSocket notification channel implementation
pub struct WebSocketChannelImpl {
    // TODO: Add WebSocket connection manager
    // websocket_manager: Arc<WebSocketManager>,
}

impl WebSocketChannelImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl NotificationChannelTrait for WebSocketChannelImpl {
    fn send_notification(
        &self,
        notification: Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            // TODO: Implement actual WebSocket notification
            // - Find active WebSocket connections for user
            // - Format WebSocket message
            // - Send through active connections

            tracing::info!(
                "Sending WebSocket notification to user {}: {}",
                notification.user_id.0,
                notification.content.message
            );

            // Placeholder implementation
            // In a real implementation:
            // 1. Look up user's active WebSocket connections
            // 2. Format notification as WebSocket message
            // 3. Send to all active connections
            // 4. Handle offline users gracefully

            Ok(())
        })
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::WebSocket
    }
}

/// Notification channel factory - Creates channel implementations
pub struct NotificationChannelFactory;

impl NotificationChannelFactory {
    /// Create a channel implementation by type
    /// TODO: This should be updated to accept configuration parameters
    pub fn create_channel(channel_type: NotificationChannel) -> Arc<dyn NotificationChannelTrait> {
        match channel_type {
            NotificationChannel::Email => {
                // Use placeholder email channel since we don't have configuration here
                tracing::warn!(
          "Creating placeholder email channel - configuration required for real implementation"
        );
                Arc::new(PlaceholderEmailChannel)
            }
            NotificationChannel::Push => Arc::new(PushChannelImpl),
            NotificationChannel::Database => Arc::new(InAppChannelImpl::new()),
            NotificationChannel::WebSocket => Arc::new(WebSocketChannelImpl::new()),
        }
    }

    /// Create a channel implementation with configuration
    pub fn create_email_channel(
        config: NotificationConfig,
        base_url: String,
    ) -> Result<Arc<dyn NotificationChannelTrait>, Box<dyn std::error::Error + Send + Sync>> {
        let email_impl = EmailChannelImpl::new(config, base_url)?;
        Ok(Arc::new(email_impl))
    }

    /// Create all available channels with default configuration
    pub fn create_all_channels() -> Vec<Arc<dyn NotificationChannelTrait>> {
        vec![
            Self::create_channel(NotificationChannel::Email),
            Arc::new(PushChannelImpl),
            Arc::new(InAppChannelImpl::new()),
            Arc::new(WebSocketChannelImpl::new()),
        ]
    }
}

/// Placeholder email channel for fallback when configuration fails
struct PlaceholderEmailChannel;

impl NotificationChannelTrait for PlaceholderEmailChannel {
    fn send_notification(
        &self,
        notification: Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            tracing::warn!(
        "Placeholder email channel: Email notification for user {} not sent (configuration failed)",
        notification.user_id.0
      );
            Ok(())
        })
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Email
    }
}

/// Notification delivery service - Orchestrates multiple channels
pub struct NotificationDeliveryService {
    channels: Vec<Arc<dyn NotificationChannelTrait>>,
}

impl NotificationDeliveryService {
    pub fn new(channels: Vec<Arc<dyn NotificationChannelTrait>>) -> Self {
        Self { channels }
    }

    /// Send notification through specified channel
    pub async fn send_via_channel(
        &self,
        notification: &Notification,
        channel_type: NotificationChannel,
    ) -> Result<(), String> {
        let channel = self
            .channels
            .iter()
            .find(|c| c.channel_type() == channel_type)
            .ok_or_else(|| format!("Channel {:?} not found", channel_type))?;

        channel.send_notification(notification.clone()).await
    }

    /// Send notification through multiple channels
    pub async fn send_via_multiple_channels(
        &self,
        notification: &Notification,
        channel_types: &[NotificationChannel],
    ) -> Vec<Result<(), String>> {
        let mut results = Vec::new();

        for channel_type in channel_types {
            let result = self
                .send_via_channel(notification, channel_type.clone())
                .await;
            results.push(result);
        }

        results
    }

    /// Send notification through all configured channels
    pub async fn broadcast(&self, notification: &Notification) -> Vec<Result<(), String>> {
        let mut results = Vec::new();

        for channel in &self.channels {
            let result = channel.send_notification(notification.clone()).await;
            results.push(result);
        }

        results
    }
}
