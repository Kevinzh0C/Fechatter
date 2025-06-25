//! # Event-driven Cache Invalidation System (TEMPORARILY DISABLED)
//!
//! Provides automatic cache invalidation based on application events.
//!
//! ## Architecture
//! - Event subscription and handling
//! - Configurable batch processing
//! - Retry mechanism with exponential backoff
//! - Performance monitoring and metrics
//!
//! NOTE: This module is temporarily disabled due to missing event types.

// Temporarily disabled - missing types
// use crate::services::application::{
//   ApplicationEvent, ApplicationEventHandler, ChatEvent, EventError, MessageEvent, UserEvent,
// };
use crate::AppError;
use crate::services::infrastructure::cache::{DistributedLockCacheInvalidator, UnifiedCacheService};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Cache invalidation configuration
#[derive(Debug, Clone)]
pub struct CacheInvalidationConfig {
  /// Enable batch invalidation optimization
  pub enable_batch_invalidation: bool,
  /// Time window for batch invalidation (milliseconds)
  pub batch_window_ms: u64,
  /// Number of invalidation retry attempts
  pub max_retry_attempts: u32,
  /// Enable detailed logging
  pub enable_detailed_logging: bool,
}

impl Default for CacheInvalidationConfig {
  fn default() -> Self {
    Self {
      enable_batch_invalidation: true,
      batch_window_ms: 50,
      max_retry_attempts: 3,
      enable_detailed_logging: false,
    }
  }
}

/// Performance metrics for cache invalidation
#[derive(Debug, Clone, Default)]
pub struct InvalidationMetrics {
  /// Total number of invalidations
  pub total_invalidations: u64,
  /// Number of failed invalidations
  pub failed_invalidations: u64,
  /// Number of operations saved by batching
  pub operations_saved: u64,
  /// Average latency in microseconds
  pub avg_latency_us: u64,
}

/// Batch invalidation buffer
struct BatchBuffer {
  /// Chat-related invalidations: chat_id -> affected_user_ids
  chat_invalidations: HashMap<i64, HashSet<i64>>,
  /// User-related invalidations
  user_invalidations: HashSet<i64>,
  /// Last flush timestamp
  last_flush: std::time::Instant,
}

impl Default for BatchBuffer {
  fn default() -> Self {
    Self {
      chat_invalidations: HashMap::new(),
      user_invalidations: HashSet::new(),
      last_flush: std::time::Instant::now(),
    }
  }
}

/// Types of cache invalidation operations
#[derive(Debug, Clone)]
enum InvalidationType {
  ChatCreated {
    creator_id: i64,
    member_ids: Vec<i64>,
  },
  MessageSent {
    chat_id: i64,
    sender_id: i64,
  },
  MemberJoined {
    chat_id: i64,
    user_id: i64,
  },
  UserUpdated {
    user_id: i64,
  },
}

/// Production-grade cache event subscriber
pub struct CacheEventSubscriber {
  /// Cache invalidation handler
  invalidator: Arc<DistributedLockCacheInvalidator>,
  /// Configuration
  config: CacheInvalidationConfig,
  /// Batch buffer for invalidations
  batch_buffer: Arc<RwLock<BatchBuffer>>,
  /// Performance metrics
  metrics: Arc<RwLock<InvalidationMetrics>>,
}

impl CacheEventSubscriber {
  /// Create a new cache event subscriber
  pub fn new(cache_service: Arc<UnifiedCacheService>, config: CacheInvalidationConfig) -> Self {
    let invalidator = Arc::new(DistributedLockCacheInvalidator::new(cache_service));

    let subscriber = Self {
      invalidator,
      config,
      batch_buffer: Arc::new(RwLock::new(BatchBuffer::default())),
      metrics: Arc::new(RwLock::new(InvalidationMetrics::default())),
    };

    // Start background flush task if batch mode is enabled
    if subscriber.config.enable_batch_invalidation {
      subscriber.start_batch_flush_task();
    }

    subscriber
  }

