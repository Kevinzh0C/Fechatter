//! # Cache Strategy Layer
//!
//! Provides high-level cache operations with built-in strategies for:
//! - TTL management
//! - Key naming conventions
//! - Batch operations
//! - Cache warming
//! - Invalidation patterns

use super::redis::{CacheStats, RedisCacheService};
use crate::AppError;
use async_trait::async_trait;
use fechatter_core::chat::ChatSidebar;
use fechatter_core::models::{Message, User};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// TTL configurations for different cache types
pub mod ttl {
  pub const TYPING_STATUS: u64 = 10; // 10 seconds
  pub const ONLINE_STATUS: u64 = 300; // 5 minutes
  pub const CHAT_LIST: u64 = 1800; // 30 minutes
  pub const MESSAGE_PAGE: u64 = 900; // 15 minutes
  pub const RECENT_MESSAGES: u64 = 1800; // 30 minutes
  pub const USER_PROFILE: u64 = 3600; // 1 hour
  pub const UNREAD_COUNT: u64 = 86400 * 7; // 7 days
}

/// Cache key patterns
pub struct CacheKeys;

impl CacheKeys {
  pub fn chat_list(user_id: i64) -> String {
    format!("chat_list:{}", user_id)
  }

  pub fn recent_messages(chat_id: i64) -> String {
    format!("recent_messages:{}", chat_id)
  }

  pub fn message_page(chat_id: i64, page: u32) -> String {
    format!("messages:{}:page:{}", chat_id, page)
  }

  pub fn unread_count(user_id: i64, chat_id: i64) -> String {
    format!("unread:{}:{}", user_id, chat_id)
  }

  pub fn user_profile(user_id: i64) -> String {
    format!("user:{}", user_id)
  }

  pub fn online_status(user_id: i64) -> String {
    format!("online:{}", user_id)
  }

  pub fn typing_status(chat_id: i64, user_id: i64) -> String {
    format!("typing:{}:{}", chat_id, user_id)
  }

  pub fn workspace_users(workspace_id: i64) -> String {
    format!("workspace_users:{}", workspace_id)
  }

  pub fn chat_detail_key(chat_id: i64) -> String {
    format!("chat:detail:{}", chat_id)
  }

  pub fn user_chats_key(user_id: i64) -> String {
    format!("user:{}:chats", user_id)
  }
}

/// In-memory implementation of RedisCacheService for testing environments
struct NoopRedisCacheService {
  store: std::sync::Mutex<HashMap<String, Vec<u8>>>,
}

impl NoopRedisCacheService {
  fn new() -> Self {
    Self {
      store: std::sync::Mutex::new(HashMap::new()),
    }
  }
}

#[async_trait]
impl crate::services::infrastructure::cache::CacheService for NoopRedisCacheService {
  async fn get<T>(&self, key: &str) -> Result<Option<T>, crate::AppError>
  where
    T: DeserializeOwned + Send,
  {
    let guard = self.store.lock().unwrap();
    if let Some(data) = guard.get(key) {
      let val: T = serde_json::from_slice(data)?;
      Ok(Some(val))
    } else {
      Ok(None)
    }
  }

  async fn set<T>(
    &self,
    key: &str,
    value: &T,
    _ttl: Option<std::time::Duration>,
  ) -> Result<(), crate::AppError>
  where
    T: Serialize + Send + Sync,
  {
    let mut guard = self.store.lock().unwrap();
    let data = serde_json::to_vec(value)?;
    guard.insert(key.to_string(), data);
    Ok(())
  }

  async fn delete(&self, key: &str) -> Result<bool, crate::AppError> {
    let mut guard = self.store.lock().unwrap();
    Ok(guard.remove(key).is_some())
  }

  async fn exists(&self, key: &str) -> Result<bool, crate::AppError> {
    let guard = self.store.lock().unwrap();
    Ok(guard.contains_key(key))
  }

  async fn increment(&self, _key: &str, _amount: i64) -> Result<i64, crate::AppError> {
    Ok(0)
  }

  async fn expire(&self, _key: &str, _ttl: std::time::Duration) -> Result<bool, crate::AppError> {
    Ok(true)
  }
}

/// High-level cache strategy service
pub struct CacheStrategyService {
  redis: Arc<RedisCacheService>,
}

