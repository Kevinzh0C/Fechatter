//! # Typing Indicator Service
//!
//! Uses in-memory cache to manage typing status to avoid database hotspots
//! TTL: 10 seconds, frontend sends heartbeat every 3 seconds

use crate::error::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct TypingUser {
  pub user_id: i64,
  pub user_name: String,
  pub started_at: Instant,
  pub expires_at: Instant,
}

/// Typing indicator service - In-memory management for high performance
pub struct TypingIndicatorService {
  /// chat_id -> typing users
  typing_users: Arc<RwLock<HashMap<i64, HashMap<i64, TypingUser>>>>,
  ttl: Duration,
}

impl TypingIndicatorService {
  pub fn new() -> Self {
    Self {
      typing_users: Arc::new(RwLock::new(HashMap::new())),
      ttl: Duration::from_secs(10),
    }
  }

  /// Set user typing status
  pub async fn set_typing(
    &self,
    chat_id: i64,
    user_id: i64,
    user_name: String,
    is_typing: bool,
  ) -> Result<(), AppError> {
    let mut typing_map = self.typing_users.write().await;

    if is_typing {
      let now = Instant::now();
      let typing_user = TypingUser {
        user_id,
        user_name,
        started_at: now,
        expires_at: now + self.ttl,
      };

      typing_map
        .entry(chat_id)
        .or_insert_with(HashMap::new)
        .insert(user_id, typing_user);

      debug!("User {} started typing in chat {}", user_id, chat_id);
    } else {
      if let Some(chat_users) = typing_map.get_mut(&chat_id) {
        chat_users.remove(&user_id);
        if chat_users.is_empty() {
          typing_map.remove(&chat_id);
        }
      }
      debug!("User {} stopped typing in chat {}", user_id, chat_id);
    }

    Ok(())
  }

  /// Get typing users for a chat (excluding expired)
  pub async fn get_typing_users(&self, chat_id: i64) -> Vec<TypingUser> {
    let mut typing_map = self.typing_users.write().await;
    let now = Instant::now();

    // Clean up expired entries
    if let Some(chat_users) = typing_map.get_mut(&chat_id) {
      chat_users.retain(|_, user| user.expires_at > now);

      if chat_users.is_empty() {
        typing_map.remove(&chat_id);
        Vec::new()
      } else {
        chat_users.values().cloned().collect()
      }
    } else {
      Vec::new()
    }
  }

  /// Clean up all expired typing indicators
  pub async fn cleanup_expired(&self) {
    let mut typing_map = self.typing_users.write().await;
    let now = Instant::now();

    typing_map.retain(|_, chat_users| {
      chat_users.retain(|_, user| user.expires_at > now);
      !chat_users.is_empty()
    });
  }

  /// Start background cleanup task
  pub fn start_cleanup_task(self: Arc<Self>) {
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_secs(30));
      loop {
        interval.tick().await;
        self.cleanup_expired().await;
      }
    });
  }
}

/// Factory functions
pub fn create_typing_indicator_service() -> Arc<TypingIndicatorService> {
  let service = Arc::new(TypingIndicatorService::new());
  // Start background cleanup
  service.clone().start_cleanup_task();
  service
}