  /// Start background task for batch flushing
  fn start_batch_flush_task(&self) {
    let buffer = self.batch_buffer.clone();
    let invalidator = self.invalidator.clone();
    let window_ms = self.config.batch_window_ms;
    let metrics = self.metrics.clone();

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(window_ms));

      loop {
        interval.tick().await;
        Self::flush_batch_if_needed(&buffer, &invalidator, &metrics, window_ms).await;
      }
    });
  }

  /// Flush batch buffer if time window has elapsed
  async fn flush_batch_if_needed(
    buffer: &Arc<RwLock<BatchBuffer>>,
    invalidator: &Arc<DistributedLockCacheInvalidator>,
    metrics: &Arc<RwLock<InvalidationMetrics>>,
    window_ms: u64,
  ) {
    let mut buffer_guard = buffer.write().await;
    let now = std::time::Instant::now();

    if now.duration_since(buffer_guard.last_flush).as_millis() >= window_ms as u128 {
      Self::flush_batch(&mut buffer_guard, invalidator, metrics).await;
      buffer_guard.last_flush = now;
    }
  }

  /// Flush all pending invalidations
  async fn flush_batch(
    buffer: &mut BatchBuffer,
    invalidator: &Arc<DistributedLockCacheInvalidator>,
    metrics: &Arc<RwLock<InvalidationMetrics>>,
  ) {
    let start = std::time::Instant::now();
    let mut operations = 0;
    let mut saved = 0;

    // Process chat invalidations
    for (chat_id, user_ids) in buffer.chat_invalidations.drain() {
      if !user_ids.is_empty() {
        let unique_users: Vec<i64> = user_ids.into_iter().collect();
        let _ = invalidator
          .invalidate_chat_related_with_lock(chat_id, &unique_users)
          .await;
        operations += 1;
      }
    }

    // Process user invalidations
    if !buffer.user_invalidations.is_empty() {
      let unique_users: Vec<i64> = buffer.user_invalidations.drain().collect();
      saved += unique_users.len().saturating_sub(1);

      for user_id in unique_users {
        let _ = invalidator.handle_user_updated_with_lock(user_id).await;
        operations += 1;
      }
    }

    // Update metrics
    if operations > 0 {
      let latency = start.elapsed().as_micros() as u64;
      let mut metrics_guard = metrics.write().await;

      let prev_total = metrics_guard.total_invalidations;
      metrics_guard.total_invalidations += operations;
      metrics_guard.operations_saved += saved as u64;

      // Calculate moving average for latency
      if prev_total > 0 {
        metrics_guard.avg_latency_us =
          (metrics_guard.avg_latency_us * prev_total + latency) / metrics_guard.total_invalidations;
      } else {
        metrics_guard.avg_latency_us = latency;
      }

      debug!(
        "Batch flush: {} operations, {} saved, {}μs",
        operations, saved, latency
      );
    }
  }

  /// Handle cache invalidation based on configuration
  #[instrument(skip(self))]
  async fn invalidate_with_strategy(
    &self,
    invalidation_type: InvalidationType,
  ) -> Result<(), AppError> {
    if self.config.enable_batch_invalidation {
      self.add_to_batch(invalidation_type).await
    } else {
      self.execute_invalidation(invalidation_type).await
    }
  }

  /// Add invalidation request to batch buffer
  async fn add_to_batch(&self, invalidation_type: InvalidationType) -> Result<(), AppError> {
    let mut buffer = self.batch_buffer.write().await;

    match invalidation_type {
      InvalidationType::ChatCreated {
        creator_id,
        member_ids,
      } => {
        buffer.user_invalidations.insert(creator_id);
        buffer.user_invalidations.extend(&member_ids);
      }
      InvalidationType::MessageSent { chat_id, sender_id } => {
        buffer
          .chat_invalidations
          .entry(chat_id)
          .or_default()
          .insert(sender_id);
      }
      InvalidationType::MemberJoined { chat_id, user_id } => {
        buffer.user_invalidations.insert(user_id);
        buffer
          .chat_invalidations
          .entry(chat_id)
          .or_default()
          .insert(user_id);
      }
      InvalidationType::UserUpdated { user_id } => {
        buffer.user_invalidations.insert(user_id);
      }
    }

    Ok(())
  }

  /// Execute cache invalidation with retry mechanism
  async fn execute_invalidation(
    &self,
    invalidation_type: InvalidationType,
  ) -> Result<(), AppError> {
    let mut attempts = 0;
    let max_attempts = self.config.max_retry_attempts;

    loop {
      attempts += 1;

      // Execute invalidation based on type
      let result: Result<(), AppError> = match &invalidation_type {
        InvalidationType::ChatCreated {
          creator_id,
          member_ids,
        } => {
          // For chat created, we'll use the batch invalidation
          let mut affected_users = member_ids.clone();
          affected_users.push(*creator_id);
          self
            .invalidator
            .invalidate_chat_related_with_lock(0, &affected_users) // Use dummy chat_id
            .await
        }
        InvalidationType::MessageSent { chat_id, sender_id } => {
          self
            .invalidator
            .handle_message_sent_with_lock(*chat_id, *sender_id)
            .await
        }
        InvalidationType::MemberJoined { chat_id, user_id } => {
          self
            .invalidator
            .handle_member_joined_with_lock(*chat_id, *user_id)
            .await
        }
        InvalidationType::UserUpdated { user_id } => {
          self.invalidator.handle_user_updated_with_lock(*user_id).await
        }
      };

      match result {
        Ok(_) => {
          if self.config.enable_detailed_logging {
            info!("Cache invalidation successful: {:?}", invalidation_type);
          }
          return Ok(());
        }
        Err(e) if attempts < max_attempts => {
          warn!(
            "Cache invalidation failed (attempt {}/{}): {:?}, error: {}",
            attempts, max_attempts, invalidation_type, e
          );
          // Exponential backoff
          tokio::time::sleep(tokio::time::Duration::from_millis(
            100 * 2_u64.pow(attempts - 1),
          ))
          .await;
        }
        Err(e) => {
          error!(
            "Cache invalidation failed after {} attempts: {:?}, error: {}",
            attempts, invalidation_type, e
          );

          let mut metrics = self.metrics.write().await;
          metrics.failed_invalidations += 1;

          return Err(AppError::Internal(format!(
            "Cache invalidation failed: {}",
            e
          )));
        }
      }
    }
  }

  /// Get current performance metrics
  pub async fn get_metrics(&self) -> InvalidationMetrics {
    self.metrics.read().await.clone()
  }
}

