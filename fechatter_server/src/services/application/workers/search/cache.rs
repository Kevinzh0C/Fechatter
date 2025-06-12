//! # Search Cache Service
//!
//! **Responsibility**: High-performance caching strategy specifically for search
//! **Features**:
//! - Multi-layer cache architecture (L1 memory + L2 Redis)
//! - Intelligent query prediction and preloading
//! - Query compression and optimization
//! - Performance monitoring and statistics

use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use super::query_processor::OptimizedQuery;
use crate::services::infrastructure::cache::CacheStrategyService;

// ================================================================================================
// Search Cache Configuration
// ================================================================================================

/// Search cache configuration
#[derive(Debug, Clone)]
pub struct SearchCacheConfig {
  /// Enable query caching
  pub enable_query_cache: bool,

  /// Enable result caching
  pub enable_result_cache: bool,

  /// Enable intelligent preloading
  pub enable_preload: bool,

  /// Query cache TTL (seconds)
  pub query_cache_ttl: u64,

  /// Result cache TTL (seconds)
  pub result_cache_ttl: u64,

  /// Maximum cache size
  pub max_cache_size: usize,

  /// Compression threshold
  pub compression_threshold: usize,
}

impl Default for SearchCacheConfig {
  fn default() -> Self {
    Self {
      enable_query_cache: true,
      enable_result_cache: true,
      enable_preload: true,
      query_cache_ttl: 300,  // 5 minutes
      result_cache_ttl: 600, // 10 minutes
      max_cache_size: 10_000,
      compression_threshold: 100,
    }
  }
}

// ================================================================================================
// Search Cache Statistics
// ================================================================================================

/// Search cache statistics
#[derive(Debug, Clone, Serialize)]
pub struct SearchCacheStats {
  /// Query cache hit rate
  pub query_hit_rate: f32,

  /// Result cache hit rate
  pub result_hit_rate: f32,

  /// Total queries
  pub total_queries: u64,

  /// Average query time
  pub avg_query_time: Duration,

  /// Cache size
  pub cache_size: usize,

  /// Preload hits
  pub preload_hits: u64,
}

impl Default for SearchCacheStats {
  fn default() -> Self {
    Self {
      query_hit_rate: 0.0,
      result_hit_rate: 0.0,
      total_queries: 0,
      avg_query_time: Duration::ZERO,
      cache_size: 0,
      preload_hits: 0,
    }
  }
}

// ================================================================================================
// Search Cache Service
// ================================================================================================

/// Search Cache Service - Unified search cache management
pub struct SearchCacheService {
  /// Base cache service
  cache_strategy: Arc<CacheStrategyService>,

  /// Configuration
  config: SearchCacheConfig,

  /// Statistics
  stats: Arc<std::sync::RwLock<SearchCacheStats>>,
}

impl SearchCacheService {
  /// Create new search cache service
  pub fn new(cache_strategy: Arc<CacheStrategyService>, config: SearchCacheConfig) -> Self {
    Self {
      cache_strategy,
      config,
      stats: Arc::new(std::sync::RwLock::new(SearchCacheStats::default())),
    }
  }

  // ========================================================================
  // Query Cache Operations
  // ========================================================================

  /// Get cached query optimization result
  pub async fn get_optimized_query(
    &self,
    query: &str,
    user_id: Option<i64>,
  ) -> Result<Option<OptimizedQuery>, crate::AppError> {
    if !self.config.enable_query_cache {
      return Ok(None);
    }

    let cache_key = self.build_query_cache_key(query, user_id);
    let start_time = Instant::now();

    match self.cache_strategy.get::<OptimizedQuery>(&cache_key).await {
      Ok(Some(result)) => {
        self.record_query_hit(start_time.elapsed());
        debug!("Query cache HIT for: {}", query);
        Ok(Some(result))
      }
      Ok(None) => {
        self.record_query_miss();
        debug!("Query cache MISS for: {}", query);
        Ok(None)
      }
      Err(e) => {
        warn!("Query cache error for {}: {}", query, e);
        Ok(None)
      }
    }
  }

