use crate::config::EmailTemplateConfig;
use crate::domains::notification::{Notification, NotificationContent, NotificationType};
/// Email template system for Fechatter notification system
///
/// This module provides template management and rendering for email notifications.
/// Templates are rendered using Handlebars with support for dynamic data injection.
use handlebars::{Handlebars, HelperResult, JsonRender, Output, RenderContext, RenderError};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Email template data for rendering
#[derive(Debug, Clone)]
pub struct EmailTemplateData {
  pub user_name: String,
  pub user_email: String,
  pub subject: String,
  pub body_text: String,
  pub body_html: String,
  pub action_url: Option<String>,
  pub metadata: HashMap<String, String>,
}

/// Email template service
pub struct EmailTemplateService {
  handlebars: Handlebars<'static>,
  config: EmailTemplateConfig,
  base_url: String,
}

impl EmailTemplateService {
  /// Create new email template service
  pub fn new(config: EmailTemplateConfig, base_url: String) -> Result<Self, RenderError> {
    let mut handlebars = Handlebars::new();

    // Register built-in templates
    Self::register_templates(&mut handlebars)?;

    // Register custom helpers
    Self::register_helpers(&mut handlebars);

    Ok(Self {
      handlebars,
      config,
      base_url,
    })
  }

  /// Render email template for notification
  pub fn render_notification_email(
    &self,
    notification: &Notification,
    user_name: &str,
    user_email: &str,
  ) -> Result<EmailTemplateData, RenderError> {
    let template_name = self.get_template_name(&notification.notification_type);
    let context = self.build_template_context(notification, user_name, user_email);

    let subject = self
      .handlebars
      .render(&format!("{}_subject", template_name), &context)?;

    let body_text = self
      .handlebars
      .render(&format!("{}_text", template_name), &context)?;

    let body_html = self
      .handlebars
      .render(&format!("{}_html", template_name), &context)?;

    let action_url = self.build_action_url(notification);

    Ok(EmailTemplateData {
      user_name: user_name.to_string(),
      user_email: user_email.to_string(),
      subject,
      body_text,
      body_html,
      action_url,
      metadata: HashMap::new(),
    })
  }

  /// Get template name for notification type
  fn get_template_name(&self, notification_type: &NotificationType) -> &str {
    match notification_type {
      NotificationType::Mention => &self.config.mention_template,
      NotificationType::DirectMessage => &self.config.direct_message_template,
      NotificationType::ChatInvite => &self.config.chat_invite_template,
      NotificationType::WorkspaceInvite => &self.config.workspace_invite_template,
      NotificationType::System => "system_notification",
    }
  }

  /// Build template context from notification
  fn build_template_context(
    &self,
    notification: &Notification,
    user_name: &str,
    user_email: &str,
  ) -> Value {
    let mut context = json!({
        "user_name": user_name,
        "user_email": user_email,
        "notification_title": notification.title,
        "notification_message": notification.content.message,
        "base_url": self.base_url,
        "timestamp": notification.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        "notification_type": format!("{:?}", notification.notification_type),
        "priority": format!("{:?}", notification.priority),
    });

    // Add chat-specific context
    if let Some(chat_id) = notification.content.chat_id {
      context["chat_id"] = json!(chat_id.0);
      context["chat_url"] = json!(format!("{}/chat/{}", self.base_url, chat_id.0));
    }

    // Add sender-specific context
    if let Some(sender_id) = notification.content.sender_id {
      context["sender_id"] = json!(sender_id.0);
    }

    // Add metadata
    if let Some(metadata) = &notification.content.metadata {
      context["metadata"] = json!(metadata);
    }

    context
  }

  /// Build action URL for notification
  fn build_action_url(&self, notification: &Notification) -> Option<String> {
    match &notification.notification_type {
      NotificationType::Mention | NotificationType::DirectMessage => notification
        .content
        .chat_id
        .map(|chat_id| format!("{}/chat/{}", self.base_url, chat_id.0)),
      NotificationType::ChatInvite => notification
        .content
        .chat_id
        .map(|chat_id| format!("{}/chat/{}/join", self.base_url, chat_id.0)),
      NotificationType::WorkspaceInvite => Some(format!("{}/accept-invite", self.base_url)),
      NotificationType::System => None,
    }
  }

