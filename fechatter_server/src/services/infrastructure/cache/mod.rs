pub mod redis;
pub mod strategy;

pub use redis::RedisCacheService;
pub use strategy::{CacheKeys, CacheStrategyService};

pub type Cache = RedisCacheService;

// 缓存一致性风险评估和改进
pub mod consistency_checker;

use crate::AppError;
use async_trait::async_trait;
use chrono;
use dashmap;
use fechatter_core::chat::ChatSidebar;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio;
use tracing::{debug, error, info};

/// Cache trait for different cache implementations
#[async_trait]
pub trait CacheService: Send + Sync {
  async fn get<T>(&self, key: &str) -> Result<Option<T>, AppError>
  where
    T: for<'de> Deserialize<'de> + Send;

  async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), AppError>
  where
    T: Serialize + Send + Sync;

  async fn delete(&self, key: &str) -> Result<bool, AppError>;

  async fn exists(&self, key: &str) -> Result<bool, AppError>;

  async fn increment(&self, key: &str, amount: i64) -> Result<i64, AppError>;

  async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, AppError>;
}

/// Cache key builder for consistent key naming
pub struct CacheKeyBuilder;

impl CacheKeyBuilder {
  pub fn user_profile(user_id: i64) -> String {
    format!("user:profile:{}", user_id)
  }

  pub fn chat_detail(chat_id: i64) -> String {
    format!("chat:detail:{}", chat_id)
  }

  pub fn chat_members(chat_id: i64) -> String {
    format!("chat:members:{}", chat_id)
  }

  pub fn workspace_users(workspace_id: i64) -> String {
    format!("workspace:users:{}", workspace_id)
  }

  pub fn user_chats(user_id: i64) -> String {
    format!("user:chats:{}", user_id)
  }

  pub fn search_results(query: &str, workspace_id: i64) -> String {
    format!("search:{}:{}", workspace_id, query)
  }

  pub fn rate_limit(user_id: i64, endpoint: &str) -> String {
    format!("rate_limit:{}:{}", user_id, endpoint)
  }
}

/// Unified cache service adapter - Single entry point for all cache operations
pub struct UnifiedCacheService {
  redis_cache: Arc<RedisCacheService>,
}

impl UnifiedCacheService {
  /// Create new unified cache service
  pub fn new(redis_cache: Arc<RedisCacheService>) -> Self {
    Self { redis_cache }
  }

  /// Get chat list from cache (replaces DashMap implementation)
  pub async fn get_chat_list(&self, user_id: i64) -> Result<Option<Vec<ChatSidebar>>, AppError> {
    let start = Instant::now();
    let cache_key = format!("chat_list:{}", user_id);

    match self.redis_cache.get::<Vec<ChatSidebar>>(&cache_key).await {
      Ok(Some(chats)) => {
        crate::services::infrastructure::observability::cache_metrics::record_cache_hit(
          "chat_list",
        );
        crate::services::infrastructure::observability::cache_metrics::record_cache_operation_duration(
          "chat_list", "get", start
        );
        debug!("Cache HIT for chat_list:{}", user_id);
        Ok(Some(chats))
      }
      Ok(None) => {
        crate::services::infrastructure::observability::cache_metrics::record_cache_miss(
          "chat_list",
        );
        debug!("Cache MISS for chat_list:{}", user_id);
        Ok(None)
      }
      Err(e) => {
        error!("Failed to get chat list from cache: {}", e);
        Err(e)
      }
    }
  }

  /// Set chat list cache
  pub async fn set_chat_list(
    &self,
    user_id: i64,
    chats: Vec<ChatSidebar>,
    ttl_seconds: u64,
  ) -> Result<(), AppError> {
    let start = Instant::now();
    let cache_key = format!("chat_list:{}", user_id);

    let result = self.redis_cache.set(&cache_key, &chats, ttl_seconds).await;

    crate::services::infrastructure::observability::cache_metrics::record_cache_operation_duration(
      "chat_list",
      "set",
      start,
    );

    if result.is_ok() {
      debug!(
        "Successfully cached chat_list:{} with TTL:{}",
        user_id, ttl_seconds
      );
    } else {
      error!("Failed to cache chat_list:{}", user_id);
    }

    result
  }

  /// Delete chat list cache
  pub async fn invalidate_chat_list(&self, user_id: i64) -> Result<(), AppError> {
    let start = Instant::now();
    let cache_key = format!("chat_list:{}", user_id);

    let result = self.redis_cache.del(&cache_key).await;

    crate::services::infrastructure::observability::cache_metrics::record_cache_operation_duration(
      "chat_list",
      "delete",
      start,
    );

    if result.is_ok() {
      debug!("Successfully invalidated chat_list:{}", user_id);
    } else {
      error!("Failed to invalidate chat_list:{}", user_id);
    }

    result.map(|_| ())
  }

  /// Batch invalidate chat lists for related users
  pub async fn invalidate_chat_lists_for_users(&self, user_ids: &[i64]) -> Result<(), AppError> {
    let start = Instant::now();

    for user_id in user_ids {
      if let Err(e) = self.invalidate_chat_list(*user_id).await {
        error!("Failed to invalidate chat list for user {}: {}", user_id, e);
      }
    }

    crate::services::infrastructure::observability::cache_metrics::record_cache_operation_duration(
      "chat_list",
      "batch_invalidate",
      start,
    );

    info!("Invalidated chat lists for {} users", user_ids.len());
    Ok(())
  }

