// Infrastructure layer for notification - Concrete implementations
pub mod channels;
pub mod email_templates;

// Re-export for easy access
pub use channels::{
  EmailChannelImpl, InAppChannelImpl, NotificationChannelFactory, NotificationDeliveryService,
  PushChannelImpl, WebSocketChannelImpl,
};
pub use email_templates::{EmailTemplateData, EmailTemplateService};