  /// Register built-in email templates
  fn register_templates(handlebars: &mut Handlebars) -> Result<(), RenderError> {
    // Mention notification templates
    handlebars.register_template_string(
      "mention_notification_subject",
      "You were mentioned in {{notification_title}}",
    )?;

    handlebars.register_template_string(
      "mention_notification_text",
      include_str!("templates/mention_notification.txt"),
    )?;

    handlebars.register_template_string(
      "mention_notification_html",
      include_str!("templates/mention_notification.html"),
    )?;

    // Direct message notification templates
    handlebars.register_template_string(
      "direct_message_notification_subject",
      "New message: {{notification_title}}",
    )?;

    handlebars.register_template_string(
      "direct_message_notification_text",
      include_str!("templates/direct_message_notification.txt"),
    )?;

    handlebars.register_template_string(
      "direct_message_notification_html",
      include_str!("templates/direct_message_notification.html"),
    )?;

    // Chat invite notification templates
    handlebars.register_template_string(
      "chat_invite_notification_subject",
      "You've been invited to {{notification_title}}",
    )?;

    handlebars.register_template_string(
      "chat_invite_notification_text",
      include_str!("templates/chat_invite_notification.txt"),
    )?;

    handlebars.register_template_string(
      "chat_invite_notification_html",
      include_str!("templates/chat_invite_notification.html"),
    )?;

    // Workspace invite notification templates
    handlebars.register_template_string(
      "workspace_invite_notification_subject",
      "Welcome to {{notification_title}}",
    )?;

    handlebars.register_template_string(
      "workspace_invite_notification_text",
      include_str!("templates/workspace_invite_notification.txt"),
    )?;

    handlebars.register_template_string(
      "workspace_invite_notification_html",
      include_str!("templates/workspace_invite_notification.html"),
    )?;

    // System notification templates
    handlebars.register_template_string(
      "system_notification_subject",
      "System: {{notification_title}}",
    )?;

    handlebars.register_template_string(
      "system_notification_text",
      include_str!("templates/system_notification.txt"),
    )?;

    handlebars.register_template_string(
      "system_notification_html",
      include_str!("templates/system_notification.html"),
    )?;

    Ok(())
  }

  /// Register custom Handlebars helpers
  fn register_helpers(handlebars: &mut Handlebars) {
    // Helper for formatting dates
    handlebars.register_helper(
      "format_date",
      Box::new(
        |h: &handlebars::Helper,
         _: &Handlebars,
         _: &handlebars::Context,
         _: &mut RenderContext,
         out: &mut dyn Output|
         -> HelperResult {
          let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("format_date helper requires a parameter"))?;

          let formatted = match param.value().as_str() {
            Some(date_str) => {
              // Basic date formatting - can be enhanced with chrono
              date_str.to_string()
            }
            None => "Invalid date".to_string(),
          };

          out.write(&formatted)?;
          Ok(())
        },
      ),
    );

    // Helper for truncating text
    handlebars.register_helper(
      "truncate",
      Box::new(
        |h: &handlebars::Helper,
         _: &Handlebars,
         _: &handlebars::Context,
         _: &mut RenderContext,
         out: &mut dyn Output|
         -> HelperResult {
          let text = h
            .param(0)
            .ok_or_else(|| RenderError::new("truncate helper requires text parameter"))?
            .value()
            .as_str()
            .unwrap_or("");

          let length = h.param(1).and_then(|p| p.value().as_u64()).unwrap_or(100) as usize;

          let truncated = if text.len() > length {
            format!("{}...", &text[..length])
          } else {
            text.to_string()
          };

          out.write(&truncated)?;
          Ok(())
        },
      ),
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use fechatter_core::{ChatId, UserId};

  #[test]
  fn test_email_template_service_creation() {
    let config = EmailTemplateConfig::default();
    let base_url = "https://fechatter.com".to_string();

    let service = EmailTemplateService::new(config, base_url);
    assert!(service.is_ok());
  }

  #[test]
  fn test_template_context_building() {
    let config = EmailTemplateConfig::default();
    let base_url = "https://fechatter.com".to_string();
    let service = EmailTemplateService::new(config, base_url).unwrap();

    let notification = Notification {
      id: 1,
      user_id: UserId::new(123),
      notification_type: NotificationType::Mention,
      title: "Test Mention".to_string(),
      content: NotificationContent {
        message: "You were mentioned in a message".to_string(),
        chat_id: Some(ChatId::new(456)),
        sender_id: Some(UserId::new(789)),
        metadata: Some("test_metadata".to_string()),
      },
      priority: crate::domains::notification::NotificationPriority::Normal,
      is_read: false,
      created_at: Utc::now(),
      read_at: None,
    };

    let context = service.build_template_context(&notification, "John Doe", "john@example.com");

    assert_eq!(context["user_name"], "John Doe");
    assert_eq!(context["user_email"], "john@example.com");
    assert_eq!(context["notification_title"], "Test Mention");
    assert_eq!(context["chat_id"], 456);
  }

  #[test]
  fn test_action_url_building() {
    let config = EmailTemplateConfig::default();
    let base_url = "https://fechatter.com".to_string();
    let service = EmailTemplateService::new(config, base_url).unwrap();

    let notification = Notification {
      id: 1,
      user_id: UserId::new(123),
      notification_type: NotificationType::Mention,
      title: "Test".to_string(),
      content: NotificationContent {
        message: "Test message".to_string(),
        chat_id: Some(ChatId::new(456)),
        sender_id: None,
        metadata: None,
      },
      priority: crate::domains::notification::NotificationPriority::Normal,
      is_read: false,
      created_at: Utc::now(),
      read_at: None,
    };

    let action_url = service.build_action_url(&notification);
    assert_eq!(
      action_url,
      Some("https://fechatter.com/chat/456".to_string())
    );
  }
}