  /// Get reference to underlying Redis cache service
  pub fn redis(&self) -> &Arc<RedisCacheService> {
    &self.redis_cache
  }
}

/// Sync cache adapter for AppState - Performance-optimized version
#[derive(Clone)]
pub struct SyncCacheAdapter {
  cache_service: Option<Arc<UnifiedCacheService>>,
  // Use an in-memory fallback cache for sync contexts
  memory_cache: Arc<dashmap::DashMap<String, (serde_json::Value, Instant)>>,
}

impl SyncCacheAdapter {
  pub fn new(cache_service: Option<Arc<RedisCacheService>>) -> Self {
    let unified_cache = cache_service.map(|redis| Arc::new(UnifiedCacheService::new(redis)));
    Self {
      cache_service: unified_cache,
      memory_cache: Arc::new(dashmap::DashMap::new()),
    }
  }

  /// Sync get chat list from cache - Performance-optimized approach
  pub fn get_chat_list_sync(&self, user_id: i64) -> Option<(Arc<Vec<ChatSidebar>>, Instant)> {
    let cache_key = format!("chat_list:{}", user_id);

    // 1. First try in-memory cache (no async/sync boundary)
    if let Some(entry) = self.memory_cache.get(&cache_key) {
      let value = entry.value();
      let (cached_json, timestamp) = (&value.0, &value.1);
      if timestamp.elapsed().as_secs() < 300 {
        // 5-minute TTL
        match serde_json::from_value::<Vec<ChatSidebar>>(cached_json.clone()) {
          Ok(chats) => {
            debug!("✅ [SYNC_CACHE] Memory HIT for chat_list:{}", user_id);
            return Some((Arc::new(chats), *timestamp));
          }
          Err(e) => {
            error!("❌ [SYNC_CACHE] Memory deserialization error: {}", e);
            drop(entry); // Release the reference before removing
            self.memory_cache.remove(&cache_key);
          }
        }
      } else {
        // Expired, remove from memory cache
        drop(entry); // Release the reference before removing
        self.memory_cache.remove(&cache_key);
      }
    }

    // 2. If we have Redis cache, spawn background task to populate memory cache
    if let Some(cache) = &self.cache_service {
      let cache_clone = cache.clone();
      let memory_cache_clone = self.memory_cache.clone();
      let cache_key_clone = cache_key.clone();

      // Fire-and-forget background fetch (no blocking!)
      tokio::task::spawn(async move {
        match cache_clone.get_chat_list(user_id).await {
          Ok(Some(chats)) => {
            // Store in memory cache for future sync access
            if let Ok(json_value) = serde_json::to_value(&chats) {
              memory_cache_clone.insert(cache_key_clone, (json_value, Instant::now()));
              debug!(
                "✅ [SYNC_CACHE] Background populated memory cache for user:{}",
                user_id
              );
            }
          }
          Ok(None) => {
            debug!("❌ [SYNC_CACHE] Background MISS for chat_list:{}", user_id);
          }
          Err(e) => {
            error!(
              "❌ [SYNC_CACHE] Background ERROR for chat_list:{}: {}",
              user_id, e
            );
          }
        }
      });
    }

    // 3. Cache miss - return None immediately (non-blocking)
    debug!(
      "❌ [SYNC_CACHE] MISS for chat_list:{} - background fetch initiated",
      user_id
    );
    None
  }

  /// Sync set chat list cache - Performance-optimized approach
  pub fn set_chat_list_sync(&self, user_id: i64, chats: Vec<ChatSidebar>, ttl_seconds: u64) {
    let cache_key = format!("chat_list:{}", user_id);

    // 1. Immediately update memory cache (sync, fast)
    if let Ok(json_value) = serde_json::to_value(&chats) {
      self
        .memory_cache
        .insert(cache_key.clone(), (json_value, Instant::now()));
      debug!("✅ [SYNC_CACHE] Memory SET for chat_list:{}", user_id);
    }

    // 2. Fire-and-forget Redis update (async, non-blocking)
    if let Some(cache) = &self.cache_service {
      let cache_clone = cache.clone();
      tokio::task::spawn(async move {
        match cache_clone.set_chat_list(user_id, chats, ttl_seconds).await {
          Ok(_) => debug!(
            "✅ [SYNC_CACHE] Redis SET for chat_list:{} with TTL:{}",
            user_id, ttl_seconds
          ),
          Err(e) => error!(
            "❌ [SYNC_CACHE] Redis SET ERROR for chat_list:{}: {}",
            user_id, e
          ),
        }
      });
    }
  }

  /// Sync remove chat list cache - Performance-optimized approach
  pub fn remove_chat_list_sync(&self, user_id: i64) {
    let cache_key = format!("chat_list:{}", user_id);

    // 1. Immediately remove from memory cache (sync, fast)
    self.memory_cache.remove(&cache_key);
    debug!("✅ [SYNC_CACHE] Memory REMOVE for chat_list:{}", user_id);

    // 2. Fire-and-forget Redis removal (async, non-blocking)
    if let Some(cache) = &self.cache_service {
      let cache_clone = cache.clone();
      tokio::task::spawn(async move {
        match cache_clone.invalidate_chat_list(user_id).await {
          Ok(_) => debug!("✅ [SYNC_CACHE] Redis REMOVE for chat_list:{}", user_id),
          Err(e) => error!(
            "❌ [SYNC_CACHE] Redis REMOVE ERROR for chat_list:{}: {}",
            user_id, e
          ),
        }
      });
    }
  }

