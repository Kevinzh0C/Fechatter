//! # Gateway Cache System - Pingora Native Implementation
//!
//! **High-performance caching for Gateway responses**
//!
//! Features:
//! - Memory-based cache storage
//! - TTL-based expiration
//! - Cache key generation with variants
//! - Permission-aware caching
//! - Response compression support
//! - Cache statistics and monitoring

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
  pub enabled: bool,
  pub default_ttl_secs: u64,
  pub max_size_bytes: usize,
  pub compress: bool,
  pub key_prefix: String,
  pub variants: Vec<CacheVariant>,
  pub rules: Vec<CacheRule>,
}

impl Default for CacheConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      default_ttl_secs: 300,             // 5 minutes
      max_size_bytes: 100 * 1024 * 1024, // 100MB
      compress: true,
      key_prefix: "gateway".to_string(),
      variants: vec![
        CacheVariant::UserId,
        CacheVariant::WorkspaceId,
        CacheVariant::UserPermissions,
      ],
      rules: Vec::new(),
    }
  }
}

/// Cache key variants for generating different cache keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheVariant {
  UserId,
  WorkspaceId,
  UserPermissions,
  QueryParam(String),
  Header(String),
  Method,
  Path,
}

/// Cache rule for specific paths/patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheRule {
  pub paths: Vec<String>,
  pub methods: Vec<String>,
  pub ttl_secs: u64,
  pub key_params: Vec<String>,
  pub vary_headers: Vec<String>,
  pub enabled: bool,
}

/// Cached entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
  pub content: Bytes,
  pub status_code: u16,
  pub headers: HashMap<String, String>,
  pub created_at: u64,
  pub ttl_secs: u64,
  pub compressed: bool,
  pub content_type: Option<String>,
  pub etag: Option<String>,
  pub permissions_hash: Option<String>,
}

impl CacheEntry {
  /// Check if cache entry is expired
  pub fn is_expired(&self) -> bool {
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();
    now > self.created_at + self.ttl_secs
  }

  /// Get remaining TTL in seconds
  pub fn remaining_ttl(&self) -> u64 {
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();
    if now > self.created_at + self.ttl_secs {
      0
    } else {
      self.created_at + self.ttl_secs - now
    }
  }

  /// Get cache headers for response
  pub fn get_cache_headers(&self) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("x-cache".to_string(), "HIT".to_string());
    headers.insert("x-cache-ttl".to_string(), self.remaining_ttl().to_string());
    headers.insert("x-cache-created".to_string(), self.created_at.to_string());

    if let Some(etag) = &self.etag {
      headers.insert("etag".to_string(), etag.clone());
    }

    headers
  }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
  pub entries: u64,
  pub hits: u64,
  pub misses: u64,
  pub hit_rate: f64,
  pub memory_usage: u64,
  pub total_requests: u64,
  pub evictions: u64,
}

impl CacheStats {
  /// Update hit rate calculation
  pub fn update_hit_rate(&mut self) {
    self.total_requests = self.hits + self.misses;
    self.hit_rate = if self.total_requests > 0 {
      self.hits as f64 / self.total_requests as f64 * 100.0
    } else {
      0.0
    };
  }

  /// Record cache hit
  pub fn record_hit(&mut self) {
    self.hits += 1;
    self.update_hit_rate();
  }

  /// Record cache miss
  pub fn record_miss(&mut self) {
    self.misses += 1;
    self.update_hit_rate();
  }
}

/// Memory-based cache storage
pub struct MemoryCache {
  storage: Arc<RwLock<HashMap<String, CacheEntry>>>,
  stats: Arc<RwLock<CacheStats>>,
  config: CacheConfig,
}

impl MemoryCache {
  /// Create new memory cache
  pub fn new(config: CacheConfig) -> Self {
    Self {
      storage: Arc::new(RwLock::new(HashMap::new())),
      stats: Arc::new(RwLock::new(CacheStats::default())),
      config,
    }
  }

