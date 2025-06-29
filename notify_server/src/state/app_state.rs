use anyhow::Result;
use dashmap::DashMap;
use std::{collections::HashSet, ops::Deref, sync::Arc};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::{
  analytics::AnalyticsPublisher,
  config::AppConfig,
  connections::manager::{ConnectionManager, ConnectionStats},
  error::NotifyError,
  events::types::NotifyEvent,
};
use fechatter_core::{ChatId, ErrorMapper, TokenManager, TokenVerifier, UserClaims, UserId};

type UserConnections = Arc<DashMap<UserId, broadcast::Sender<Arc<NotifyEvent>>>>;
type ChatMembers = Arc<DashMap<ChatId, HashSet<UserId>>>;
type UserChats = Arc<DashMap<UserId, HashSet<ChatId>>>;

#[derive(Clone)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
  pub config: AppConfig,
  pub user_connections: UserConnections,
  pub chat_members: ChatMembers,
  pub user_chats: UserChats,
  pub connection_manager: ConnectionManager,
  pub analytics: AnalyticsPublisher,
  token_manager: TokenManager,
}

impl TokenVerifier for AppState {
  type Claims = UserClaims;
  type Error = NotifyError;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    debug!("🔍 [NOTIFY] Starting JWT token verification");
    debug!("🔍 [NOTIFY] Token length: {} chars", token.len());
    debug!("🔍 [NOTIFY] Token preview: {}...", &token[..std::cmp::min(30, token.len())]);
    
    match self
      .inner
      .token_manager
      .verify_token(token)
    {
      Ok(claims) => {
        debug!("✅ [NOTIFY] JWT verification SUCCESS for user {}", claims.id.0);
        debug!("🔍 [NOTIFY] Claims: email={}, workspace_id={}", claims.email, claims.workspace_id.0);
        Ok(claims)
      }
      Err(e) => {
        warn!("❌ [NOTIFY] JWT verification FAILED: {:?}", e);
        warn!("🔍 [NOTIFY] Token that failed: {}...", &token[..std::cmp::min(50, token.len())]);
        Err(NotifyError::map_error(e))
      }
    }
  }
}