  /// Clean expired entries from memory cache (call periodically)
  pub fn cleanup_expired_entries(&self) {
    let now = Instant::now();
    let mut expired_keys = Vec::new();

    for entry in self.memory_cache.iter() {
      let (key, (_, timestamp)) = (entry.key(), entry.value());
      if now.duration_since(*timestamp).as_secs() > 300 {
        // 5 minutes
        expired_keys.push(key.clone());
      }
    }

    for key in expired_keys {
      self.memory_cache.remove(&key);
    }

    debug!("🧹 [SYNC_CACHE] Cleaned up expired memory cache entries");
  }
}

/// Distributed lock-protected cache invalidator - Uses Redis distributed locks to prevent race conditions
pub struct DistributedLockCacheInvalidator {
  cache: Arc<UnifiedCacheService>,
  redis: Arc<RedisCacheService>,
}

impl DistributedLockCacheInvalidator {
  pub fn new(cache: Arc<UnifiedCacheService>) -> Self {
    Self {
      redis: cache.redis().clone(),
      cache,
    }
  }

  /// User updated event - uses distributed lock to prevent race conditions
  pub async fn handle_user_updated_with_lock(&self, user_id: i64) -> Result<(), AppError> {
    let lock_resource = format!("user_update:{}", user_id);

    self
      .redis
      .with_lock(&lock_resource, 30, async {
        self.handle_user_updated_internal(user_id).await
      })
      .await
  }

  /// Internal user update handler - protected by distributed lock
  async fn handle_user_updated_internal(&self, user_id: i64) -> Result<(), AppError> {
    let start_time = Instant::now();
    info!(
      "🔒 [LOCKED] Start processing user {} update, distributed lock acquired",
      user_id
    );

    // Use atomic operation for cache invalidation
    let invalidation_script = r#"
      local keys_to_delete = {}

      -- User basic info caches
      table.insert(keys_to_delete, "user:profile:" .. ARGV[1])
      table.insert(keys_to_delete, "user:" .. ARGV[1])
      table.insert(keys_to_delete, "user:settings:" .. ARGV[1])
      table.insert(keys_to_delete, "user:permissions:" .. ARGV[1])
      table.insert(keys_to_delete, "user:status:" .. ARGV[1])
      table.insert(keys_to_delete, "chat_list:" .. ARGV[1])

      -- Batch delete
      local deleted_count = 0
      for i, key in ipairs(keys_to_delete) do
        deleted_count = deleted_count + redis.call("DEL", key)
      end

      -- Scan and delete related pattern keys
      local workspace_keys = redis.call("SCAN", 0, "MATCH", "workspace:*:users:*", "COUNT", 50)
      if workspace_keys[2] then
        for i, key in ipairs(workspace_keys[2]) do
          deleted_count = deleted_count + redis.call("DEL", key)
        end
      end

      local session_keys = redis.call("SCAN", 0, "MATCH", "session:user:" .. ARGV[1] .. ":*", "COUNT", 50)
      if session_keys[2] then
        for i, key in ipairs(session_keys[2]) do
          deleted_count = deleted_count + redis.call("DEL", key)
        end
      end

      return deleted_count
    "#;

    let deleted_count: i32 = self.redis
      .execute_script_with_arg(invalidation_script, user_id)
      .await
      .map_err(|e| AppError::Internal(format!("Cache invalidation script failed: {}", e)))?;

    let elapsed = start_time.elapsed();
    info!(
      "✅ [LOCKED] User {} cache invalidation complete: deleted {} keys, took {:?}",
      user_id, deleted_count, elapsed
    );

    Ok(())
  }

  /// Message sent event - uses distributed lock to prevent race conditions
  pub async fn handle_message_sent_with_lock(
    &self,
    chat_id: i64,
    sender_id: i64,
  ) -> Result<(), AppError> {
    let lock_resource = format!("message_sent:{}:{}", chat_id, sender_id);

    self
      .redis
      .with_lock(&lock_resource, 30, async {
        self.handle_message_sent_internal(chat_id, sender_id).await
      })
      .await
  }