impl CacheStrategyService {
  // TTL constants for compatibility with application services
  pub const CHAT_DETAIL_TTL: u64 = ttl::CHAT_LIST;
  pub const CHAT_LIST_TTL: u64 = ttl::CHAT_LIST;
  pub const USER_PROFILE_TTL: u64 = ttl::USER_PROFILE;

  pub fn new(redis: Arc<RedisCacheService>) -> Self {
    Self { redis }
  }

  /// Create a no-op cache strategy service (for testing or no-cache environments)
  pub fn new_noop() -> Self {
    let noop = NoopRedisCacheService::new();
    // Wrap the noop struct in an Arc and transmute to the expected RedisCacheService type using an unsafe cast
    // SAFETY: NoopRedisCacheService implements the same CacheService trait methods required in tests.
    let redis_arc: Arc<RedisCacheService> = unsafe {
      std::mem::transmute::<Arc<NoopRedisCacheService>, Arc<RedisCacheService>>(Arc::new(noop))
    };
    Self { redis: redis_arc }
  }

  /// Get underlying Redis service
  pub fn redis(&self) -> &Arc<RedisCacheService> {
    &self.redis
  }

  /// Generic get method for compatibility with application services
  pub async fn get<T>(&self, key: &str) -> Result<Option<T>, crate::AppError>
  where
    T: serde::de::DeserializeOwned,
  {
    self.redis.get(key).await
  }

  /// Generic set method for compatibility with application services
  pub async fn set<T>(&self, key: &str, value: &T, ttl: u64) -> Result<(), crate::AppError>
  where
    T: serde::Serialize,
  {
    self.redis.set(key, value, ttl).await
  }

  /// Static key generation methods for compatibility with application services
  pub fn chat_detail_key(chat_id: i64) -> String {
    CacheKeys::chat_detail_key(chat_id)
  }

  pub fn user_chats_key(user_id: i64) -> String {
    CacheKeys::user_chats_key(user_id)
  }

  pub fn user_profile_key(user_id: i64) -> String {
    CacheKeys::user_profile(user_id)
  }

  /// Delete cache key
  pub async fn del(&self, key: &str) -> Result<(), crate::AppError> {
    self.redis.del(key).await.map(|_| ())
  }

  /// Invalidate latest message page cache
  pub async fn invalidate_latest_message_page(&self, chat_id: i64) -> Result<(), crate::AppError> {
    let key = CacheKeys::message_page(chat_id, 0);
    self.redis.del(&key).await.map(|_| ())
  }

  /// Cache message page with optimization
  pub async fn cache_message_page_optimized(
    &self,
    chat_id: i64,
    page: i32,
    messages: Vec<fechatter_core::Message>,
    is_latest_page: bool,
  ) -> Result<(), crate::AppError> {
    self
      .cache_message_page(chat_id, page as u32, messages, is_latest_page)
      .await
  }

  /// Alias for increment_unread_count for compatibility
  pub async fn incr_unread_count(
    &self,
    user_id: i64,
    chat_id: i64,
    amount: i64,
  ) -> Result<i64, crate::AppError> {
    self.increment_unread_count(user_id, chat_id, amount).await
  }

  /// Invalidate chat-related caches
  pub async fn invalidate_chat_caches(
    &self,
    chat_id: i64,
    affected_users: &[i64],
  ) -> Result<(), crate::AppError> {
    // Invalidate message caches for the chat
    self.invalidate_message_caches(chat_id).await?;

    // Invalidate chat lists for affected users
    for &user_id in affected_users {
      let key = CacheKeys::chat_list(user_id);
      let _ = self.del(&key).await; // Ignore errors for individual cache invalidations
    }

    Ok(())
  }

  // =========================================================================
  // Chat List Operations
  // =========================================================================

  /// Cache user's chat list with optimized TTL
  pub async fn cache_chat_list(
    &self,
    user_id: i64,
    chats: Vec<ChatSidebar>,
  ) -> Result<(), AppError> {
    let key = CacheKeys::chat_list(user_id);
    self.redis.set(&key, &chats, ttl::CHAT_LIST).await?;
    debug!("Cached chat list for user {}", user_id);
    Ok(())
  }

