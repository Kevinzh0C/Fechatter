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
    self.pipeline.query_async::<()>(&mut *conn).await?;
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
    let count: i32 = conn.del(&full_key).await?;
    Ok(count > 0)
  }

  pub async fn incr(&self, key: &str, by: i64) -> Result<i64, AppError> {
    let mut conn = self.conn.write().await;
    let full_key = self.make_key(key);
    let result: i64 = conn.incr(&full_key, by).await?;
    Ok(result)
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
      RedisCacheService::new("redis://localhost:6379", "test")
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