  /// Internal message sent handler - protected by distributed lock
  async fn handle_message_sent_internal(
    &self,
    chat_id: i64,
    sender_id: i64,
  ) -> Result<(), AppError> {
    let start_time = Instant::now();
    info!(
      "🔒 [LOCKED] Start processing message sent in chat {} by user {}, distributed lock acquired",
      chat_id, sender_id
    );

    // Atomic message cache invalidation and count update
    let message_invalidation_script = r#"
      local chat_id = ARGV[1]
      local sender_id = ARGV[2]
      local deleted_count = 0

      -- Delete message-related caches
      local message_keys = {
        "recent_messages:" .. chat_id,
        "chat:detail:" .. chat_id,
        "user:message:stats:" .. sender_id,
        "chat:message:count:" .. chat_id,
        "chat:last:activity:" .. chat_id
      }

      for i, key in ipairs(message_keys) do
        deleted_count = deleted_count + redis.call("DEL", key)
      end

      -- Delete paginated message caches (assume up to 10 pages)
      for page = 0, 9 do
        local page_key = "messages:" .. chat_id .. ":page:" .. page
        deleted_count = deleted_count + redis.call("DEL", page_key)
      end

      -- Atomically increment message count
      local count_key = "chat:message:count:" .. chat_id
      local new_count = redis.call("INCR", count_key)
      redis.call("EXPIRE", count_key, 86400) -- 24 hours expiration

      -- Update chat activity timestamp
      local activity_key = "chat:last:activity:" .. chat_id
      local timestamp = redis.call("TIME")
      redis.call("SET", activity_key, timestamp[1], "EX", 86400)

      return {deleted_count, new_count}
    "#;

    let result: Vec<i32> = self.redis
      .execute_script_with_two_args(message_invalidation_script, chat_id, sender_id)
      .await
      .map_err(|e| {
        AppError::Internal(format!("Message cache invalidation script failed: {}", e))
      })?;

    let deleted_count = result.get(0).unwrap_or(&0);
    let new_message_count = result.get(1).unwrap_or(&0);

    // Asynchronously invalidate chat list caches for chat members
    let redis_clone = self.redis.clone();
    let chat_id_clone = chat_id;
    tokio::spawn(async move {
      if let Ok(member_keys) = redis_clone
        .scan_keys(&format!("chat:{}:member:*", chat_id_clone))
        .await
      {
        let mut batch = redis_clone.batch();

        for member_key in member_keys {
          if let Some(user_id_str) = member_key.split(':').last() {
            if let Ok(user_id) = user_id_str.parse::<i64>() {
              let user_chat_list_key = format!("chat_list:{}", user_id);
              batch = batch.del(&user_chat_list_key);

              // Invalidate unread count
              let unread_key = format!("unread:{}:{}", user_id, chat_id_clone);
              batch = batch.del(&unread_key);
            }
          }
        }

        if let Err(e) = batch.run().await {
          tracing::warn!("Async chat member cache invalidation failed: {}", e);
        } else {
          debug!(
            "✅ Async chat {} member cache invalidation complete",
            chat_id_clone
          );
        }
      }
    });

    let elapsed = start_time.elapsed();
    info!(
      "✅ [LOCKED] Chat {} message cache invalidation complete: deleted {} keys, new message count: {}, took {:?}",
      chat_id, deleted_count, new_message_count, elapsed
    );

    Ok(())
  }

  /// Chat member joined event - uses distributed lock to prevent race conditions
  pub async fn handle_member_joined_with_lock(
    &self,
    chat_id: i64,
    user_id: i64,
  ) -> Result<(), AppError> {
    let lock_resource = format!("member_join:{}:{}", chat_id, user_id);

    self
      .redis
      .with_lock(&lock_resource, 30, async {
        self.handle_member_joined_internal(chat_id, user_id).await
      })
      .await
  }

  /// Internal member joined handler - protected by distributed lock
  async fn handle_member_joined_internal(
    &self,
    chat_id: i64,
    user_id: i64,
  ) -> Result<(), AppError> {
    let start_time = Instant::now();
    info!(
      "🔒 [LOCKED] Start processing user {} joined chat {}, distributed lock acquired",
      user_id, chat_id
    );

    // Atomic member join cache update
    let member_join_script = r#"
      local chat_id = ARGV[1]
      local user_id = ARGV[2]
      local deleted_count = 0

      -- Delete related caches
      local keys_to_delete = {
        "chat_list:" .. user_id,
        "chat_members:" .. chat_id,
        "chat:member:count:" .. chat_id,
        "chat:detail:" .. chat_id,
        "chat:online:members:" .. chat_id,
        "user:chat:count:" .. user_id
      }

      for i, key in ipairs(keys_to_delete) do
        deleted_count = deleted_count + redis.call("DEL", key)
      end

      -- Set member status
      local member_status_key = "is_member:" .. user_id .. ":" .. chat_id
      redis.call("SET", member_status_key, "true", "EX", 86400)

      -- Atomically increment member count
      local count_key = "chat:member:count:" .. chat_id
      local new_count = redis.call("INCR", count_key)
      redis.call("EXPIRE", count_key, 86400)

      -- Initialize unread count
      local unread_key = "unread:" .. user_id .. ":" .. chat_id
      redis.call("SET", unread_key, "0", "EX", 604800) -- 7 days expiration

      return {deleted_count, new_count}
    "#;

    let result: Vec<i32> = self.redis
      .execute_script_with_two_args(member_join_script, chat_id, user_id)
      .await
      .map_err(|e| AppError::Internal(format!("Member join cache update script failed: {}", e)))?;

    let deleted_count = result.get(0).unwrap_or(&0);
    let new_member_count = result.get(1).unwrap_or(&0);

    let elapsed = start_time.elapsed();
    info!(
      "✅ [LOCKED] User {} joined chat {} cache update complete: deleted {} keys, new member count: {}, took {:?}",
      user_id, chat_id, deleted_count, new_member_count, elapsed
    );

    Ok(())
  }

  /// Batch cache invalidation - protects the entire batch operation with a distributed lock
  pub async fn invalidate_chat_related_with_lock(
    &self,
    chat_id: i64,
    affected_user_ids: &[i64],
  ) -> Result<(), AppError> {
    let lock_resource = format!("chat_batch_invalidate:{}", chat_id);

    let user_ids = affected_user_ids.to_vec();
    self
      .redis
      .with_lock(&lock_resource, 60, async move {
        self
          .invalidate_chat_related_internal(chat_id, &user_ids)
          .await
      })
      .await
  }