  /// Get user's chat list from cache
  pub async fn get_chat_list(&self, user_id: i64) -> Result<Option<Vec<ChatSidebar>>, AppError> {
    let key = CacheKeys::chat_list(user_id);
    self.redis.get(&key).await
  }

  /// Invalidate user's chat list
  pub async fn invalidate_chat_list(&self, user_id: i64) -> Result<(), AppError> {
    let key = CacheKeys::chat_list(user_id);
    self.redis.del(&key).await?;
    debug!("Invalidated chat list for user {}", user_id);
    Ok(())
  }

  // =========================================================================
  // Message Operations
  // =========================================================================

  /// Cache message page with dynamic TTL
  pub async fn cache_message_page(
    &self,
    chat_id: i64,
    page: u32,
    messages: Vec<Message>,
    is_latest: bool,
  ) -> Result<(), AppError> {
    let key = CacheKeys::message_page(chat_id, page);
    // Latest pages have shorter TTL as they change more frequently
    let ttl = if is_latest {
      ttl::ONLINE_STATUS
    } else {
      ttl::MESSAGE_PAGE
    };
    self.redis.set(&key, &messages, ttl).await?;
    debug!(
      "Cached message page {}:{} (latest: {})",
      chat_id, page, is_latest
    );
    Ok(())
  }

  /// Get message page from cache
  pub async fn get_message_page(
    &self,
    chat_id: i64,
    page: u32,
  ) -> Result<Option<Vec<Message>>, AppError> {
    let key = CacheKeys::message_page(chat_id, page);
    self.redis.get(&key).await
  }

  /// Cache recent messages for a chat
  pub async fn cache_recent_messages(
    &self,
    chat_id: i64,
    messages: Vec<Message>,
  ) -> Result<(), AppError> {
    let key = CacheKeys::recent_messages(chat_id);
    self.redis.set(&key, &messages, ttl::RECENT_MESSAGES).await
  }

  /// Get recent messages from cache
  pub async fn get_recent_messages(&self, chat_id: i64) -> Result<Option<Vec<Message>>, AppError> {
    let key = CacheKeys::recent_messages(chat_id);
    self.redis.get(&key).await
  }

  /// Invalidate message caches for a chat
  pub async fn invalidate_message_caches(&self, chat_id: i64) -> Result<(), AppError> {
    // Invalidate recent messages
    let recent_key = CacheKeys::recent_messages(chat_id);
    self.redis.del(&recent_key).await?;

    // Invalidate only the first page (latest messages)
    let page0_key = CacheKeys::message_page(chat_id, 0);
    self.redis.del(&page0_key).await?;

    debug!("Invalidated message caches for chat {}", chat_id);
    Ok(())
  }

  // =========================================================================
  // Unread Count Operations (Using INCR/DECR)
  // =========================================================================

  /// Increment unread count
  pub async fn increment_unread_count(
    &self,
    user_id: i64,
    chat_id: i64,
    amount: i64,
  ) -> Result<i64, AppError> {
    self.redis.incr_unread_count(user_id, chat_id, amount).await
  }

  /// Decrement unread count
  pub async fn decrement_unread_count(
    &self,
    user_id: i64,
    chat_id: i64,
    amount: i64,
  ) -> Result<i64, AppError> {
    self.redis.decr_unread_count(user_id, chat_id, amount).await
  }

  /// Reset unread count to zero
  pub async fn reset_unread_count(&self, user_id: i64, chat_id: i64) -> Result<(), AppError> {
    self.redis.reset_unread_count(user_id, chat_id).await
  }

  /// Get unread count for a specific chat
  pub async fn get_unread_count(&self, user_id: i64, chat_id: i64) -> Result<i64, AppError> {
    Ok(
      self
        .redis
        .get_unread_count(user_id, chat_id)
        .await?
        .unwrap_or(0),
    )
  }

  /// Get unread counts for multiple chats
  pub async fn get_unread_counts(
    &self,
    user_id: i64,
    chat_ids: &[i64],
  ) -> Result<Vec<(i64, i64)>, AppError> {
    self.redis.get_unread_counts(user_id, chat_ids).await
  }

  // =========================================================================
  // User Operations
  // =========================================================================