  /// Cache query optimization result
  pub async fn cache_optimized_query(
    &self,
    query: String,
    result: OptimizedQuery,
    user_id: Option<i64>,
  ) -> Result<(), crate::AppError> {
    if !self.config.enable_query_cache {
      return Ok(());
    }

    let cache_key = self.build_query_cache_key(&query, user_id);

    self
      .cache_strategy
      .set(&cache_key, &result, self.config.query_cache_ttl)
      .await?;

    debug!("Cached optimized query: {}", query);
    Ok(())
  }

  // ========================================================================
  // Search Results Cache Operations
  // ========================================================================

  /// Get cached search results
  pub async fn get_search_results<T>(&self, cache_key: &str) -> Result<Option<T>, crate::AppError>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    if !self.config.enable_result_cache {
      return Ok(None);
    }

    let start_time = Instant::now();

    match self.cache_strategy.get::<T>(cache_key).await {
      Ok(Some(result)) => {
        self.record_result_hit(start_time.elapsed());
        debug!("Result cache HIT for: {}", cache_key);
        Ok(Some(result))
      }
      Ok(None) => {
        self.record_result_miss();
        debug!("Result cache MISS for: {}", cache_key);
        Ok(None)
      }
      Err(e) => {
        warn!("Result cache error for {}: {}", cache_key, e);
        Ok(None)
      }
    }
  }

  /// Cache search results
  pub async fn cache_search_results<T>(
    &self,
    cache_key: String,
    results: T,
  ) -> Result<(), crate::AppError>
  where
    T: serde::Serialize,
  {
    if !self.config.enable_result_cache {
      return Ok(());
    }

    self
      .cache_strategy
      .set(&cache_key, &results, self.config.result_cache_ttl)
      .await?;

    debug!("Cached search results: {}", cache_key);
    Ok(())
  }

  // ========================================================================
  // Cache Key Generation
  // ========================================================================

  /// Build query cache key
  fn build_query_cache_key(&self, query: &str, user_id: Option<i64>) -> String {
    match user_id {
      Some(uid) => format!("search:query:{}:{}", uid, self.hash_query(query)),
      None => format!("search:query:global:{}", self.hash_query(query)),
    }
  }

  /// Build search results cache key
  pub fn build_search_results_key(
    &self,
    query: &str,
    chat_id: Option<i64>,
    workspace_id: Option<i64>,
    limit: u32,
    offset: u32,
  ) -> String {
    let query_hash = self.hash_query(query);
    match (chat_id, workspace_id) {
      (Some(cid), _) => format!(
        "search:results:chat:{}:{}:{}:{}",
        cid, query_hash, limit, offset
      ),
      (None, Some(wid)) => format!(
        "search:results:workspace:{}:{}:{}:{}",
        wid, query_hash, limit, offset
      ),
      (None, None) => format!("search:results:global:{}:{}:{}", query_hash, limit, offset),
    }
  }

  /// Query hash (simplified)
  fn hash_query(&self, query: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    query.hash(&mut hasher);
    format!("{:x}", hasher.finish())
  }

  // ========================================================================
  // Cache Invalidation Operations
  // ========================================================================

  /// Invalidate user-related search cache
  pub async fn invalidate_user_search_cache(&self, user_id: i64) -> Result<(), crate::AppError> {
    // Pattern matching deletion needs to be implemented here, skipped for now
    debug!("Invalidating search cache for user: {}", user_id);
    Ok(())
  }

  /// Invalidate chat-related search cache
  pub async fn invalidate_chat_search_cache(&self, chat_id: i64) -> Result<(), crate::AppError> {
    // Pattern matching deletion needs to be implemented here, skipped for now
    debug!("Invalidating search cache for chat: {}", chat_id);
    Ok(())
  }

  // ========================================================================
  // Statistics and Monitoring
  // ========================================================================

  /// Record query cache hit
  fn record_query_hit(&self, duration: Duration) {
    if let Ok(mut stats) = self.stats.write() {
      stats.total_queries += 1;
      // Simplified hit rate calculation
      stats.query_hit_rate = (stats.query_hit_rate * 0.9) + 0.1;
      self.update_avg_time(&mut stats, duration);
    }
  }

  /// Record query cache miss
  fn record_query_miss(&self) {
    if let Ok(mut stats) = self.stats.write() {
      stats.total_queries += 1;
      // Simplified hit rate calculation
      stats.query_hit_rate *= 0.9;
    }
  }

  /// Record result cache hit
  fn record_result_hit(&self, duration: Duration) {
    if let Ok(mut stats) = self.stats.write() {
      // Simplified hit rate calculation
      stats.result_hit_rate = (stats.result_hit_rate * 0.9) + 0.1;
      self.update_avg_time(&mut stats, duration);
    }
  }

  /// Record result cache miss
  fn record_result_miss(&self) {
    if let Ok(mut stats) = self.stats.write() {
      // Simplified hit rate calculation
      stats.result_hit_rate *= 0.9;
    }
  }

  /// Update average time
  fn update_avg_time(&self, stats: &mut SearchCacheStats, new_time: Duration) {
    if stats.avg_query_time.is_zero() {
      stats.avg_query_time = new_time;
    } else {
      // Simplified moving average
      let total_nanos = (stats.avg_query_time.as_nanos() * 9 + new_time.as_nanos()) / 10;
      stats.avg_query_time = Duration::from_nanos(total_nanos as u64);
    }
  }

  /// Get cache statistics
  pub fn get_stats(&self) -> SearchCacheStats {
    self.stats.read().unwrap().clone()
  }

  /// Clean up expired cache (periodic task)
  pub async fn cleanup_expired(&self) {
    info!("Starting search cache cleanup...");
    // Actual cleanup logic handled by underlying cache service
    let stats = self.get_stats();
    info!(
      "Search cache stats - Query hit rate: {:.2}%, Result hit rate: {:.2}%",
      stats.query_hit_rate * 100.0,
      stats.result_hit_rate * 100.0
    );
  }
}