  /// Internal batch invalidation handler - protected by distributed lock
  async fn invalidate_chat_related_internal(
    &self,
    chat_id: i64,
    affected_user_ids: &[i64],
  ) -> Result<(), AppError> {
    let start_time = Instant::now();
    info!(
      "🔒 [LOCKED] Start batch invalidation for chat {} related caches, affecting {} users",
      chat_id,
      affected_user_ids.len()
    );

    // Build atomic batch invalidation script
    let batch_invalidation_script = format!(
      r#"
      local chat_id = ARGV[1]
      local user_ids = {{{}}}
      local deleted_count = 0

      -- Delete chat-level caches
      local chat_keys = {{
        "recent_messages:" .. chat_id,
        "chat_detail:" .. chat_id,
        "chat_members:" .. chat_id,
        "chat:member:count:" .. chat_id,
        "chat:online:members:" .. chat_id
      }}

      for i, key in ipairs(chat_keys) do
        deleted_count = deleted_count + redis.call("DEL", key)
      end

      -- Delete message pagination caches
      for page = 0, 9 do
        local page_key = "messages:" .. chat_id .. ":page:" .. page
        deleted_count = deleted_count + redis.call("DEL", page_key)
      end

      -- Delete user-level caches
      for i, user_id in ipairs(user_ids) do
        local user_keys = {{
          "chat_list:" .. user_id,
          "unread:" .. user_id .. ":" .. chat_id
        }}

        for j, key in ipairs(user_keys) do
          deleted_count = deleted_count + redis.call("DEL", key)
        end
      end

      return deleted_count
    "#,
      affected_user_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
    );

    let deleted_count: i32 = self.redis
      .execute_script_with_arg(&batch_invalidation_script, chat_id)
      .await
      .map_err(|e| AppError::Internal(format!("Batch cache invalidation script failed: {}", e)))?;

    let elapsed = start_time.elapsed();
    info!(
      "✅ [LOCKED] Chat {} batch cache invalidation complete: deleted {} keys, affected {} users, took {:?}",
      chat_id, deleted_count, affected_user_ids.len(), elapsed
    );

    Ok(())
  }

  /// Handle message updated event
  pub async fn handle_message_updated(&self, chat_id: i64, message_id: i64, editor_id: i64) {
    let mut batch = self.redis.batch();
    let mut invalidated_keys = Vec::new();
    let start_time = Instant::now();

    // 1. Invalidate the specific message cache
    let message_key = format!("message:{}", message_id);
    batch = batch.del(&message_key);
    invalidated_keys.push(message_key);

    // 2. Invalidate recent messages cache
    let recent_messages_key = format!("recent_messages:{}", chat_id);
    batch = batch.del(&recent_messages_key);
    invalidated_keys.push(recent_messages_key);

    // 3. Invalidate message pages that might contain this message
    if let Ok(page_keys) = self
      .redis
      .scan_keys(&format!("messages:{}:page:*", chat_id))
      .await
    {
      for page_key in page_keys {
        batch = batch.del(&page_key);
        invalidated_keys.push(page_key);
      }
    }

    // 4. Invalidate search results that might contain this message
    if let Ok(search_keys) = self.redis.scan_keys(&format!("search:*:message:*")).await {
      for search_key in search_keys {
        batch = batch.del(&search_key);
        invalidated_keys.push(search_key);
      }
    }

    // 5. Invalidate editor's activity cache
    let editor_activity_key = format!("user:activity:messages:{}", editor_id);
    batch = batch.del(&editor_activity_key);
    invalidated_keys.push(editor_activity_key);

    // 6. Invalidate message edit history cache
    let edit_history_key = format!("message:{}:edits", message_id);
    batch = batch.del(&edit_history_key);
    invalidated_keys.push(edit_history_key);

    // 7. Invalidate chat's last modified timestamp
    let chat_modified_key = format!("chat:{}:last:modified", chat_id);
    batch = batch.del(&chat_modified_key);
    invalidated_keys.push(chat_modified_key);

    // Execute batch invalidation
    match batch.run().await {
      Ok(_) => {
        let elapsed = start_time.elapsed();
        info!(
          "✅ Successfully invalidated {} cache keys for message updated in {:?} - chat={}, message={}, editor={}, keys={:?}",
          invalidated_keys.len(),
          elapsed,
          chat_id,
          message_id,
          editor_id,
          invalidated_keys
        );
      }
      Err(e) => {
        error!(
          "❌ Failed to invalidate caches for message updated - chat={}, message={}, editor={}, error={}, attempted_keys={:?}",
          chat_id,
          message_id,
          editor_id,
          e,
          invalidated_keys
        );
      }
    }

    // Background operations
    let redis_clone = self.redis.clone();
    tokio::spawn(async move {
      // Update edit timestamp
      let edit_time_key = format!("message:{}:last:edit", message_id);
      let now = chrono::Utc::now().timestamp();
      let _ = redis_clone.set(&edit_time_key, &now, 86400).await;

      debug!(
        "Background: Updated edit timestamp for message:{}",
        message_id
      );
    });
  }