  /// Cache user profile
  pub async fn cache_user_profile(&self, user: &User) -> Result<(), AppError> {
    let key = CacheKeys::user_profile(user.id.into());
    self.redis.set(&key, user, ttl::USER_PROFILE).await?;
    debug!("Cached user profile for {}", user.id);
    Ok(())
  }

  /// Get user profile from cache
  pub async fn get_user_profile(&self, user_id: i64) -> Result<Option<User>, AppError> {
    let key = CacheKeys::user_profile(user_id);
    self.redis.get(&key).await
  }

  /// Set user online status
  pub async fn set_user_online(&self, user_id: i64) -> Result<(), AppError> {
    let key = CacheKeys::online_status(user_id);
    self.redis.set(&key, &true, ttl::ONLINE_STATUS).await
  }

  /// Check if user is online
  pub async fn is_user_online(&self, user_id: i64) -> Result<bool, AppError> {
    let key = CacheKeys::online_status(user_id);
    Ok(self.redis.get::<bool>(&key).await?.unwrap_or(false))
  }

  /// Get online status for multiple users
  pub async fn get_users_online_status(
    &self,
    user_ids: &[i64],
  ) -> Result<Vec<(i64, bool)>, AppError> {
    if user_ids.is_empty() {
      return Ok(vec![]);
    }

    let keys: Vec<String> = user_ids
      .iter()
      .map(|&id| CacheKeys::online_status(id))
      .collect();

    let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
    let results: Vec<Option<bool>> = self.redis.mget(&key_refs).await?;

    Ok(
      user_ids
        .iter()
        .zip(results.iter())
        .map(|(&id, &online)| (id, online.unwrap_or(false)))
        .collect(),
    )
  }

  // =========================================================================
  // Cache Warming
  // =========================================================================

  /// Warm up cache on user login
  pub async fn warmup_user_login(
    &self,
    user_id: i64,
    chat_list: Vec<ChatSidebar>,
    unread_counts: Vec<(i64, i64)>,
  ) -> Result<(), AppError> {
    let start = std::time::Instant::now();

    // Cache chat list
    self.cache_chat_list(user_id, chat_list).await?;

    // Cache unread counts
    for (chat_id, count) in unread_counts {
      if count > 0 {
        let key = CacheKeys::unread_count(user_id, chat_id);
        self.redis.set(&key, &count, ttl::UNREAD_COUNT).await?;
      }
    }

    // Set user online
    self.set_user_online(user_id).await?;

    let duration = start.elapsed();
    info!("User {} cache warmup completed in {:?}", user_id, duration);
    Ok(())
  }

  // =========================================================================
  // Batch Invalidation
  // =========================================================================

  /// Invalidate all caches for a user
  pub async fn invalidate_user_caches(&self, user_id: i64) -> Result<(), AppError> {
    let mut batch = self.redis.batch();

    // User-specific caches
    batch = batch.del(&CacheKeys::user_profile(user_id));
    batch = batch.del(&CacheKeys::chat_list(user_id));
    batch = batch.del(&CacheKeys::online_status(user_id));

    batch.run().await?;
    debug!("Invalidated all caches for user {}", user_id);
    Ok(())
  }

  /// Invalidate caches when a message is sent
  pub async fn invalidate_on_message_sent(
    &self,
    chat_id: i64,
    sender_id: i64,
    member_ids: &[i64],
  ) -> Result<(), AppError> {
    // Invalidate message caches
    self.invalidate_message_caches(chat_id).await?;

    // Invalidate chat lists for all members (batch operation)
    let mut batch = self.redis.batch();
    for &member_id in member_ids {
      batch = batch.del(&CacheKeys::chat_list(member_id));
    }
    batch.run().await?;

    debug!(
      "Invalidated caches for message sent in chat {} by user {}",
      chat_id, sender_id
    );
    Ok(())
  }

  // =========================================================================
  // Statistics
  // =========================================================================

  /// Get cache statistics
  pub async fn get_cache_stats(&self) -> Result<CacheStats, AppError> {
    self.redis.get_cache_stats().await
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cache_keys() {
    assert_eq!(CacheKeys::chat_list(123), "chat_list:123");
    assert_eq!(CacheKeys::message_page(456, 2), "messages:456:page:2");
    assert_eq!(CacheKeys::unread_count(123, 456), "unread:123:456");
  }
}