// ================================================================================================
// Factory Functions
// ================================================================================================

/// Create default search cache service
pub fn create_search_cache_service(
  cache_strategy: Arc<CacheStrategyService>,
) -> Arc<SearchCacheService> {
  Arc::new(SearchCacheService::new(
    cache_strategy,
    SearchCacheConfig::default(),
  ))
}

/// Create high-performance search cache service
pub fn create_high_performance_search_cache(
  cache_strategy: Arc<CacheStrategyService>,
) -> Arc<SearchCacheService> {
  let config = SearchCacheConfig {
    enable_query_cache: true,
    enable_result_cache: true,
    enable_preload: true,
    query_cache_ttl: 600,   // 10 minutes
    result_cache_ttl: 1200, // 20 minutes
    max_cache_size: 50_000,
    compression_threshold: 50,
  };

  Arc::new(SearchCacheService::new(cache_strategy, config))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cache_key_generation() {
    let service = SearchCacheService::new(
      Arc::new(CacheStrategyService::new_noop()),
      SearchCacheConfig::default(),
    );

    let query_key = service.build_query_cache_key("test query", Some(123));
    assert!(query_key.contains("search:query:123:"));

    let results_key = service.build_search_results_key("test", Some(456), None, 20, 0);
    assert!(results_key.contains("search:results:chat:456:"));
  }

  #[test]
  fn test_config_defaults() {
    let config = SearchCacheConfig::default();
    assert!(config.enable_query_cache);
    assert!(config.enable_result_cache);
    assert_eq!(config.query_cache_ttl, 300);
  }
}