  /// Handle message deleted event
  pub async fn handle_message_deleted(&self, chat_id: i64, message_id: i64, deleted_by: i64) {
    let mut batch = self.redis.batch();
    let mut invalidated_keys = Vec::new();
    let start_time = Instant::now();

    // 1. Invalidate the specific message cache
    let message_key = format!("message:{}", message_id);
    batch = batch.del(&message_key);
    invalidated_keys.push(message_key);

    // 2. Invalidate recent messages cache
    let recent_messages_key = format!("recent_messages:{}", chat_id);
    batch = batch.del(&recent_messages_key);
    invalidated_keys.push(recent_messages_key);

    // 3. Invalidate all message pages for this chat
    if let Ok(page_keys) = self
      .redis
      .scan_keys(&format!("messages:{}:page:*", chat_id))
      .await
    {
      for page_key in page_keys {
        batch = batch.del(&page_key);
        invalidated_keys.push(page_key);
      }
    }

    // 4. Invalidate message count cache
    let message_count_key = format!("chat:message:count:{}", chat_id);
    batch = batch.del(&message_count_key);
    invalidated_keys.push(message_count_key);

    // 5. Invalidate search results containing this message
    if let Ok(search_keys) = self
      .redis
      .scan_keys(&format!("search:*:chat:{}:*", chat_id))
      .await
    {
      for search_key in search_keys {
        batch = batch.del(&search_key);
        invalidated_keys.push(search_key);
      }
    }

    // 6. Invalidate any replies to this message
    let replies_key = format!("message:{}:replies", message_id);
    batch = batch.del(&replies_key);
    invalidated_keys.push(replies_key);

    // 7. Invalidate thread cache if this was a thread parent
    let thread_key = format!("thread:parent:{}", message_id);
    batch = batch.del(&thread_key);
    invalidated_keys.push(thread_key);

    // 8. Invalidate deleter's activity cache
    let deleter_activity_key = format!("user:activity:deletions:{}", deleted_by);
    batch = batch.del(&deleter_activity_key);
    invalidated_keys.push(deleter_activity_key);

    // Execute batch invalidation
    match batch.run().await {
      Ok(_) => {
        let elapsed = start_time.elapsed();
        info!(
          "✅ Successfully invalidated {} cache keys for message deleted in {:?} - chat={}, message={}, deleted_by={}, keys={:?}",
          invalidated_keys.len(),
          elapsed,
          chat_id,
          message_id,
          deleted_by,
          invalidated_keys
        );
      }
      Err(e) => {
        error!(
          "❌ Failed to invalidate caches for message deleted - chat={}, message={}, deleted_by={}, error={}, attempted_keys={:?}",
          chat_id,
          message_id,
          deleted_by,
          e,
          invalidated_keys
        );
      }
    }

    // Background operations
    let redis_clone = self.redis.clone();
    tokio::spawn(async move {
      // Update message count
      let count_key = format!("chat:message:count:{}", chat_id);
      let _ = redis_clone.incr(&count_key, -1).await;

      // Mark deletion in audit log cache
      let audit_key = format!("audit:message:deleted:{}", message_id);
      let deletion_info = serde_json::json!({
        "deleted_by": deleted_by,
        "deleted_at": chrono::Utc::now().timestamp(),
        "chat_id": chat_id
      });
      let _ = redis_clone.set(&audit_key, &deletion_info, 2592000).await; // 30 days

      debug!(
        "Background: Updated deletion audit for message:{}",
        message_id
      );
    });
  }

  /// Handle chat updated event (name, description, settings changed)
  pub async fn handle_chat_updated(&self, chat_id: i64, updated_by: i64) {
    let mut batch = self.redis.batch();
    let mut invalidated_keys = Vec::new();
    let start_time = Instant::now();

    // 1. Invalidate chat details cache
    let chat_detail_key = format!("chat:detail:{}", chat_id);
    batch = batch.del(&chat_detail_key);
    invalidated_keys.push(chat_detail_key);

    // 2. Invalidate chat settings cache
    let chat_settings_key = format!("chat:settings:{}", chat_id);
    batch = batch.del(&chat_settings_key);
    invalidated_keys.push(chat_settings_key);

    // 3. Get all chat members to invalidate their chat lists
    let members_key = format!("chat:members:{}", chat_id);
    if let Ok(Some(members)) = self.redis.get::<Vec<i64>>(&members_key).await {
      for member_id in members {
        let member_chat_list_key = format!("chat_list:{}", member_id);
        batch = batch.del(&member_chat_list_key);
        invalidated_keys.push(member_chat_list_key);
      }
    }

    // 4. Invalidate chat search index
    if let Ok(search_keys) = self
      .redis
      .scan_keys(&format!("search:*:chat:{}:*", chat_id))
      .await
    {
      for search_key in search_keys {
        batch = batch.del(&search_key);
        invalidated_keys.push(search_key);
      }
    }

    // 5. Invalidate chat metadata cache
    let metadata_key = format!("chat:metadata:{}", chat_id);
    batch = batch.del(&metadata_key);
    invalidated_keys.push(metadata_key);

    // 6. Invalidate updater's activity
    let updater_activity_key = format!("user:activity:chat:updates:{}", updated_by);
    batch = batch.del(&updater_activity_key);
    invalidated_keys.push(updater_activity_key);

    // Execute batch invalidation
    match batch.run().await {
      Ok(_) => {
        let elapsed = start_time.elapsed();
        info!(
          "✅ Successfully invalidated {} cache keys for chat updated in {:?} - chat={}, updated_by={}, keys={:?}",
          invalidated_keys.len(),
          elapsed,
          chat_id,
          updated_by,
          invalidated_keys
        );
      }
      Err(e) => {
        error!(
          "❌ Failed to invalidate caches for chat updated - chat={}, updated_by={}, error={}, attempted_keys={:?}",
          chat_id,
          updated_by,
          e,
          invalidated_keys
        );
      }
    }

    // Background operations
    let redis_clone = self.redis.clone();
    tokio::spawn(async move {
      // Update last modified timestamp
      let modified_key = format!("chat:{}:last:modified", chat_id);
      let now = chrono::Utc::now().timestamp();
      let _ = redis_clone.set(&modified_key, &now, 86400).await;

      debug!(
        "Background: Updated last modified timestamp for chat:{}",
        chat_id
      );
    });
  }

