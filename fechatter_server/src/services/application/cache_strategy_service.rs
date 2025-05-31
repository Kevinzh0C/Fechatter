//! # Cache Strategy Service - 缓存策略服务
//!
//! ## 单一职责
//! - 统一缓存策略管理
//! - 缓存键生成规范化
//! - TTL策略配置
//! - 缓存失效策略

use fechatter_core::contracts::CacheService;
use std::sync::Arc;
use tracing::warn;

/// 缓存策略服务 - 单一职责：统一缓存管理
pub struct CacheStrategyService {
  cache: Arc<dyn CacheService>,
}

impl CacheStrategyService {
  // ========================================================================
  // CONSTRUCTOR - 构造函数
  // ========================================================================

  pub fn new(cache: Arc<dyn CacheService>) -> Self {
    Self { cache }
  }

  // ========================================================================
  // CACHE KEY GENERATION - 缓存键生成 (单一职责)
  // ========================================================================

  /// 生成用户资料缓存键
  pub fn user_profile_key(user_id: i64) -> String {
    format!("user:profile:{}", user_id)
  }

  /// 生成用户聊天列表缓存键
  pub fn user_chats_key(user_id: i64) -> String {
    format!("user:{}:chats", user_id)
  }

  /// 生成聊天详情缓存键
  pub fn chat_detail_key(chat_id: i64) -> String {
    format!("chat:detail:{}", chat_id)
  }

  /// 生成聊天消息缓存键
  pub fn chat_messages_key(chat_id: i64, page: i32) -> String {
    format!("chat:{}:messages:page:{}", chat_id, page)
  }

  /// 生成用户未读计数缓存键
  pub fn unread_count_key(user_id: i64) -> String {
    format!("user:{}:unread", user_id)
  }

  // ========================================================================
  // TTL STRATEGY - TTL策略 (单一职责)
  // ========================================================================

  /// 用户资料TTL - 30分钟 (相对稳定)
  pub const USER_PROFILE_TTL: u64 = 1800;

  /// 聊天列表TTL - 10分钟 (频繁访问)
  pub const CHAT_LIST_TTL: u64 = 600;

  /// 聊天详情TTL - 30分钟 (相对稳定)
  pub const CHAT_DETAIL_TTL: u64 = 1800;

  /// 消息页面TTL - 1小时 (历史数据)
  pub const MESSAGE_PAGE_TTL: u64 = 3600;

  /// 未读计数TTL - 5分钟 (实时性要求高)
  pub const UNREAD_COUNT_TTL: u64 = 300;

  // ========================================================================
  // CACHE OPERATIONS - 缓存操作 (单一职责)
  // ========================================================================

  /// 获取缓存数据
  pub async fn get<T>(&self, key: &str) -> Result<Option<T>, fechatter_core::error::CoreError>
  where
    T: serde::de::DeserializeOwned,
  {
    self.cache.get(key).await
  }

  /// 设置缓存数据
  pub async fn set<T>(
    &self,
    key: &str,
    value: &T,
    ttl: u64,
  ) -> Result<(), fechatter_core::error::CoreError>
  where
    T: serde::Serialize,
  {
    self.cache.set(key, value, ttl).await
  }

  /// 删除单个缓存
  pub async fn delete(&self, key: &str) -> Result<(), fechatter_core::error::CoreError> {
    self.cache.delete(key).await
  }

  // ========================================================================
  // BATCH CACHE INVALIDATION - 批量缓存失效 (单一职责)
  // ========================================================================

  /// 清除用户相关缓存
  pub async fn invalidate_user_caches(&self, user_id: i64) {
    let keys = vec![
      Self::user_profile_key(user_id),
      Self::user_chats_key(user_id),
      Self::unread_count_key(user_id),
    ];

    self.invalidate_keys(keys).await;
  }

  /// 清除聊天相关缓存
  pub async fn invalidate_chat_caches(&self, chat_id: i64, affected_user_ids: &[i64]) {
    let mut keys = vec![Self::chat_detail_key(chat_id)];

    // 添加受影响用户的聊天列表缓存
    for &user_id in affected_user_ids {
      keys.push(Self::user_chats_key(user_id));
      keys.push(Self::unread_count_key(user_id));
    }

    self.invalidate_keys(keys).await;
  }

  /// 清除消息相关缓存
  pub async fn invalidate_message_caches(&self, chat_id: i64, affected_user_ids: &[i64]) {
    let mut keys = Vec::new();

    // 清除消息页面缓存 (前5页)
    for page in 1..=5 {
      keys.push(Self::chat_messages_key(chat_id, page));
    }

    // 清除用户未读计数
    for &user_id in affected_user_ids {
      keys.push(Self::unread_count_key(user_id));
    }

    self.invalidate_keys(keys).await;
  }

  // ========================================================================
  // PRIVATE HELPER METHODS - 私有辅助方法
  // ========================================================================

  /// 批量删除缓存键 (内部单一职责方法)
  async fn invalidate_keys(&self, keys: Vec<String>) {
    for key in keys {
      if let Err(e) = self.cache.delete(&key).await {
        warn!("Failed to invalidate cache key {}: {}", key, e);
      }
    }
  }
}

// ============================================================================
// CACHE STRATEGY ENUMS - 缓存策略枚举
// ============================================================================

/// 缓存数据类型
#[derive(Debug, Clone)]
pub enum CacheDataType {
  UserProfile,
  ChatList,
  ChatDetail,
  MessagePage,
  UnreadCount,
}

impl CacheDataType {
  /// 获取对应的TTL
  pub fn ttl(&self) -> u64 {
    match self {
      CacheDataType::UserProfile => CacheStrategyService::USER_PROFILE_TTL,
      CacheDataType::ChatList => CacheStrategyService::CHAT_LIST_TTL,
      CacheDataType::ChatDetail => CacheStrategyService::CHAT_DETAIL_TTL,
      CacheDataType::MessagePage => CacheStrategyService::MESSAGE_PAGE_TTL,
      CacheDataType::UnreadCount => CacheStrategyService::UNREAD_COUNT_TTL,
    }
  }
}

/// 缓存失效模式
#[derive(Debug, Clone)]
pub enum InvalidationPattern {
  /// 单用户相关数据失效
  SingleUser(i64),
  /// 聊天相关数据失效
  ChatRelated(i64, Vec<i64>),
  /// 消息相关数据失效
  MessageRelated(i64, Vec<i64>),
  /// 全局数据失效
  Global,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cache_key_generation() {
    assert_eq!(
      CacheStrategyService::user_profile_key(123),
      "user:profile:123"
    );
    assert_eq!(CacheStrategyService::user_chats_key(456), "user:456:chats");
    assert_eq!(
      CacheStrategyService::chat_detail_key(789),
      "chat:detail:789"
    );
    assert_eq!(
      CacheStrategyService::chat_messages_key(100, 2),
      "chat:100:messages:page:2"
    );
    assert_eq!(
      CacheStrategyService::unread_count_key(200),
      "user:200:unread"
    );
  }

  #[test]
  fn test_cache_data_type_ttl() {
    assert_eq!(CacheDataType::UserProfile.ttl(), 1800);
    assert_eq!(CacheDataType::ChatList.ttl(), 600);
    assert_eq!(CacheDataType::ChatDetail.ttl(), 1800);
    assert_eq!(CacheDataType::MessagePage.ttl(), 3600);
    assert_eq!(CacheDataType::UnreadCount.ttl(), 300);
  }
}