impl Deref for AppState {
  type Target = AppStateInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl AppState {
  pub fn new(config: AppConfig) -> Result<Self, anyhow::Error> {
    let user_connections = Arc::new(DashMap::new());
    let chat_members = Arc::new(DashMap::new());
    let user_chats = Arc::new(DashMap::new());
    let connection_manager = ConnectionManager::new();
    let token_manager = TokenManager::new(&config.auth)?;
    
    // Create a disabled analytics publisher initially
    // Will be initialized properly in try_new_async()
    let analytics = AnalyticsPublisher::default();

    Ok(Self {
      inner: Arc::new(AppStateInner {
        config,
        user_connections,
        chat_members,
        user_chats,
        connection_manager,
        analytics,
        token_manager,
      }),
    })
  }

  /// Async constructor that properly initializes analytics
  pub async fn try_new_async(config: AppConfig) -> Result<Self, anyhow::Error> {
    let user_connections = Arc::new(DashMap::new());
    let chat_members = Arc::new(DashMap::new());
    let user_chats = Arc::new(DashMap::new());
    let connection_manager = ConnectionManager::new();
    let token_manager = TokenManager::new(&config.auth)?;
    
    // Initialize analytics publisher with proper config
    let analytics = AnalyticsPublisher::new(config.analytics.clone()).await?;
    info!("📊 Analytics publisher initialized: enabled={}", analytics.is_enabled());

    Ok(Self {
      inner: Arc::new(AppStateInner {
        config,
        user_connections,
        chat_members,
        user_chats,
        connection_manager,
        analytics,
        token_manager,
      }),
    })
  }

  /// Get application config
  pub fn config(&self) -> &AppConfig {
    &self.config
  }

  /// Get connection manager
  pub fn connection_manager(&self) -> &ConnectionManager {
    &self.connection_manager
  }

  /// Get connection statistics
  pub fn get_connection_stats(&self) -> ConnectionStats {
    self.connection_manager.get_stats()
  }

  /// Get all chats a user is in (from database)
  pub async fn get_user_chats(&self, user_id: UserId) -> Result<HashSet<ChatId>, anyhow::Error> {
    use sqlx::Row;

    let pool = sqlx::PgPool::connect(&self.config.server.db_url).await?;

    let rows =
      sqlx::query("SELECT chat_id FROM chat_members WHERE user_id = $1 AND left_at IS NULL")
        .bind(user_id.0 as i64)
        .fetch_all(&pool)
        .await?;

    let chat_ids: HashSet<ChatId> = rows
      .iter()
      .map(|row| ChatId(row.get::<i64, _>("chat_id") as i64))
      .collect();

    debug!(
      "📊 User {} is a member of {} chats",
      user_id.0,
      chat_ids.len()
    );
    Ok(chat_ids)
  }

  /// Get the number of chats a user is in (from database)
  pub async fn get_user_chat_count(&self, user_id: UserId) -> Result<usize, anyhow::Error> {
    use sqlx::Row;

    let pool = sqlx::PgPool::connect(&self.config.server.db_url).await?;

    let row = sqlx::query("SELECT COUNT(*) as count FROM chat_members WHERE user_id = $1 AND left_at IS NULL")
        .bind(user_id.0 as i64)
        .fetch_one(&pool)
        .await?;

    let count = row.get::<i64, _>("count") as usize;
    debug!("📊 User {} is a member of {} chats", user_id.0, count);
    Ok(count)
  }

  /// Get all members of a chat (from database)
  pub async fn get_chat_members(&self, chat_id: ChatId) -> Result<HashSet<UserId>, anyhow::Error> {
    use sqlx::Row;

    let pool = sqlx::PgPool::connect(&self.config.server.db_url).await?;

    let rows =
      sqlx::query("SELECT user_id FROM chat_members WHERE chat_id = $1 AND left_at IS NULL")
        .bind(chat_id.0 as i64)
        .fetch_all(&pool)
        .await?;

    let user_ids: HashSet<UserId> = rows
      .iter()
      .map(|row| UserId(row.get::<i64, _>("user_id") as i64))
      .collect();

    debug!("💬 Chat {} has {} members", chat_id.0, user_ids.len());
    Ok(user_ids)
  }

  /// Get online members of a chat (based on cache and connection state)
  pub async fn get_online_chat_members(&self, chat_id: ChatId) -> Vec<UserId> {
    // First check cache
    if let Some(members) = self.chat_members.get(&chat_id) {
      return members
        .iter()
        .filter(|user_id| self.user_connections.contains_key(user_id))
        .cloned()
        .collect();
    }

    // Not in cache, query from database and cache it
    if let Ok(members) = self.get_chat_members(chat_id).await {
      self.chat_members.insert(chat_id, members.clone());

      // Return online members
      return members
        .into_iter()
        .filter(|user_id| self.user_connections.contains_key(user_id))
        .collect();
    }

    Vec::new()
  }

  /// Register user to all chats when they connect
  pub async fn register_user_to_chats(&self, user_id: UserId) -> Result<(), anyhow::Error> {
    // Query chats the user is in
    let user_chats = self.get_user_chats(user_id).await?;

    // Add user to each chat's member map
    for chat_id in &user_chats {
      self
        .chat_members
        .entry(*chat_id)
        .or_default()
        .insert(user_id);
    }

    // Update user-chats map
    self.user_chats.insert(user_id, user_chats.clone());

    info!(
      "✅ User {} registered to {} chats",
      user_id.0,
      user_chats.len()
    );
    Ok(())
  }

  /// Clean up mappings when user disconnects
  pub async fn unregister_user_from_chats(&self, user_id: UserId) {
    // Remove from user connections
    self.user_connections.remove(&user_id);

    // Remove from all chat member maps
    if let Some((_, user_chats)) = self.user_chats.remove(&user_id) {
      for chat_id in user_chats {
        if let Some(mut members) = self.chat_members.get_mut(&chat_id) {
          members.remove(&user_id);
        }
      }
    }

    info!("🧹 User {} has been unregistered from all chats", user_id.0);
  }

  /// Send event to a specific user
  pub fn send_to_user(&self, user_id: UserId, event: Arc<NotifyEvent>) -> bool {
    if let Some(tx) = self.user_connections.get(&user_id) {
      match tx.send(event) {
        Ok(_) => true,
        Err(e) => {
          warn!("❌ Failed to send event to user {}: {}", user_id.0, e);
          // Automatically clean up invalid connection
          self.user_connections.remove(&user_id);
          false
        }
      }
    } else {
      false
    }
  }

  /// Broadcast event to all online members of a chat
  pub async fn broadcast_to_chat(&self, chat_id: ChatId, event: Arc<NotifyEvent>) -> usize {
    let online_members = self.get_online_chat_members(chat_id).await;
    let mut sent_count = 0;

    for user_id in online_members {
      if self.send_to_user(user_id, event.clone()) {
        sent_count += 1;
      }
    }

    if sent_count > 0 {
      info!(
        "📡 Event broadcasted to {} online members in chat {}",
        sent_count, chat_id.0
      );
    }

    sent_count
  }

  /// Broadcast event to multiple users
  pub fn broadcast_to_users(&self, user_ids: Vec<UserId>, event: Arc<NotifyEvent>) -> usize {
    let mut sent_count = 0;

    for user_id in user_ids {
      if self.send_to_user(user_id, event.clone()) {
        sent_count += 1;
      }
    }

    if sent_count > 0 {
      info!("📡 Event broadcasted to {} users", sent_count);
    }

    sent_count
  }

  /// Check if a user is online
  pub fn is_user_online(&self, user_id: UserId) -> bool {
    self.user_connections.contains_key(&user_id)
  }

  /// Get number of online users
  pub fn online_user_count(&self) -> usize {
    self.user_connections.len()
  }

  /// Get number of active chats
  pub fn active_chat_count(&self) -> usize {
    self.chat_members.len()
  }

  /// Add user to chat (used by event processor)
  pub async fn add_user_to_chat(&self, user_id: UserId, chat_id: ChatId) {
    // Add to chat members
    self.chat_members
      .entry(chat_id)
      .or_default()
      .insert(user_id);

    // Add to user chats
    self.user_chats
      .entry(user_id)
      .or_default()
      .insert(chat_id);

    info!("👥 Added user {} to chat {}", user_id.0, chat_id.0);
  }

  /// Remove user from chat (used by event processor)
  pub async fn remove_user_from_chat(&self, user_id: UserId, chat_id: ChatId) {
    // Remove from chat members
    if let Some(mut members) = self.chat_members.get_mut(&chat_id) {
      members.remove(&user_id);
    }

    // Remove from user chats
    if let Some(mut chats) = self.user_chats.get_mut(&user_id) {
      chats.remove(&chat_id);
    }

    info!("👥 Removed user {} from chat {}", user_id.0, chat_id.0);
  }

  /// Send notification to a specific user (JSON format)
  pub async fn send_notification_to_user(
    &self,
    user_id: UserId,
    notification: serde_json::Value,
  ) -> Result<(), NotifyError> {
    let event = Arc::new(NotifyEvent::Generic(notification));
    if self.send_to_user(user_id, event) {
      Ok(())
    } else {
      Err(NotifyError::Internal(format!("User {} not connected", user_id.0)))
    }
  }

  /// Update connection status (simplified implementation)
  pub async fn update_connection_status(&self, user_id: UserId, status: ConnectionUpdate) {
    match status {
      ConnectionUpdate::Connected => {
        info!("🟢 User {} connected", user_id.0);
        // Connection is already tracked in user_connections map
      }
      ConnectionUpdate::Disconnected => {
        info!("🔴 User {} disconnected", user_id.0);
        self.unregister_user_from_chats(user_id).await;
      }
    }
  }

  /// Broadcast notification to all connected users
  pub async fn broadcast_to_all_users(&self, notification: serde_json::Value) -> Result<(), NotifyError> {
    let event = Arc::new(NotifyEvent::Generic(notification));
    let user_ids: Vec<UserId> = self.user_connections.iter().map(|entry| *entry.key()).collect();
    
    let sent_count = self.broadcast_to_users(user_ids, event);
    info!("📢 Broadcasted notification to {} users", sent_count);
    
    Ok(())
  }

  /// Get analytics publisher reference
  pub fn analytics_publisher(&self) -> &AnalyticsPublisher {
    &self.analytics
  }
}

/// Connection update types for event processor
#[derive(Debug, Clone)]
pub enum ConnectionUpdate {
  Connected,
  Disconnected,
}