/*
Temporarily disabled due to missing ApplicationEventHandler trait

#[async_trait]
impl ApplicationEventHandler for CacheEventSubscriber {
  // #[instrument(skip(self, event))]
  // async fn handle(&self, event: &ApplicationEvent) -> Result<(), EventError> {
  //   let result = match event {
  //     // Handle chat events
  //     ApplicationEvent::Chat(chat_event) => match chat_event {
  //       ChatEvent::ChatCreated {
  //         creator_id,
  //         initial_members,
  //         ..
  //       } => {
  //         self
  //           .invalidate_with_strategy(InvalidationType::ChatCreated {
  //             creator_id: *creator_id,
  //             member_ids: initial_members.clone(),
  //           })
  //           .await
  //       }
  //       ChatEvent::MembersAdded {
  //         chat_id,
  //         new_members,
  //         ..
  //       } => {
  //         for &user_id in new_members {
  //           self
  //             .invalidate_with_strategy(InvalidationType::MemberJoined {
  //               chat_id: *chat_id,
  //               user_id,
  //             })
  //             .await
  //             .map_err(|e| EventError::HandlingFailed(e.to_string()))?;
  //         }
  //         Ok(())
  //       }
  //       ChatEvent::MembersRemoved {
  //         chat_id,
  //         removed_members,
  //         ..
  //       } => {
  //         for &user_id in removed_members {
  //           self
  //             .invalidate_with_strategy(InvalidationType::MemberJoined {
  //               chat_id: *chat_id,
  //               user_id,
  //             })
  //             .await
  //             .map_err(|e| EventError::HandlingFailed(e.to_string()))?;
  //         }
  //         Ok(())
  //       }
  //     },

  //     // Handle message events
  //     ApplicationEvent::Message(message_event) => match message_event {
  //       MessageEvent::MessageSent {
  //         chat_id, sender_id, ..
  //       } => {
  //         self
  //           .invalidate_with_strategy(InvalidationType::MessageSent {
  //             chat_id: *chat_id,
  //             sender_id: *sender_id,
  //           })
  //           .await
  //       }
  //       MessageEvent::MessageEdited {
  //         chat_id, editor_id, ..
  //       } => {
  //         self
  //           .invalidate_with_strategy(InvalidationType::MessageSent {
  //             chat_id: *chat_id,
  //             sender_id: *editor_id,
  //           })
  //           .await
  //       }
  //       MessageEvent::MessageDeleted {
  //         chat_id,
  //         deleted_by,
  //         ..
  //       } => {
  //         self
  //           .invalidate_with_strategy(InvalidationType::MessageSent {
  //             chat_id: *chat_id,
  //             sender_id: *deleted_by,
  //           })
  //           .await
  //       }
  //     },

  //     // Handle user events
  //     ApplicationEvent::User(user_event) => match user_event {
  //       UserEvent::UserProfileUpdated { user_id, .. }
  //       | UserEvent::UserStatusChanged { user_id, .. } => {
  //         self
  //           .invalidate_with_strategy(InvalidationType::UserUpdated { user_id: *user_id })
  //           .await
  //       }
  //       _ => Ok(()),
  //     },

  //     // Other events are not handled
  //     _ => Ok(()),
  //   };

  //   result.map_err(|e| EventError::HandlingFailed(e.to_string()))
  // }

  // fn name(&self) -> &'static str {
  //   "CacheEventSubscriber"
  // }
}

/// Register cache event subscriber with the application
pub fn register_cache_event_subscriber(
  app_state: &AppState,
  event_publisher: &mut crate::services::application::ApplicationEventPublisher,
) -> Result<(), AppError> {
  // Get cache service from AppState
  let cache_service = app_state
    .cache_service()
    .ok_or_else(|| AppError::Internal("Cache service not initialized".to_string()))?;

  let unified_cache = Arc::new(UnifiedCacheService::new(cache_service.clone()));

  // Create configuration
  let config = CacheInvalidationConfig {
    enable_batch_invalidation: true,
    batch_window_ms: 50,
    max_retry_attempts: 3,
    enable_detailed_logging: app_state.config.features.observability.log_level == "debug",
  };

  // Create and register subscriber
  let subscriber = Arc::new(CacheEventSubscriber::new(unified_cache, config));
  event_publisher.register_handler(subscriber);

  info!("✅ Cache event subscriber registered successfully");
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_batch_buffer_deduplication() {
    let buffer = Arc::new(RwLock::new(BatchBuffer::default()));
    let mut guard = buffer.write().await;

    // Test user deduplication
    guard.user_invalidations.insert(1);
    guard.user_invalidations.insert(1);
    guard.user_invalidations.insert(2);

    assert_eq!(guard.user_invalidations.len(), 2);
  }

  #[tokio::test]
  async fn test_invalidation_types() {
    // Test that all invalidation types can be created
    let _chat_created = InvalidationType::ChatCreated {
      creator_id: 1,
      member_ids: vec![2, 3, 4],
    };

    let _message_sent = InvalidationType::MessageSent {
      chat_id: 1,
      sender_id: 2,
    };
  }
}
*/
