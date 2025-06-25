use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, Pipeline};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

use crate::AppError;
use fechatter_core::chat::ChatSidebar;
use fechatter_core::models::Message;

pub mod ttl {
  pub const SHORT: u64 = 300;
  pub const MEDIUM: u64 = 1800;
  pub const LONG: u64 = 3600;
  pub const DAY: u64 = 86400;
}

pub struct RedisCacheService {
  client: Arc<Client>,
  conn: Arc<RwLock<MultiplexedConnection>>,
  prefix: String,
}

pub struct BatchOp<'a> {
  cache: &'a RedisCacheService,
  pipeline: Pipeline,
}

impl<'a> BatchOp<'a> {
  pub fn set<T: Serialize>(mut self, key: &str, val: &T, ttl: u64) -> Result<Self, AppError> {
    let full_key = self.cache.make_key(key);
    let data = serde_json::to_vec(val)?;
    self.pipeline.set_ex(&full_key, data, ttl);
    Ok(self)
  }

  pub fn del(mut self, key: &str) -> Self {
    let full_key = self.cache.make_key(key);
    self.pipeline.del(&full_key);
    self
  }

  pub async fn run(self) -> Result<(), AppError> {
    let mut conn = self.cache.conn.write().await;
    let _: redis::Value = self.pipeline.query_async(&mut *conn).await?;
    Ok(())
  }
}

impl RedisCacheService {
  pub async fn new(url: &str, prefix: &str) -> Result<Self, AppError> {
    let client = Arc::new(Client::open(url)?);
    let conn = client.get_multiplexed_async_connection().await?;

    Ok(Self {
      client,
      conn: Arc::new(RwLock::new(conn)),
      prefix: prefix.to_string(),
    })
  }

  fn make_key(&self, key: &str) -> String {
    format!("{}:{}", self.prefix, key)
  }

  pub async fn set<T: Serialize>(&self, key: &str, val: &T, ttl: u64) -> Result<(), AppError> {
    let mut conn = self.conn.write().await;
    let full_key = self.make_key(key);
    let data = serde_json::to_vec(val)?;
    conn.set_ex::<_, _, ()>(&full_key, data, ttl).await?;
    Ok(())
  }

  pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, AppError> {
    let mut conn = self.conn.write().await;
    let full_key = self.make_key(key);

    let data: Option<Vec<u8>> = conn.get(&full_key).await?;
    match data {
      Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
      None => Ok(None),
    }
  }

  pub async fn mget<T: DeserializeOwned>(&self, keys: &[&str]) -> Result<Vec<Option<T>>, AppError> {
    if keys.is_empty() {
      return Ok(vec![]);
    }

    let mut conn = self.conn.write().await;
    let full_keys: Vec<String> = keys.iter().map(|k| self.make_key(k)).collect();
    let values: Vec<Option<Vec<u8>>> = conn.get(&full_keys).await?;

    let mut results = Vec::new();
    for val in values {
      match val {
        Some(data) => match serde_json::from_slice(&data) {
          Ok(item) => results.push(Some(item)),
          Err(e) => {
            warn!("Failed to deserialize: {}", e);
            results.push(None);
          }
        },
        None => results.push(None),
      }
    }
    Ok(results)
  }

  pub async fn del(&self, key: &str) -> Result<bool, AppError> {
    let mut conn = self.conn.write().await;
    let full_key = self.make_key(key);
    let deleted: bool = conn.del(&full_key).await?;
    Ok(deleted)
  }

  pub async fn exists(&self, key: &str) -> Result<bool, AppError> {
    let mut conn = self.conn.write().await;
    let full_key = self.make_key(key);
    let exists: bool = conn.exists(&full_key).await?;
    Ok(exists)
  }

  pub async fn del_pattern(&self, pattern: &str) -> Result<u64, AppError> {
    let mut conn = self.conn.write().await;
    let full_pattern = self.make_key(pattern);

    // Use SCAN to find all matching keys
    let mut keys_to_delete = Vec::new();
    let mut cursor = "0".to_string();

    loop {
      let result: (String, Vec<String>) = redis::cmd("SCAN")
        .arg(&cursor)
        .arg("MATCH")
        .arg(&full_pattern)
        .arg("COUNT")
        .arg(100)
        .query_async(&mut *conn)
        .await?;

      cursor = result.0;
      keys_to_delete.extend(result.1);

      if cursor == "0" {
        break;
      }
    }

    if keys_to_delete.is_empty() {
      return Ok(0);
    }

    // Delete all matching keys
    let deleted: u64 = conn.del(&keys_to_delete).await?;
    Ok(deleted)
  }

