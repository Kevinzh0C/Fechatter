//! # Infrastructure Service Contracts
//!
//! This module defines contract interfaces for infrastructure layer services, following the dependency inversion principle:
//! - **Core Layer**: Defines interface contracts (traits)
//! - **Server Layer**: Implements concrete infrastructure services
//!
//! ## Architectural Benefits
//!
//! - ✅ **Testability**: Core layer can be unit tested with mock implementations
//! - ✅ **Replaceability**: Easy to swap underlying implementations (Redis → Memory Cache)
//! - ✅ **Loose Coupling**: Core layer is independent of specific infrastructure implementations
//! - ✅ **Clear Contracts**: Interfaces clearly define service boundaries
//!
//! ## Usage Examples
//!
//! ### Implementing Interfaces in Server Layer
//!
//! ```rust
//! // fechatter_server/src/infrastructure/cache/redis.rs
//! use fechatter_core::contracts::CacheService;
//!
//! pub struct RedisCacheService {
//!     client: redis::Client,
//! }
//!
//! #[async_trait]
//! impl CacheService for RedisCacheService {
//!     async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, CoreError> {
//!         // Redis implementation
//!     }
//!     // ... other methods
//! }
//! ```
//!
//! ### Using in Application Services
//!
//! ```rust
//! // fechatter_server/src/services/application/user_app_service.rs
//! use fechatter_core::contracts::CacheService;
//!
//! pub struct UserAppService<C: CacheService> {
//!     cache: C,
//! }
//!
//! impl<C: CacheService> UserAppService<C> {
//!     pub async fn get_user_profile(&self, user_id: &str) -> Result<User, CoreError> {
//!         // Try cache first
//!         if let Some(user) = self.cache.get::<User>(&format!("user:{}", user_id)).await? {
//!             return Ok(user);
//!         }
//!         // Cache miss, fetch from database...
//!     }
//! }
//! ```

use crate::error::CoreError;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

//==============================================================================
// INFRASTRUCTURE SERVICE CONTRACTS
//==============================================================================

/// Cache Service Interface - Pluggable Component
///
/// Provides unified cache operations interface, supporting multiple cache implementations (Redis, Memory etc)
#[async_trait]
pub trait CacheService: Send + Sync {
  /// Get byte data
  async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, CoreError>;

  /// Set byte data
  async fn set_bytes(&self, key: &str, value: Vec<u8>, ttl: u64) -> Result<(), CoreError>;

  /// Delete cache item
  async fn delete(&self, key: &str) -> Result<(), CoreError>;

  /// Check if key exists
  async fn exists(&self, key: &str) -> Result<bool, CoreError>;

  /// Delete multiple keys by pattern
  async fn delete_pattern(&self, pattern: &str) -> Result<u64, CoreError>;
}

/// Event Service Interface - Pluggable Component
///
/// Provides event publish-subscribe mechanism, supporting multiple message queue implementations (NATS, RabbitMQ etc)
#[async_trait]
pub trait EventService: Send + Sync {
  /// Publish event to specified topic
  async fn publish(&self, topic: &str, event: Event) -> Result<(), CoreError>;

  /// Subscribe to event stream for specified topic
  async fn subscribe(&self, topic: &str) -> Result<EventStream, CoreError>;

  /// Acknowledge event completion
  async fn ack(&self, event_id: &str) -> Result<(), CoreError>;
}

/// Search Service Interface - Pluggable Component
///
/// Provides full-text search functionality, supporting multiple search engine implementations (Meilisearch, Elasticsearch etc)
#[async_trait]
pub trait SearchService: Send + Sync {
  /// Index document
  async fn index_document(&self, index: &str, doc: Document) -> Result<(), CoreError>;

  /// Search documents
  async fn search(&self, index: &str, query: SearchQuery) -> Result<SearchResult, CoreError>;

  /// Delete document
  async fn delete_document(&self, index: &str, id: &str) -> Result<(), CoreError>;

  /// Update document
  async fn update_document(&self, index: &str, id: &str, doc: Document) -> Result<(), CoreError>;
}

/// AI Service Interface - Pluggable Component
///
/// Provides AI functionality interface, supporting multiple AI model implementations (OpenAI, Anthropic etc)
#[async_trait]
pub trait AIService: Send + Sync {
  /// Chat completion
  async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<String, CoreError>;

  /// Generate summary
  async fn generate_summary(&self, text: &str) -> Result<String, CoreError>;

  /// Analyze sentiment
  async fn analyze_sentiment(&self, text: &str) -> Result<Sentiment, CoreError>;

  /// Suggest replies
  async fn suggest_replies(&self, context: &str) -> Result<Vec<String>, CoreError>;
}

//==============================================================================
// DATA TYPES
//==============================================================================

/// Event data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
  pub id: String,
  pub timestamp: i64,
  pub data: serde_json::Value,
}

/// Event stream type
pub type EventStream = Pin<Box<dyn Stream<Item = Event> + Send>>;

/// Search document
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
  pub id: String,
  pub fields: serde_json::Value,
}

/// Search query
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchQuery {
  pub query: String,
  pub filters: Option<serde_json::Value>,
  pub limit: u32,
  pub offset: u32,
}

/// Search result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
  pub hits: Vec<Document>,
  pub total: u64,
  pub took_ms: u64,
}

/// Chat message
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
  pub role: String,
  pub content: String,
}

/// Sentiment analysis result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sentiment {
  pub score: f32,
  pub label: String,
}

//==============================================================================
// HELPER IMPLEMENTATIONS
//==============================================================================

/// Convenience methods for CacheService
impl dyn CacheService {
  /// Get JSON data
  pub async fn get<T: for<'de> serde::Deserialize<'de>>(
    &self,
    key: &str,
  ) -> Result<Option<T>, CoreError> {
    match self.get_bytes(key).await? {
      Some(bytes) => {
        let value = serde_json::from_slice(&bytes)
          .map_err(|e| CoreError::ValidationError(format!("JSON deserialization error: {}", e)))?;
        Ok(Some(value))
      }
      None => Ok(None),
    }
  }

  /// Set JSON data
  pub async fn set<T: serde::Serialize>(
    &self,
    key: &str,
    value: &T,
    ttl: u64,
  ) -> Result<(), CoreError> {
    let bytes = serde_json::to_vec(value)
      .map_err(|e| CoreError::ValidationError(format!("JSON serialization error: {}", e)))?;
    self.set_bytes(key, bytes, ttl).await
  }
}
