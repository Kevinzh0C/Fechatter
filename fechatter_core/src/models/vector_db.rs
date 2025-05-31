use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use utoipa::ToSchema;

use crate::{
  error::CoreError,
  models::{ChatId, MessageId, UserId},
};

/// Filter operations for metadata queries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FilterOp {
  Eq,
  Neq,
  Gt,
  Gte,
  Lt,
  Lte,
  Contains,
  StartsWith,
  EndsWith,
}

/// Single metadata filter condition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilterCondition {
  pub field: String,
  pub op: FilterOp,
  pub value: String,
}

/// Advanced metadata filter with multiple conditions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct MetadataFilter {
  pub conditions: Vec<FilterCondition>,
  pub chat_id: Option<ChatId>,
  pub sender_id: Option<UserId>,
  pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl MetadataFilter {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_condition(&mut self, field: &str, op: FilterOp, value: &str) -> &mut Self {
    self.conditions.push(FilterCondition {
      field: field.to_string(),
      op,
      value: value.to_string(),
    });
    self
  }

  pub fn with_chat_id(&mut self, chat_id: ChatId) -> &mut Self {
    self.chat_id = Some(chat_id);
    self
  }

  pub fn with_sender_id(&mut self, sender_id: UserId) -> &mut Self {
    self.sender_id = Some(sender_id);
    self
  }

  pub fn with_time_range(&mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> &mut Self {
    self.time_range = Some((start, end));
    self
  }

  // Builder pattern constructors
  pub fn for_chat(chat_id: ChatId) -> Self {
    let mut filter = Self::default();
    filter.chat_id = Some(chat_id);
    filter
  }

  pub fn for_sender(sender_id: UserId) -> Self {
    let mut filter = Self::default();
    filter.sender_id = Some(sender_id);
    filter
  }

  pub fn for_time_period(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
    let mut filter = Self::default();
    filter.time_range = Some((start, end));
    filter
  }

  pub fn equals(field: &str, value: &str) -> Self {
    let mut filter = Self::default();
    filter.add_condition(field, FilterOp::Eq, value);
    filter
  }

  pub fn contains(field: &str, value: &str) -> Self {
    let mut filter = Self::default();
    filter.add_condition(field, FilterOp::Contains, value);
    filter
  }

  // 将过滤器转换为可用于查询的格式
  pub fn to_query_params(&self) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();

    if let Some(chat_id) = self.chat_id {
      params.insert("chat_id".to_string(), chat_id.to_string());
    }

    if let Some(sender_id) = self.sender_id {
      params.insert("sender_id".to_string(), sender_id.to_string());
    }

    if let Some((start, end)) = self.time_range {
      params.insert("time_start".to_string(), start.to_rfc3339());
      params.insert("time_end".to_string(), end.to_rfc3339());
    }

    for (i, condition) in self.conditions.iter().enumerate() {
      let prefix = format!("condition_{}_", i);
      params.insert(format!("{}field", prefix), condition.field.clone());
      params.insert(
        format!("{}op", prefix),
        format!("{:?}", condition.op).to_lowercase(),
      );
      params.insert(format!("{}value", prefix), condition.value.clone());
    }

    params
  }
}

/// Common metadata fields for vector storage
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Metadata {
  pub chat_id: ChatId,
  pub message_id: MessageId,
  pub sender_id: UserId,
  pub timestamp: DateTime<Utc>,
  #[serde(flatten)]
  pub additional: Value,
}

/// Vector database search result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VectorSearchResult {
  pub id: String,
  pub score: f32,
  pub metadata: Option<Metadata>,
  pub payload: Value,
}

/// Message chunk for vector storage
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageChunk {
  pub id: String,
  pub chat_id: ChatId,
  pub message_id: MessageId,
  pub content: String,
  pub timestamp: DateTime<Utc>,
  pub sender_id: UserId,
  pub chunk_index: usize,
  pub total_chunks: usize,
}

/// Enhanced search result with context
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnhancedSearchResult {
  pub message_id: MessageId,
  pub chat_id: ChatId,
  pub content: String,
  pub snippet: String,
  pub relevance_score: f32,
  pub timestamp: DateTime<Utc>,
  pub context_before: Option<String>,
  pub context_after: Option<String>,
}

/// Vector embedding entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VectorEmbedding {
  pub id: String,
  pub embedding: Vec<f32>,
  #[serde(flatten)]
  pub metadata: Metadata,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Message embedding entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageEmbedding {
  pub id: i64,
  pub message_id: MessageId,
  pub chat_id: ChatId,
  pub chunk_index: i32,
  pub chunk_content: String,
  pub embedding: Vec<f32>,
  pub metadata: Value,
  pub created_at: DateTime<Utc>,
}

/// Vector query parameters
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VectorQuery {
  pub vector: Vec<f32>,
  pub limit: usize,
  pub filter: Option<MetadataFilter>,
  pub include_metadata: bool,
}

/// Vector database trait for different implementations
#[async_trait]
pub trait VectorDatabase: Send + Sync {
  /// Insert a vector with metadata (overwrites if ID already exists)
  async fn insert(
    &self,
    id: &str,
    vector: &[f32],
    metadata: Value,
    created_at: DateTime<Utc>,
  ) -> Result<(), CoreError>;

  /// Upsert a vector with metadata (creates or updates)
  async fn upsert(
    &self,
    id: &str,
    vector: &[f32],
    metadata: Value,
    created_at: DateTime<Utc>,
  ) -> Result<(), CoreError>;

  /// Batch insert vectors
  async fn batch_insert(
    &self,
    items: &[(&str, Vec<f32>, Value, DateTime<Utc>)],
  ) -> Result<(), CoreError>;

  /// Search for similar vectors
  async fn search(
    &self,
    query_vector: &[f32],
    limit: usize,
    filter: Option<MetadataFilter>,
  ) -> Result<Vec<VectorSearchResult>, CoreError>;

  /// Update metadata for a vector
  async fn update_metadata(&self, id: &str, metadata: Value) -> Result<(), CoreError>;

  /// Delete a vector
  async fn delete(&self, id: &str) -> Result<(), CoreError>;

  /// Delete vectors by filter
  async fn delete_by_filter(&self, filter: MetadataFilter) -> Result<usize, CoreError>;

  /// Get vector by ID
  async fn get(&self, id: &str) -> Result<Option<(Vec<f32>, Value)>, CoreError>;
}

/// Message vector repository for semantic search
#[async_trait]
pub trait MessageVectorRepository: Send + Sync {
  /// Search messages with additional context
  async fn search_messages(
    &self,
    query_vector: &[f32],
    chat_id: ChatId,
    limit: usize,
  ) -> Result<Vec<VectorSearchResult>, CoreError>;

  /// Index message for search
  async fn index_message(
    &self,
    message_id: MessageId,
    chat_id: ChatId,
    sender_id: UserId,
    content: &str,
    timestamp: DateTime<Utc>,
  ) -> Result<(), CoreError>;

  /// Remove message from index
  async fn delete_message(&self, message_id: MessageId) -> Result<(), CoreError>;

  /// Get message embeddings for a chat
  async fn get_chat_embeddings(&self, chat_id: ChatId) -> Result<Vec<MessageEmbedding>, CoreError>;
}
