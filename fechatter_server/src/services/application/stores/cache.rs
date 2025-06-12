//! # Core Cache Store
//!
//! **Responsibility**: Unified cache strategy management and invalidation control
//! **Principles**: Simplify cache usage, unified TTL policies, intelligent invalidation mechanisms

use std::sync::Arc;
use tracing::{debug, warn};

/// Cache data types
#[derive(Debug, Clone, Copy)]
pub enum CacheDataType {
  UserProfile,
  ChatList,
  MessageList,
  SearchResults,
  FileMetadata,
}

/// Invalidation patterns
#[derive(Debug, Clone)]

pub enum InvalidationPattern {
  /// Single key invalidation
  Single(String),
  /// Pattern matching invalidation
  Pattern(String),
  /// Batch invalidation
  Batch(Vec<String>),
}

/// Cache strategy service
pub struct CacheStrategyService {
  cache: Option<Arc<crate::services::infrastructure::cache::RedisCacheService>>,
}

impl CacheStrategyService {
  /// Create a new cache strategy service
  pub fn new(cache: Arc<crate::services::infrastructure::cache::RedisCacheService>) -> Self {
    Self { cache: Some(cache) }
  }

  /// Create a cache strategy service with optional cache
  pub fn new_optional(
    cache: Option<Arc<crate::services::infrastructure::cache::RedisCacheService>>,
  ) -> Self {
    Self { cache }
  }

  /// Create a no-op cache strategy service (for testing or no-cache environments)
  pub fn new_noop() -> Self {
    Self { cache: None }
  }

  // ============================================================================
  // TTL Policy Constants
  // ============================================================================

  /// User profile cache TTL (30 minutes)
  pub const USER_PROFILE_TTL: u64 = 1800;

  /// Chat list cache TTL (10 minutes)
  pub const CHAT_LIST_TTL: u64 = 600;

  /// Message list cache TTL (5 minutes)
  pub const MESSAGE_LIST_TTL: u64 = 300;

  /// Search results cache TTL (2 minutes)
  pub const SEARCH_RESULTS_TTL: u64 = 120;

  /// File metadata cache TTL (1 hour)
  pub const FILE_METADATA_TTL: u64 = 3600;

  /// Chat detail cache TTL (alias for compatibility)
  pub const CHAT_DETAIL_TTL: u64 = Self::CHAT_LIST_TTL;

  // ============================================================================
  // Cache Key Generation Methods
  // ============================================================================

  /// Generate user profile cache key
  pub fn user_profile_key(user_id: i64) -> String {
    format!("user:profile:{}", user_id)
  }

  /// Generate chat list cache key
  pub fn chat_list_key(user_id: i64) -> String {
    format!("user:chats:{}", user_id)
  }

  /// Generate user chats cache key (alias for compatibility)
  pub fn user_chats_key(user_id: i64) -> String {
    Self::chat_list_key(user_id)
  }

  /// Generate chat detail cache key
  pub fn chat_detail_key(chat_id: i64) -> String {
    format!("chat:detail:{}", chat_id)
  }

  /// Generate message list cache key
  pub fn message_list_key(chat_id: i64, page: u32) -> String {
    format!("chat:messages:{}:page:{}", chat_id, page)
  }