  /// Get cached entry
  pub async fn get(&self, key: &str) -> Option<CacheEntry> {
    if !self.config.enabled {
      return None;
    }

    let storage = self.storage.read().ok()?;

    if let Some(entry) = storage.get(key) {
      if entry.is_expired() {
        drop(storage);
        // Clean up expired entry
        if let Ok(mut storage_write) = self.storage.write() {
          storage_write.remove(key);
        }
        if let Ok(mut stats) = self.stats.write() {
          stats.record_miss();
        }
        None
      } else {
        if let Ok(mut stats) = self.stats.write() {
          stats.record_hit();
        }
        debug!("💾 [CACHE] Cache HIT for key: {}", key);
        Some(entry.clone())
      }
    } else {
      if let Ok(mut stats) = self.stats.write() {
        stats.record_miss();
      }
      debug!("💔 [CACHE] Cache MISS for key: {}", key);
      None
    }
  }

  /// Store entry in cache
  pub async fn set(&self, key: String, entry: CacheEntry) -> Result<(), String> {
    if !self.config.enabled {
      return Ok(());
    }

    let mut storage = self
      .storage
      .write()
      .map_err(|_| "Failed to acquire cache write lock".to_string())?;

    // Check memory limits
    let current_size = self.calculate_memory_usage(&storage);
    let entry_size = entry.content.len();

    if current_size + entry_size > self.config.max_size_bytes {
      // Evict oldest entries
      self.evict_entries(&mut storage, entry_size);
    }

    storage.insert(key.clone(), entry);

    // Update stats
    if let Ok(mut stats) = self.stats.write() {
      stats.entries = storage.len() as u64;
      stats.memory_usage = self.calculate_memory_usage(&storage) as u64;
    }

    debug!("💾 [CACHE] Stored entry for key: {}", key);
    Ok(())
  }

  /// Clear all cache entries
  pub async fn clear(&self) -> Result<(), String> {
    let mut storage = self
      .storage
      .write()
      .map_err(|_| "Failed to acquire cache write lock".to_string())?;

    storage.clear();

    if let Ok(mut stats) = self.stats.write() {
      stats.entries = 0;
      stats.memory_usage = 0;
    }

    info!("🗑️ [CACHE] Cache cleared");
    Ok(())
  }

  /// Get cache statistics
  pub async fn get_stats(&self) -> CacheStats {
    self
      .stats
      .read()
      .map(|stats| stats.clone())
      .unwrap_or_default()
  }

  /// Calculate total memory usage
  fn calculate_memory_usage(&self, storage: &HashMap<String, CacheEntry>) -> usize {
    storage.values()
      .map(|entry| entry.content.len() + entry.headers.len() * 50) // Estimate header size
      .sum()
  }

  /// Evict entries to make space
  fn evict_entries(&self, storage: &mut HashMap<String, CacheEntry>, needed_space: usize) {
    let mut entries: Vec<(String, u64)> = storage
      .iter()
      .map(|(k, v)| (k.clone(), v.created_at))
      .collect();

    // Sort by creation time (oldest first)
    entries.sort_by_key(|(_, created_at)| *created_at);

    let mut freed_space = 0;
    let mut evicted = 0;

    for (key, _) in entries {
      if freed_space >= needed_space {
        break;
      }

      if let Some(entry) = storage.remove(&key) {
        freed_space += entry.content.len();
        evicted += 1;
      }
    }

    if let Ok(mut stats) = self.stats.write() {
      stats.evictions += evicted;
    }

    info!(
      "🗑️ [CACHE] Evicted {} entries, freed {} bytes",
      evicted, freed_space
    );
  }
}

/// Cache key generator
pub struct CacheKeyGenerator {
  config: CacheConfig,
}

impl CacheKeyGenerator {
  pub fn new(config: CacheConfig) -> Self {
    Self { config }
  }