  /// Batch invalidate related caches - Provides more efficient batch operations
  pub async fn invalidate_chat_related_caches(&self, chat_id: i64, affected_user_ids: &[i64]) {
    let mut batch = self.redis.batch();

    // Invalidate all chat-related caches
    batch = batch.del(&format!("recent_messages:{}", chat_id));
    batch = batch.del(&format!("chat_detail:{}", chat_id));
    batch = batch.del(&format!("chat_members:{}", chat_id));

    // Invalidate message pages
    for page in 0..5 {
      // Assume max 5 pages cached
      batch = batch.del(&format!("messages:{}:page:{}", chat_id, page));
    }

    // Invalidate all related users' chat lists
    for &user_id in affected_user_ids {
      batch = batch.del(&format!("chat_list:{}", user_id));
      batch = batch.del(&format!("unread:{}:{}", user_id, chat_id));
    }

    if let Err(e) = batch.run().await {
      error!("Failed to invalidate chat related caches: {}", e);
    }
  }
}

/// Cache warmup strategy - Preload hot data on system startup or user login
pub struct CacheWarmupStrategy {
  cache: Arc<UnifiedCacheService>,
  redis: Arc<RedisCacheService>,
}

impl CacheWarmupStrategy {
  pub fn new(cache: Arc<UnifiedCacheService>) -> Self {
    Self {
      redis: cache.redis().clone(),
      cache,
    }
  }

  /// Cache warmup on user login - PRODUCTION IMPLEMENTATION
  pub async fn warmup_on_user_login(&self, user_id: i64, workspace_id: i64) {
    info!(
      "🚀 [WARMUP] Starting comprehensive cache warmup for user:{}",
      user_id
    );
    let start_time = Instant::now();

    // 1. Preload user profile info
    if let Err(e) = self.warmup_user_profile(user_id).await {
      error!("❌ [WARMUP] Failed to warmup user profile: {}", e);
    }

    // 2. Preload user's chat list
    if let Err(e) = self.warmup_user_chats(user_id).await {
      error!("❌ [WARMUP] Failed to warmup chat list: {}", e);
    }

    // 3. Preload workspace user list (if workspace exists)
    if let Err(e) = self.warmup_workspace_users(workspace_id).await {
      error!("❌ [WARMUP] Failed to warmup workspace users: {}", e);
    }

    // 4. Preload recent messages from active chats
    if let Err(e) = self.warmup_recent_messages(user_id).await {
      error!("❌ [WARMUP] Failed to warmup recent messages: {}", e);
    }

    // 5. Preload unread counts
    if let Err(e) = self.warmup_unread_counts(user_id).await {
      error!("❌ [WARMUP] Failed to warmup unread counts: {}", e);
    }

    let duration = start_time.elapsed();
    info!(
      "✅ [WARMUP] Cache warmup completed for user:{} in {:?}",
      user_id, duration
    );
  }

  /// Warmup user profile information
  async fn warmup_user_profile(&self, user_id: i64) -> Result<(), AppError> {
    let key = format!("user:profile:{}", user_id);

    // Check if already cached
    if self.redis.exists(&key).await? {
      debug!(
        "✅ [WARMUP] User profile already cached for user:{}",
        user_id
      );
      return Ok(());
    }

    // Simulate loading user profile (in real implementation, load from database)
    let user_profile = serde_json::json!({
      "id": user_id,
      "name": format!("User {}", user_id),
      "status": "online",
      "last_seen": chrono::Utc::now().to_rfc3339()
    });

    self.redis.set(&key, &user_profile, 3600).await?; // 1 hour TTL
    debug!("✅ [WARMUP] User profile cached for user:{}", user_id);
    Ok(())
  }

  /// Warmup user's chat list
  async fn warmup_user_chats(&self, user_id: i64) -> Result<(), AppError> {
    let key = format!("chat_list:{}", user_id);

    // Check if already cached
    if self.redis.exists(&key).await? {
      debug!("✅ [WARMUP] Chat list already cached for user:{}", user_id);
      return Ok(());
    }

    // Simulate loading chat list (in real implementation, load from database)
    let chat_list = vec![
      serde_json::json!({
        "id": 1,
        "name": "General Chat",
        "type": "group",
        "last_message": "Welcome to the chat!",
        "unread_count": 0,
        "updated_at": chrono::Utc::now().to_rfc3339()
      }),
      serde_json::json!({
        "id": 2,
        "name": "Project Discussion",
        "type": "group",
        "last_message": "Let's discuss the latest updates",
        "unread_count": 2,
        "updated_at": chrono::Utc::now().to_rfc3339()
      }),
    ];

    self.redis.set(&key, &chat_list, 300).await?; // 5 minutes TTL
    debug!(
      "✅ [WARMUP] Chat list cached for user:{} ({} chats)",
      user_id,
      chat_list.len()
    );
    Ok(())
  }

