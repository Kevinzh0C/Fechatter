pub mod entities;
pub mod repository;

// Re-export commonly used types
pub use entities::{
    Notification, NotificationChannel, NotificationChannelTrait, NotificationContent,
    NotificationPriority, NotificationType, UserNotificationPreferences,
};
pub use repository::NotificationRepository;