  /// Generate search results cache key
  pub fn search_results_key(query: &str, user_id: i64) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    let query_hash = format!("{:x}", hasher.finalize());
    format!("search:{}:user:{}", query_hash, user_id)
  }

  // ============================================================================
  // Unified Cache Operation Methods
  // ============================================================================

  /// Get TTL policy
  pub fn get_ttl_for_type(&self, data_type: CacheDataType) -> u64 {
    match data_type {
      CacheDataType::UserProfile => Self::USER_PROFILE_TTL,
      CacheDataType::ChatList => Self::CHAT_LIST_TTL,
      CacheDataType::MessageList => Self::MESSAGE_LIST_TTL,
      CacheDataType::SearchResults => Self::SEARCH_RESULTS_TTL,
      CacheDataType::FileMetadata => Self::FILE_METADATA_TTL,
    }
  }

  /// Set cache value
  pub async fn set<T>(&self, key: &str, value: &T, ttl: u64) -> Result<(), String>
  where
    T: serde::Serialize,
  {
    if let Some(cache) = &self.cache {
      match serde_json::to_string(value) {
        Ok(json_value) => cache
          .set(key, &json_value, ttl)
          .await
          .map_err(|e| e.to_string()),
        Err(e) => {
          warn!("Failed to serialize value for cache key {}: {}", key, e);
          Err(e.to_string())
        }
      }
    } else {
      debug!("Cache not available, skipping set for key: {}", key);
      Ok(())
    }
  }

  /// Get cache value
  pub async fn get<T>(&self, key: &str) -> Result<Option<T>, String>
  where
    T: serde::de::DeserializeOwned,
  {
    if let Some(cache) = &self.cache {
      match cache.get::<String>(key).await {
        Ok(Some(json_value)) => match serde_json::from_str::<T>(&json_value) {
          Ok(value) => Ok(Some(value)),
          Err(e) => {
            warn!("Failed to deserialize cached value for key {}: {}", key, e);
            Err(e.to_string())
          }
        },
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
      }
    } else {
      debug!("Cache not available, returning None for key: {}", key);
      Ok(None)
    }
  }

  /// Delete cache key
  pub async fn del(&self, key: &str) -> Result<(), String> {
    if let Some(cache) = &self.cache {
      cache.del(key).await.map(|_| ()).map_err(|e| e.to_string())
    } else {
      debug!("Cache not available, skipping delete for key: {}", key);
      Ok(())
    }
  }

  // ============================================================================
  // Direct Invalidation Methods - Not Dependent on Event System
  // ============================================================================

  /// Handle cache invalidation after message sent
  pub async fn invalidate_on_message_sent(&self, chat_id: i64, chat_members: &[i64]) {
    self.invalidate_message_caches(chat_id, chat_members).await;
  }

  /// Handle cache invalidation after chat updated
  pub async fn invalidate_on_chat_updated(&self, chat_id: i64, affected_users: &[i64]) {
    self.invalidate_chat_caches(chat_id, affected_users).await;
  }

  /// Handle cache invalidation after user updated
  pub async fn invalidate_on_user_updated(&self, user_id: i64) {
    self.invalidate_user_caches(user_id).await;
  }

  // ============================================================================
  // Batch Invalidation Methods
  // ============================================================================

  /// Invalidate all user-related caches
  pub async fn invalidate_user_caches(&self, user_id: i64) {
    let keys_to_invalidate = vec![
      Self::user_profile_key(user_id),
      Self::chat_list_key(user_id),
    ];

    for key in keys_to_invalidate {
      if let Err(e) = self.del(&key).await {
        warn!("Failed to invalidate cache key {}: {}", key, e);
      }
    }
  }

  /// Invalidate message-related caches
  pub async fn invalidate_message_caches(&self, chat_id: i64, affected_users: &[i64]) {
    // Invalidate message list cache (manually invalidate main pages instead of using pattern matching)
    if let Some(cache) = &self.cache {
      // Invalidate first few pages manually since del_pattern is not available
      for page in 0..5 {
        let key = format!("chat:messages:{}:page:{}", chat_id, page);
        if let Err(e) = cache.del(&key).await {
          warn!("Failed to invalidate message cache {}: {}", key, e);
        }
      }
    }

    // Invalidate affected users' chat lists
    for &user_id in affected_users {
      let key = Self::chat_list_key(user_id);
      if let Err(e) = self.del(&key).await {
        warn!("Failed to invalidate chat list for user {}: {}", user_id, e);
      }
    }
  }

  /// Invalidate chat-related caches
  pub async fn invalidate_chat_caches(&self, chat_id: i64, affected_users: &[i64]) {
    // Invalidate chat detail cache
    let chat_detail_key = format!("chat:detail:{}", chat_id);
    if let Err(e) = self.del(&chat_detail_key).await {
      warn!("Failed to invalidate chat detail cache: {}", e);
    }

    // Invalidate affected users' chat lists
    for &user_id in affected_users {
      let key = Self::chat_list_key(user_id);
      if let Err(e) = self.del(&key).await {
        warn!("Failed to invalidate chat list for user {}: {}", user_id, e);
      }
    }
  }

  /// Batch invalidate caches
  pub async fn invalidate_batch(&self, pattern: InvalidationPattern) {
    match pattern {
      InvalidationPattern::Single(key) => {
        if let Err(e) = self.del(&key).await {
          warn!("Failed to invalidate single cache key {}: {}", key, e);
        }
      }
      InvalidationPattern::Pattern(pattern) => {
        warn!(
          "Pattern-based cache invalidation not supported in current Redis implementation: {}",
          pattern
        );
        // TODO: Implement pattern-based invalidation using Redis SCAN command
        // For now, we can only manually invalidate known keys
      }
      InvalidationPattern::Batch(keys) => {
        for key in keys {
          if let Err(e) = self.del(&key).await {
            warn!("Failed to invalidate cache key {}: {}", key, e);
          }
        }
      }
    }
  }
}