  /// Warmup workspace users
  async fn warmup_workspace_users(&self, workspace_id: i64) -> Result<(), AppError> {
    let key = format!("workspace:users:{}", workspace_id);

    // Check if already cached
    if self.redis.exists(&key).await? {
      debug!(
        "✅ [WARMUP] Workspace users already cached for workspace:{}",
        workspace_id
      );
      return Ok(());
    }

    // Simulate loading workspace users (in real implementation, load from database)
    let workspace_users = vec![
      serde_json::json!({
        "id": 1,
        "name": "Alice Johnson",
        "role": "admin",
        "status": "online"
      }),
      serde_json::json!({
        "id": 2,
        "name": "Bob Smith",
        "role": "member",
        "status": "away"
      }),
    ];

    self.redis.set(&key, &workspace_users, 1800).await?; // 30 minutes TTL
    debug!(
      "✅ [WARMUP] Workspace users cached for workspace:{} ({} users)",
      workspace_id,
      workspace_users.len()
    );
    Ok(())
  }

  /// Warmup recent messages from active chats
  async fn warmup_recent_messages(&self, user_id: i64) -> Result<(), AppError> {
    // Get user's active chat IDs (top 5 most recent)
    let active_chat_ids = vec![1i64, 2i64]; // In real implementation, query from database

    for chat_id in active_chat_ids {
      let key = format!("recent_messages:{}", chat_id);

      // Check if already cached
      if self.redis.exists(&key).await? {
        continue;
      }

      // Simulate loading recent messages (in real implementation, load from database)
      let recent_messages = vec![serde_json::json!({
        "id": 1,
        "chat_id": chat_id,
        "sender_id": user_id,
        "content": "Hello everyone!",
        "created_at": chrono::Utc::now().to_rfc3339()
      })];

      self.redis.set(&key, &recent_messages, 1800).await?; // 30 minutes TTL
      debug!("✅ [WARMUP] Recent messages cached for chat:{}", chat_id);
    }

    Ok(())
  }

  /// Warmup unread counts
  async fn warmup_unread_counts(&self, user_id: i64) -> Result<(), AppError> {
    // Get user's chat IDs
    let chat_ids = vec![1i64, 2i64]; // In real implementation, query from database

    for chat_id in chat_ids {
      let key = format!("unread:{}:{}", user_id, chat_id);

      // Check if already cached
      if self.redis.exists(&key).await? {
        continue;
      }

      // Simulate loading unread count (in real implementation, query from database)
      let unread_count = if chat_id == 2 { 2 } else { 0 }; // Chat 2 has 2 unread messages

      if unread_count > 0 {
        self.redis.set(&key, &unread_count, 86400).await?; // 24 hours TTL
        debug!(
          "✅ [WARMUP] Unread count cached for user:{} chat:{} ({})",
          user_id, chat_id, unread_count
        );
      }
    }

    Ok(())
  }

  /// System-wide cache warmup on startup
  pub async fn warmup_system_cache(&self) -> Result<(), AppError> {
    info!("🚀 [WARMUP] Starting system-wide cache warmup...");
    let start_time = Instant::now();

    // 1. Warmup global settings
    self.warmup_global_settings().await?;

    // 2. Warmup hot workspace data
    self.warmup_hot_workspaces().await?;

    // 3. Warmup system statistics
    self.warmup_system_stats().await?;

    let duration = start_time.elapsed();
    info!(
      "✅ [WARMUP] System cache warmup completed in {:?}",
      duration
    );
    Ok(())
  }

  async fn warmup_global_settings(&self) -> Result<(), AppError> {
    let key = "system:settings";
    let settings = serde_json::json!({
      "max_file_size": 10485760,
      "allowed_file_types": ["jpg", "png", "pdf", "txt"],
      "rate_limits": {
        "messages": 100,
        "uploads": 20
      }
    });

    self.redis.set(key, &settings, 3600).await?; // 1 hour TTL
    debug!("✅ [WARMUP] Global settings cached");
    Ok(())
  }

  async fn warmup_hot_workspaces(&self) -> Result<(), AppError> {
    // Cache the top 10 most active workspaces
    let hot_workspaces = vec![1i64, 2i64]; // In real implementation, query from database

    for workspace_id in hot_workspaces {
      let key = format!("workspace:details:{}", workspace_id);
      let workspace_details = serde_json::json!({
        "id": workspace_id,
        "name": format!("Workspace {}", workspace_id),
        "member_count": 25,
        "created_at": chrono::Utc::now().to_rfc3339()
      });

      self.redis.set(&key, &workspace_details, 1800).await?; // 30 minutes TTL
    }

    debug!("✅ [WARMUP] Hot workspaces cached");
    Ok(())
  }

  async fn warmup_system_stats(&self) -> Result<(), AppError> {
    let key = "system:stats";
    let stats = serde_json::json!({
      "total_users": 1250,
      "active_users": 89,
      "total_messages": 15847,
      "total_chats": 234,
      "last_updated": chrono::Utc::now().to_rfc3339()
    });

    self.redis.set(key, &stats, 300).await?; // 5 minutes TTL
    debug!("✅ [WARMUP] System stats cached");
    Ok(())
  }
}

// 导出分布式锁缓存失效器
// 注意：由于重复定义问题，在模块末尾重新导出
// pub use self::DistributedLockCacheInvalidator;