  /// Generate cache key for request
  pub fn generate_key(
    &self,
    method: &str,
    path: &str,
    query: Option<&str>,
    user_id: Option<i64>,
    workspace_id: Option<i64>,
    permissions: Option<&str>,
    headers: &HashMap<String, String>,
  ) -> String {
    let mut key_parts = vec![
      self.config.key_prefix.clone(),
      method.to_string(),
      path.to_string(),
    ];

    // Add query parameters if present
    if let Some(q) = query {
      key_parts.push(format!("query:{}", q));
    }

    // Add variants based on configuration
    for variant in &self.config.variants {
      match variant {
        CacheVariant::UserId => {
          if let Some(uid) = user_id {
            key_parts.push(format!("user:{}", uid));
          }
        }
        CacheVariant::WorkspaceId => {
          if let Some(wid) = workspace_id {
            key_parts.push(format!("workspace:{}", wid));
          }
        }
        CacheVariant::UserPermissions => {
          if let Some(perms) = permissions {
            let mut hasher = Sha256::new();
            hasher.update(perms.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            key_parts.push(format!("perms:{}", &hash[..8])); // Use first 8 chars
          }
        }
        CacheVariant::QueryParam(param) => {
          if let Some(query_str) = query {
            for pair in query_str.split('&') {
              if let Some((key, value)) = pair.split_once('=') {
                if key == param {
                  key_parts.push(format!("{}:{}", param, value));
                  break;
                }
              }
            }
          }
        }
        CacheVariant::Header(header_name) => {
          if let Some(header_value) = headers.get(header_name) {
            key_parts.push(format!("{}:{}", header_name, header_value));
          }
        }
        CacheVariant::Method => {
          // Already added method above
        }
        CacheVariant::Path => {
          // Already added path above
        }
      }
    }

    // Create final key hash
    let key_str = key_parts.join(":");
    let mut hasher = Sha256::new();
    hasher.update(key_str.as_bytes());
    format!("{:x}", hasher.finalize())
  }

  /// Check if path should be cached
  pub fn should_cache(&self, method: &str, path: &str) -> Option<&CacheRule> {
    for rule in &self.config.rules {
      if !rule.enabled {
        continue;
      }

      let method_matches = rule.methods.is_empty() || rule.methods.contains(&method.to_uppercase());
      let path_matches = rule.paths.iter().any(|pattern| {
        if pattern.ends_with("/*") {
          let prefix = &pattern[..pattern.len() - 2];
          path.starts_with(prefix)
        } else {
          path == pattern
        }
      });

      if method_matches && path_matches {
        return Some(rule);
      }
    }

    None
  }
}

/// Gateway cache manager
pub struct GatewayCache {
  memory_cache: MemoryCache,
  key_generator: CacheKeyGenerator,
  config: CacheConfig,
}

impl GatewayCache {
  /// Create new gateway cache
  pub fn new(config: CacheConfig) -> Self {
    let key_generator = CacheKeyGenerator::new(config.clone());
    let memory_cache = MemoryCache::new(config.clone());

    Self {
      memory_cache,
      key_generator,
      config,
    }
  }

  /// Try to get cached response
  pub async fn get_cached_response(
    &self,
    method: &str,
    path: &str,
    query: Option<&str>,
    user_id: Option<i64>,
    workspace_id: Option<i64>,
    permissions: Option<&str>,
    headers: &HashMap<String, String>,
  ) -> Option<CacheEntry> {
    if !self.config.enabled {
      return None;
    }

    // Check if this request should be cached
    if self.key_generator.should_cache(method, path).is_none() {
      return None;
    }

    let cache_key = self.key_generator.generate_key(
      method,
      path,
      query,
      user_id,
      workspace_id,
      permissions,
      headers,
    );

    self.memory_cache.get(&cache_key).await
  }