  pub async fn incr(&self, key: &str, by: i64) -> Result<i64, AppError> {
    let mut conn = self.conn.write().await;
    let full_key = self.make_key(key);
    let result: i64 = conn.incr(&full_key, by).await?;
    Ok(result)
  }

  /// Scan for keys matching a pattern
  pub async fn scan_keys(&self, pattern: &str) -> Result<Vec<String>, AppError> {
    let mut conn = self.conn.write().await;
    let full_pattern = self.make_key(pattern);
    
    let mut all_keys = Vec::new();
    let mut cursor = "0".to_string();
    
    loop {
      let result: (String, Vec<String>) = redis::cmd("SCAN")
        .arg(&cursor)
        .arg("MATCH")
        .arg(&full_pattern)
        .arg("COUNT")
        .arg(100)
        .query_async(&mut *conn)
        .await?;
      
      cursor = result.0;
      
      // Strip the prefix from keys before returning
      for key in result.1 {
        if let Some(stripped) = key.strip_prefix(&format!("{}:", self.prefix)) {
          all_keys.push(stripped.to_string());
        } else {
          all_keys.push(key);
        }
      }
      
      if cursor == "0" {
        break;
      }
    }
    
    Ok(all_keys)
  }

  pub fn batch(&self) -> BatchOp<'_> {
    BatchOp {
      cache: self,
      pipeline: redis::pipe(),
    }
  }

  pub async fn try_lock(&self, resource: &str, ttl: u64, token: &str) -> Result<bool, AppError> {
    let key = format!("lock:{}", resource);
    let full_key = self.make_key(&key);

    let script = r#"
            if redis.call("SET", KEYS[1], ARGV[1], "NX", "EX", ARGV[2]) then
                return 1
            else
                return 0
            end
        "#;

    let mut conn = self.conn.write().await;
    let result: i32 = redis::Script::new(script)
      .key(&full_key)
      .arg(token)
      .arg(ttl as i64)
      .invoke_async(&mut *conn)
      .await?;

    Ok(result == 1)
  }

  /// 释放分布式锁 - 原子性检查token并删除
  pub async fn release_lock(&self, resource: &str, token: &str) -> Result<bool, AppError> {
    let key = format!("lock:{}", resource);
    let full_key = self.make_key(&key);

    // Lua脚本确保原子性：只有token匹配才删除
    let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

    let mut conn = self.conn.write().await;
    let result: i32 = redis::Script::new(script)
      .key(&full_key)
      .arg(token)
      .invoke_async(&mut *conn)
      .await?;

    Ok(result == 1)
  }

  /// 分布式锁包装器 - 自动获取锁、执行操作、释放锁
  pub async fn with_lock<F, R>(&self, resource: &str, ttl: u64, operation: F) -> Result<R, AppError>
  where
    F: std::future::Future<Output = Result<R, AppError>>,
  {
    let token = uuid::Uuid::new_v4().to_string();
    let lock_acquired = self.try_lock(resource, ttl, &token).await?;
    
    if !lock_acquired {
      return Err(AppError::Internal(format!("Failed to acquire lock for resource: {}", resource)));
    }

    // 执行操作
    let result = operation.await;
    
    // 释放锁
    if let Err(e) = self.release_lock(resource, &token).await {
      tracing::warn!("Failed to release lock for resource {}: {}", resource, e);
    }

    result
  }

  // Business methods - unified with database schema

  pub async fn cache_chat_list(
    &self,
    user_id: i64,
    chats: Vec<ChatSidebar>,
  ) -> Result<(), AppError> {
    let key = format!("chat_list:{}", user_id);
    self.set(&key, &chats, ttl::SHORT).await
  }

  pub async fn get_chat_list(&self, user_id: i64) -> Result<Option<Vec<ChatSidebar>>, AppError> {
    let key = format!("chat_list:{}", user_id);
    self.get(&key).await
  }

  pub async fn cache_message(&self, message_id: i64, message: &Message) -> Result<(), AppError> {
    let key = format!("message:{}", message_id);
    self.set(&key, message, ttl::LONG).await
  }

  pub async fn get_message(&self, message_id: i64) -> Result<Option<Message>, AppError> {
    let key = format!("message:{}", message_id);
    self.get(&key).await
  }

  pub async fn cache_recent_messages(
    &self,
    chat_id: i64,
    messages: Vec<Message>,
  ) -> Result<(), AppError> {
    let key = format!("recent_messages:{}", chat_id);
    self.set(&key, &messages, ttl::MEDIUM).await
  }

  pub async fn get_recent_messages(&self, chat_id: i64) -> Result<Option<Vec<Message>>, AppError> {
    let key = format!("recent_messages:{}", chat_id);
    self.get(&key).await
  }

  pub async fn cache_unread_count(
    &self,
    user_id: i64,
    chat_id: i64,
    count: i64,
  ) -> Result<(), AppError> {
    let key = format!("unread:{}:{}", user_id, chat_id);
    self.set(&key, &count, ttl::MEDIUM).await
  }

  pub async fn get_unread_count(
    &self,
    user_id: i64,
    chat_id: i64,
  ) -> Result<Option<i64>, AppError> {
    let key = format!("unread:{}:{}", user_id, chat_id);
    self.get(&key).await
  }

  pub async fn save_messages(&self, messages: Vec<(i64, Message)>) -> Result<(), AppError> {
    let mut batch = self.batch();

    for (msg_id, msg) in &messages {
      let key = format!("message:{}", msg_id);
      batch = batch.set(&key, msg, ttl::LONG)?;
    }

    batch.run().await
  }

  pub async fn set_users_online(&self, user_ids: &[i64]) -> Result<(), AppError> {
    let mut batch = self.batch();

    for &user_id in user_ids {
      let key = format!("online:{}", user_id);
      batch = batch.set(&key, &true, ttl::SHORT)?;
    }

    batch.run().await
  }

  pub async fn who_typing(&self, chat_id: i64) -> Result<Vec<i64>, AppError> {
    let pattern = format!("typing:{}:*", chat_id);
    let full_pattern = self.make_key(&pattern);

    let mut conn = self.conn.write().await;
    let mut users = Vec::new();

    let mut iter: redis::AsyncIter<String> = conn.scan_match(&full_pattern).await?;

    while let Some(key) = iter.next_item().await {
      if let Some(user_str) = key.split(':').last() {
        if let Ok(user_id) = user_str.parse::<i64>() {
          users.push(user_id);
        }
      }
    }

    Ok(users)
  }

  pub async fn invalidate_recent_messages(&self, chat_id: i64) -> Result<(), AppError> {
    let key = format!("recent_messages:{}", chat_id);
    self.del(&key).await?;
    Ok(())
  }

  pub async fn invalidate_unread_count(&self, user_id: i64, chat_id: i64) -> Result<(), AppError> {
    let key = format!("unread:{}:{}", user_id, chat_id);
    self.del(&key).await?;
    Ok(())
  }

  // New methods for better consistency
  pub async fn save_chat_list(
    &self,
    user_id: i64,
    chats: Vec<ChatSidebar>,
  ) -> Result<(), AppError> {
    self.cache_chat_list(user_id, chats).await
  }

  pub async fn load_chat_list(&self, user_id: i64) -> Result<Option<Vec<ChatSidebar>>, AppError> {
    self.get_chat_list(user_id).await
  }

  // ============================================================================
  // Optimized cache operations - Use INCR/DECR for unread counts
  // ============================================================================

  /// Increment unread count
  pub async fn incr_unread_count(
    &self,
    user_id: i64,
    chat_id: i64,
    increment: i64,
  ) -> Result<i64, AppError> {
    let key = format!("unread:{}:{}", user_id, chat_id);
    let full_key = self.make_key(&key);

    let mut conn = self.conn.write().await;
    let new_count: i64 = conn.incr(&full_key, increment).await?;

    // Set expiration time (7 days)
    let _: bool = conn.expire(&full_key, (ttl::DAY * 7) as i64).await?;

    Ok(new_count)
  }

  /// Decrement unread count
  pub async fn decr_unread_count(
    &self,
    user_id: i64,
    chat_id: i64,
    decrement: i64,
  ) -> Result<i64, AppError> {
    let key = format!("unread:{}:{}", user_id, chat_id);
    let full_key = self.make_key(&key);

    let mut conn = self.conn.write().await;
    let new_count: i64 = conn.decr(&full_key, decrement).await?;

    // Delete key if count is 0 or negative
    if new_count <= 0 {
      let _: () = conn.del(&full_key).await?;
      Ok(0)
    } else {
      Ok(new_count)
    }
  }

  /// Reset unread count to 0
  pub async fn reset_unread_count(&self, user_id: i64, chat_id: i64) -> Result<(), AppError> {
    let key = format!("unread:{}:{}", user_id, chat_id);
    self.del(&key).await?;
    Ok(())
  }

  /// Get multiple unread counts
  pub async fn get_unread_counts(
    &self,
    user_id: i64,
    chat_ids: &[i64],
  ) -> Result<Vec<(i64, i64)>, AppError> {
    if chat_ids.is_empty() {
      return Ok(vec![]);
    }

    let keys: Vec<String> = chat_ids
      .iter()
      .map(|&chat_id| format!("unread:{}:{}", user_id, chat_id))
      .collect();

    let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
    let counts: Vec<Option<i64>> = self.mget(&key_refs).await?;

    Ok(
      chat_ids
        .iter()
        .zip(counts.iter())
        .map(|(&chat_id, &count)| (chat_id, count.unwrap_or(0)))
        .collect(),
    )
  }

  // ============================================================================
  // Optimized message page caching - Only invalidate latest page
  // ============================================================================

  /// Cache message page (with longer TTL)
  pub async fn cache_message_page_optimized(
    &self,
    chat_id: i64,
    page: u32,
    messages: Vec<Message>,
    is_latest_page: bool,
  ) -> Result<(), AppError> {
    let key = format!("messages:{}:page:{}", chat_id, page);

    // Latest page uses shorter TTL (5 mins), history pages use longer TTL (1 hour)
    let ttl = if is_latest_page {
      ttl::SHORT
    } else {
      ttl::LONG
    };

    self.set(&key, &messages, ttl).await
  }

  /// Smart invalidation of message cache - Only invalidate latest page
  pub async fn invalidate_latest_message_page(&self, chat_id: i64) -> Result<(), AppError> {
    // Only invalidate page 0 (latest page)
    let key = format!("messages:{}:page:0", chat_id);
    self.del(&key).await?;

    // Also invalidate "recent messages" cache
    let recent_key = format!("recent_messages:{}", chat_id);
    self.del(&recent_key).await?;

    Ok(())
  }

  // ============================================================================
  // Optimized chat list caching - Use longer TTL
  // ============================================================================

  /// Cache chat list (with longer TTL)
  pub async fn cache_chat_list_optimized(
    &self,
    user_id: i64,
    chats: Vec<ChatSidebar>,
  ) -> Result<(), AppError> {
    let key = format!("chat_list:{}", user_id);
    // Use 30 min TTL, rely on event invalidation rather than TTL expiration
    self.set(&key, &chats, ttl::MEDIUM * 2).await
  }

  // ============================================================================
  // Cache warmup functionality
  // ============================================================================

  /// Cache warmup on user login
  pub async fn warmup_user_cache(
    &self,
    user_id: i64,
    chat_list: Vec<ChatSidebar>,
    unread_counts: Vec<(i64, i64)>,
  ) -> Result<(), AppError> {
    let start = std::time::Instant::now();

    // 1. Warmup chat list
    self.cache_chat_list_optimized(user_id, chat_list).await?;

    // 2. Warmup unread counts
    for (chat_id, count) in unread_counts {
      if count > 0 {
        let key = format!("unread:{}:{}", user_id, chat_id);
        self.set(&key, &count, ttl::DAY * 7).await?;
      }
    }

    let duration = start.elapsed();
    tracing::info!(
      "Cache warmup completed for user {} in {:?}",
      user_id,
      duration
    );

    Ok(())
  }

  /// Load hot data on system startup
  pub async fn warmup_hot_data(
    &self,
    hot_chat_ids: Vec<i64>,
    recent_messages: Vec<(i64, Vec<Message>)>,
  ) -> Result<(), AppError> {
    let start = std::time::Instant::now();
    let mut batch = self.batch();

    // Batch cache recent messages for hot chats
    for (chat_id, messages) in &recent_messages {
      let key = format!("recent_messages:{}", chat_id);
      batch = batch.set(&key, messages, ttl::MEDIUM)?;
    }

    batch.run().await?;

    let duration = start.elapsed();
    tracing::info!(
      "Hot data warmup completed for {} chats in {:?}",
      hot_chat_ids.len(),
      duration
    );

    Ok(())
  }

  // ============================================================================
  // Cache monitoring and statistics
  // ============================================================================

  /// Get cache statistics
  pub async fn get_cache_stats(&self) -> Result<CacheStats, AppError> {
    let mut conn = self.conn.write().await;

    // Use INFO stats command to get statistics
    let info: String = redis::cmd("INFO")
      .arg("stats")
      .query_async(&mut *conn)
      .await?;

    // Parse statistics
    let mut stats = CacheStats::default();
    for line in info.lines() {
      if let Some(hits) = line.strip_prefix("keyspace_hits:") {
        stats.total_hits = hits.parse().unwrap_or(0);
      } else if let Some(misses) = line.strip_prefix("keyspace_misses:") {
        stats.total_misses = misses.parse().unwrap_or(0);
      }
    }

    // Calculate hit rate
    let total = stats.total_hits + stats.total_misses;
    stats.hit_rate = if total > 0 {
      (stats.total_hits as f64) / (total as f64)
    } else {
      0.0
    };

    Ok(stats)
  }

  /// Record cache operation metrics (for monitoring)
  async fn record_cache_metric(&self, operation: &str, hit: bool, duration_ms: u64) {
    // Can integrate with Prometheus or other monitoring systems here
    tracing::debug!(
      operation = operation,
      hit = hit,
      duration_ms = duration_ms,
      "Cache operation metric"
    );
  }

  /// Execute Redis script - Production-grade script execution with error handling
  pub async fn execute_script<T: redis::FromRedisValue>(
    &self,
    script: &str,
  ) -> Result<T, AppError> {
    let mut conn = self.conn.write().await;
    let result = redis::Script::new(script)
      .invoke_async(&mut *conn)
      .await
      .map_err(|e| AppError::Internal(format!("Redis script execution failed: {}", e)))?;
    Ok(result)
  }

  /// Execute Redis script with single argument - Optimized for common use cases
  pub async fn execute_script_with_arg<T: redis::FromRedisValue>(
    &self,
    script: &str,
    arg: impl redis::ToRedisArgs,
  ) -> Result<T, AppError> {
    let mut conn = self.conn.write().await;
    let result = redis::Script::new(script)
      .arg(arg)
      .invoke_async(&mut *conn)
      .await
      .map_err(|e| AppError::Internal(format!("Redis script execution failed: {}", e)))?;
    Ok(result)
  }

  /// Execute Redis script with two arguments - Type-safe script execution
  pub async fn execute_script_with_two_args<T: redis::FromRedisValue>(
    &self,
    script: &str,
    arg1: impl redis::ToRedisArgs,
    arg2: impl redis::ToRedisArgs,
  ) -> Result<T, AppError> {
    let mut conn = self.conn.write().await;
    let result = redis::Script::new(script)
      .arg(arg1)
      .arg(arg2)
      .invoke_async(&mut *conn)
      .await
      .map_err(|e| AppError::Internal(format!("Redis script execution failed: {}", e)))?;
    Ok(result)
  }
}

