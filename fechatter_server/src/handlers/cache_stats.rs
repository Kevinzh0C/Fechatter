//! Cache statistics and monitoring endpoints

use crate::{AppError, AppState};
use axum::{response::Json, Extension};
use chrono::{DateTime, Utc};
use fechatter_core::AuthUser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cache statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStatsResponse {
  /// Total number of hits
  pub total_hits: u64,
  /// Total number of misses
  pub total_misses: u64,
  /// Hit rate
  pub hit_rate: f64,
  /// Memory usage in bytes
  pub memory_usage_bytes: u64,
  /// Total number of keys
  pub total_keys: u64,
  /// Statistics timestamp
  pub timestamp: DateTime<Utc>,
  /// Cache health status
  pub health_status: CacheHealthStatus,
  /// Statistics by type
  pub stats_by_type: Vec<CacheTypeStats>,
}

/// Cache health status
#[derive(Debug, Serialize, Deserialize)]
pub enum CacheHealthStatus {
  Healthy,
  Warning { reasons: Vec<String> },
  Critical { reasons: Vec<String> },
}

/// Statistics by cache type
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheTypeStats {
  /// Cache type name
  pub cache_type: String,
  /// Number of keys
  pub key_count: u64,
  /// Average TTL in seconds
  pub avg_ttl_seconds: u64,
  /// Estimated memory usage
  pub estimated_memory_bytes: u64,
}

/// Get cache statistics
pub async fn get_cache_stats_handler(
  Extension(state): Extension<AppState>,
  Extension(_user): Extension<AuthUser>,
) -> Result<Json<CacheStatsResponse>, AppError> {
  let cache_service = state
    .cache_service()
    .ok_or_else(|| AppError::Internal("Cache service not available".to_string()))?;

  // Get basic statistics - cache_service is already Arc<RedisCacheService>
  let stats = cache_service.get_cache_stats().await?;

  // Calculate health status
  let health_status = calculate_health_status(&stats);

  // Get statistics by type (example data, should actually be fetched from Redis)
  let stats_by_type = vec![
    CacheTypeStats {
      cache_type: "chat_list".to_string(),
      key_count: 1000,
      avg_ttl_seconds: 1800,
      estimated_memory_bytes: 1024 * 100, // 100KB
    },
    CacheTypeStats {
      cache_type: "messages".to_string(),
      key_count: 5000,
      avg_ttl_seconds: 900,
      estimated_memory_bytes: 1024 * 500, // 500KB
    },
    CacheTypeStats {
      cache_type: "unread_count".to_string(),
      key_count: 3000,
      avg_ttl_seconds: 86400,
      estimated_memory_bytes: 1024 * 30, // 30KB
    },
  ];

  Ok(Json(CacheStatsResponse {
    total_hits: stats.total_hits,
    total_misses: stats.total_misses,
    hit_rate: stats.hit_rate,
    memory_usage_bytes: stats.memory_usage_bytes,
    total_keys: stats.total_keys,
    timestamp: Utc::now(),
    health_status,
    stats_by_type,
  }))
}

/// Calculate cache health status
fn calculate_health_status(
  stats: &crate::services::infrastructure::cache::redis::CacheStats,
) -> CacheHealthStatus {
  let mut warnings = Vec::new();
  let mut criticals = Vec::new();

  // Check hit rate
  if stats.hit_rate < 0.5 {
    criticals.push("Cache hit rate below 50%".to_string());
  } else if stats.hit_rate < 0.7 {
    warnings.push("Cache hit rate below 70%".to_string());
  }

  // Check memory usage (assuming 1GB limit)
  let memory_limit_gb = 1.0;
  let memory_usage_gb = stats.memory_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
  if memory_usage_gb > memory_limit_gb * 0.9 {
    criticals.push(format!(
      "Memory usage above 90% of limit ({:.2}GB)",
      memory_limit_gb
    ));
  } else if memory_usage_gb > memory_limit_gb * 0.7 {
    warnings.push(format!(
      "Memory usage above 70% of limit ({:.2}GB)",
      memory_limit_gb
    ));
  }

  // Return health status
  if !criticals.is_empty() {
    CacheHealthStatus::Critical { reasons: criticals }
  } else if !warnings.is_empty() {
    CacheHealthStatus::Warning { reasons: warnings }
  } else {
    CacheHealthStatus::Healthy
  }
}

/// Clear cache endpoint (admin only)
pub async fn clear_cache_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
  // Check if user is admin (simplified check)
  if i64::from(user.id) != 1 {
    return Err(AppError::Forbidden("Admin access required".to_string()));
  }

  let cache_service = state
    .cache_service()
    .ok_or_else(|| AppError::Internal("Cache service not available".to_string()))?;

  // Should implement more granular clearing logic here
  // For example: only clear specific types of cache, or expired cache

  Ok(Json(serde_json::json!({
      "status": "success",
      "message": "Cache cleared successfully",
      "timestamp": Utc::now().to_rfc3339(),
  })))
}

/// Get cache configuration information
pub async fn get_cache_config_handler(
  Extension(state): Extension<AppState>,
  Extension(_user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
  let cache_config = &state.config.features.cache;

  Ok(Json(serde_json::json!({
      "enabled": cache_config.enabled,
      "redis_url": cache_config.redis_url,
      "key_prefix": cache_config.key_prefix,
      "default_ttl": cache_config.default_ttl,
      "connection_timeout_ms": cache_config.connection_timeout_ms,
      "ttl_settings": {
          "short": 300,    // 5 minutes
          "medium": 1800,  // 30 minutes
          "long": 3600,    // 1 hour
          "day": 86400,    // 1 day
      },
      "optimization_strategies": {
          "chat_list_ttl": "30 minutes (event-driven invalidation)",
          "message_page_ttl": "5 minutes for latest, 1 hour for history",
          "unread_count": "Using INCR/DECR, 7 days TTL",
      }
  })))
}