  /// Store response in cache
  pub async fn store_response(
    &self,
    method: &str,
    path: &str,
    query: Option<&str>,
    user_id: Option<i64>,
    workspace_id: Option<i64>,
    permissions: Option<&str>,
    headers: &HashMap<String, String>,
    response_content: Bytes,
    response_status: u16,
    response_headers: HashMap<String, String>,
  ) -> Result<(), String> {
    if !self.config.enabled {
      return Ok(());
    }

    // Check if this request should be cached
    let cache_rule = match self.key_generator.should_cache(method, path) {
      Some(rule) => rule,
      None => return Ok(()), // Don't cache
    };

    // Only cache successful responses
    if response_status < 200 || response_status >= 300 {
      return Ok(());
    }

    let cache_key = self.key_generator.generate_key(
      method,
      path,
      query,
      user_id,
      workspace_id,
      permissions,
      headers,
    );

    let ttl_secs = if cache_rule.ttl_secs > 0 {
      cache_rule.ttl_secs
    } else {
      self.config.default_ttl_secs
    };

    let entry = CacheEntry {
      content: response_content,
      status_code: response_status,
      headers: response_headers.clone(),
      created_at: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs(),
      ttl_secs,
      compressed: self.config.compress,
      content_type: response_headers.get("content-type").cloned(),
      etag: response_headers.get("etag").cloned(),
      permissions_hash: permissions.map(|p| {
        let mut hasher = Sha256::new();
        hasher.update(p.as_bytes());
        format!("{:x}", hasher.finalize())
      }),
    };

    self.memory_cache.set(cache_key, entry).await
  }

  /// Clear all cached data
  pub async fn clear_cache(&self) -> Result<(), String> {
    self.memory_cache.clear().await
  }

  /// Get cache statistics
  pub async fn get_cache_stats(&self) -> CacheStats {
    self.memory_cache.get_stats().await
  }

  /// Check if caching is enabled
  pub fn is_enabled(&self) -> bool {
    self.config.enabled
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

use super::*;

  #[tokio::test]
  async fn test_cache_basic_operations() {
    let config = CacheConfig::default();
    let cache = GatewayCache::new(config);

    let headers = HashMap::new();
    let response_headers = HashMap::new();
    let content = Bytes::from("test content");

    // Store response
    cache
      .store_response(
        "GET",
        "/api/test",
        None,
        Some(123),
        Some(456),
        Some("WorkspaceMember"),
        &headers,
        content.clone(),
        200,
        response_headers,
      )
      .await
      .unwrap();

    // Retrieve response
    let cached = cache
      .get_cached_response(
        "GET",
        "/api/test",
        None,
        Some(123),
        Some(456),
        Some("WorkspaceMember"),
        &headers,
      )
      .await;

    assert!(cached.is_some());
    let entry = cached.unwrap();
    assert_eq!(entry.content, content);
    assert_eq!(entry.status_code, 200);
  }

  #[tokio::test]
  async fn test_cache_expiration() {
    let mut config = CacheConfig::default();
    config.default_ttl_secs = 1; // 1 second TTL

    let cache = GatewayCache::new(config);
    let headers = HashMap::new();
    let response_headers = HashMap::new();

    // Store response
    cache
      .store_response(
        "GET",
        "/api/test",
        None,
        Some(123),
        Some(456),
        Some("WorkspaceMember"),
        &headers,
        Bytes::from("test"),
        200,
        response_headers,
      )
      .await
      .unwrap();

    // Should be cached immediately
    let cached = cache
      .get_cached_response(
        "GET",
        "/api/test",
        None,
        Some(123),
        Some(456),
        Some("WorkspaceMember"),
        &headers,
      )
      .await;
    assert!(cached.is_some());

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should be expired
    let cached = cache
      .get_cached_response(
        "GET",
        "/api/test",
        None,
        Some(123),
        Some(456),
        Some("WorkspaceMember"),
        &headers,
      )
      .await;
    assert!(cached.is_none());
  }

  #[test]
  fn test_cache_key_generation() {
    let config = CacheConfig::default();
    let generator = CacheKeyGenerator::new(config);

    let headers = HashMap::new();
    let key1 = generator.generate_key(
      "GET",
      "/api/test",
      None,
      Some(123),
      Some(456),
      Some("WorkspaceMember"),
      &headers,
    );

    let key2 = generator.generate_key(
      "GET",
      "/api/test",
      None,
      Some(124), // Different user
      Some(456),
      Some("WorkspaceMember"),
      &headers,
    );

    // Keys should be different for different users
    assert_ne!(key1, key2);

    let key3 = generator.generate_key(
      "GET",
      "/api/test",
      None,
      Some(123),
      Some(456),
      Some("WorkspaceMember"),
      &headers,
    );

    // Same parameters should generate same key
    assert_eq!(key1, key3);
  }
}