#[derive(Debug, Default)]
pub struct CacheStats {
  pub total_hits: u64,
  pub total_misses: u64,
  pub hit_rate: f64,
  pub memory_usage_bytes: u64,
  pub total_keys: u64,
}

impl From<redis::RedisError> for AppError {
  fn from(err: redis::RedisError) -> Self {
    AppError::RedisError(err.to_string())
  }
}

impl From<serde_json::Error> for AppError {
  fn from(err: serde_json::Error) -> Self {
    AppError::RedisError(format!("JSON error: {}", err))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_key_format() {
    let prefix = "test".to_string();
    let key = format!("{}:{}", prefix, "user:123");
    assert_eq!(key, "test:user:123");
  }

  #[cfg(feature = "integration_tests")]
  mod integration {
    use super::*;
    use std::sync::Arc;

    async fn get_cache() -> RedisCacheService {
      RedisCacheService::new("redis://:fechatter_redis_pass@localhost:6379", "test")
        .await
        .expect("Redis not available")
    }

    #[tokio::test]
    async fn basic_stuff() {
      let cache = get_cache().await;

      cache.set("foo", &"bar", 60).await.unwrap();
      let val: Option<String> = cache.get("foo").await.unwrap();
      assert_eq!(val, Some("bar".to_string()));

      assert!(cache.del("foo").await.unwrap());
      let val: Option<String> = cache.get("foo").await.unwrap();
      assert_eq!(val, None);
    }
  }
}
