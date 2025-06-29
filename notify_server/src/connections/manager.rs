use dashmap::DashMap;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::events::types::NotifyEvent;
use fechatter_core::{ChatId, UserId};

/// Connection type definitions
type UserConnections = Arc<DashMap<UserId, broadcast::Sender<Arc<NotifyEvent>>>>;
type ChatMembers = Arc<DashMap<ChatId, HashSet<UserId>>>;
type UserChats = Arc<DashMap<UserId, HashSet<ChatId>>>;

/// ConnectionManager - manages user connections and chat mappings
pub struct ConnectionManager {
  user_connections: UserConnections,
  chat_members: ChatMembers,
  user_chats: UserChats,
}

impl ConnectionManager {
  /// Create a new ConnectionManager
  pub fn new() -> Self {
    Self {
      user_connections: Arc::new(DashMap::new()),
      chat_members: Arc::new(DashMap::new()),
      user_chats: Arc::new(DashMap::new()),
    }
  }

  /// Get the user connections map
  pub fn user_connections(&self) -> &UserConnections {
    &self.user_connections
  }

  /// Get the chat members map
  pub fn chat_members(&self) -> &ChatMembers {
    &self.chat_members
  }

  /// Get the user chats map
  pub fn user_chats(&self) -> &UserChats {
    &self.user_chats
  }

  /// Add a user connection
  pub fn add_user_connection(&self, user_id: UserId, sender: broadcast::Sender<Arc<NotifyEvent>>) {
    self.user_connections.insert(user_id, sender);
    info!("🔗 User {} connection added", user_id.0);
  }

  /// Remove a user connection
  pub fn remove_user_connection(&self, user_id: UserId) -> bool {
    if self.user_connections.remove(&user_id).is_some() {
      info!("🔌 User {} connection removed", user_id.0);
      true
    } else {
      false
    }
  }

  /// Check if a user is online
  pub fn is_user_online(&self, user_id: UserId) -> bool {
    self.user_connections.contains_key(&user_id)
  }

  /// Get the number of online users
  pub fn online_user_count(&self) -> usize {
    self.user_connections.len()
  }

  /// Get the number of active chats
  pub fn active_chat_count(&self) -> usize {
    self.chat_members.len()
  }

  /// Add a user to a chat
  pub fn add_user_to_chat(&self, user_id: UserId, chat_id: ChatId) {
    // Add to chat members
    self
      .chat_members
      .entry(chat_id)
      .or_default()
      .insert(user_id);

    // Add to user chats
    self.user_chats.entry(user_id).or_default().insert(chat_id);

    info!("👥 User {} added to chat {}", user_id.0, chat_id.0);
  }

  /// Remove a user from a chat
  pub fn remove_user_from_chat(&self, user_id: UserId, chat_id: ChatId) {
    // Remove from chat members
    if let Some(mut members) = self.chat_members.get_mut(&chat_id) {
      members.remove(&user_id);
    }

    // Remove from user chats
    if let Some(mut chats) = self.user_chats.get_mut(&user_id) {
      chats.remove(&chat_id);
    }

    info!("👥 User {} removed from chat {}", user_id.0, chat_id.0);
  }

  /// Get online members of a chat
  pub fn get_online_chat_members(&self, chat_id: ChatId) -> Vec<UserId> {
    if let Some(members) = self.chat_members.get(&chat_id) {
      members
        .iter()
        .filter(|user_id| self.is_user_online(**user_id))
        .cloned()
        .collect()
    } else {
      Vec::new()
    }
  }

  /// Get chats a user is in
  pub fn get_user_chats(&self, user_id: UserId) -> Option<HashSet<ChatId>> {
    self.user_chats.get(&user_id).map(|chats| chats.clone())
  }

  /// Clean up all connections and mappings for a user
  pub fn cleanup_user(&self, user_id: UserId) {
    // Remove user connection
    self.remove_user_connection(user_id);

    // Remove from all chat member mappings
    if let Some((_, user_chats)) = self.user_chats.remove(&user_id) {
      for chat_id in user_chats {
        if let Some(mut members) = self.chat_members.get_mut(&chat_id) {
          members.remove(&user_id);
        }
      }
    }

    info!(
      "🧹 All connections and mappings for user {} cleaned up",
      user_id.0
    );
  }

  /// Send an event to a specific user
  pub fn send_to_user(&self, user_id: UserId, event: Arc<NotifyEvent>) -> bool {
    if let Some(tx) = self.user_connections.get(&user_id) {
      match tx.send(event) {
        Ok(_) => true,
        Err(e) => {
          warn!("ERROR: Failed to send event to user {}: {}", user_id.0, e);
          // Automatically remove invalid connection
          self.remove_user_connection(user_id);
          false
        }
      }
    } else {
      false
    }
  }

  /// Broadcast an event to all online members of a chat
  pub fn broadcast_to_chat(&self, chat_id: ChatId, event: Arc<NotifyEvent>) -> usize {
    let online_members = self.get_online_chat_members(chat_id);
    let mut sent_count = 0;

    for user_id in online_members {
      if self.send_to_user(user_id, event.clone()) {
        sent_count += 1;
      }
    }

    info!(
      "SUBSCRIPTION: Event broadcasted to {} online members in chat {}",
      sent_count, chat_id.0
    );
    sent_count
  }

  /// Broadcast an event to multiple users
  pub fn broadcast_to_users(&self, user_ids: Vec<UserId>, event: Arc<NotifyEvent>) -> usize {
    let mut sent_count = 0;

    for user_id in user_ids {
      if self.send_to_user(user_id, event.clone()) {
        sent_count += 1;
      }
    }

    info!("SUBSCRIPTION: Event broadcasted to {} users", sent_count);
    sent_count
  }

  /// Get connection statistics
  pub fn get_stats(&self) -> ConnectionStats {
    ConnectionStats {
      online_users: self.online_user_count(),
      active_chats: self.active_chat_count(),
      total_connections: self.user_connections.len(),
    }
  }
}

impl Default for ConnectionManager {
  fn default() -> Self {
    Self::new()
  }
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
  pub online_users: usize,
  pub active_chats: usize,
  pub total_connections: usize,
}
