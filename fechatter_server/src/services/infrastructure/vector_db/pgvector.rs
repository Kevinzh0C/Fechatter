use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pgvector::Vector;
use serde_json::Value;
use sqlx::{PgPool, Row};

use fechatter_core::{
  error::{CoreError, VectorDbError},
  models::time_management::TimeManager,
  models::vector_db::{
    MessageEmbedding, MessageVectorRepository, MetadataFilter, VectorDatabase, VectorSearchResult,
  },
  models::{ChatId, MessageId, UserId},
};

/// Table name constants for consistent reference and compile-time validation
const MESSAGE_EMBEDDINGS_TABLE: &str = "message_embeddings";
const VECTOR_EMBEDDINGS_TABLE: &str = "vector_embeddings";

/// Newtype wrapper for embedding vectors with dimension validation and zero-copy optimization
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct EmbeddingVector(Vector);

impl EmbeddingVector {
  /// Creates embedding vector from borrowed slice with validation
  fn from_slice(data: &[f32], expected_dim: usize) -> Result<Self, VectorDbError> {
    if data.len() != expected_dim {
      return Err(VectorDbError::Validation(format!(
        "Vector dimension mismatch: expected {expected_dim}, got {}",
        data.len()
      )));
    }
    Ok(Self(Vector::from(data.to_vec())))
  }

  /// Creates owned embedding vector from Vec
  fn from_vec(data: Vec<f32>, expected_dim: usize) -> Result<Self, VectorDbError> {
    if data.len() != expected_dim {
      return Err(VectorDbError::Validation(format!(
        "Vector dimension mismatch: expected {expected_dim}, got {}",
        data.len()
      )));
    }
    Ok(Self(Vector::from(data)))
  }

  /// Access the underlying pgvector::Vector for database operations (zero-copy)
  fn as_pgvector(&self) -> &Vector {
    &self.0
  }

  /// Access the underlying slice without copying
  fn as_slice(&self) -> &[f32] {
    self.0.as_slice()
  }
}

impl From<EmbeddingVector> for Vec<f32> {
  fn from(embedding: EmbeddingVector) -> Self {
    embedding.0.into()
  }
}

/// Configuration for vector database operations
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy)]
pub struct VectorConfig {
  pub dimension: usize,
  pub batch_size: usize,
}

impl Default for VectorConfig {
  fn default() -> Self {
    Self {
      dimension: 1536, // Common OpenAI embedding dimension
      batch_size: 100,
    }
  }
}

/// PostgreSQL vector database implementation using pgvector extension
///
/// Provides high-performance vector storage and similarity search with:
/// - Batch processing optimization (configurable batch size)
/// - Vector dimension validation at compile time where possible
/// - Reference-based operations to avoid unnecessary copies
/// - Unified time management for data consistency
pub struct PgVectorDatabase {
  pool: PgPool,
  config: VectorConfig,
}

impl PgVectorDatabase {
  /// Creates a new vector database instance
  ///
  /// # Arguments
  /// * `pool` - PostgreSQL connection pool
  /// * `config` - Vector configuration including dimension
  ///
  /// # Examples
  /// ```
  /// let config = VectorConfig { dimension: 1536, batch_size: 100 };
  /// let db = PgVectorDatabase::new(pool, config);
  /// ```
  pub fn new(pool: PgPool, config: VectorConfig) -> Self {
    Self { pool, config }
  }

  /// Creates instance with default configuration for common use cases
  pub fn with_dimension(pool: PgPool, dimension: usize) -> Self {
    let config = VectorConfig {
      dimension,
      ..VectorConfig::default()
    };
    Self::new(pool, config)
  }

  /// Validates and converts raw vector data to typed embedding
  fn validate_vector(&self, data: &[f32]) -> Result<EmbeddingVector, VectorDbError> {
    EmbeddingVector::from_slice(data, self.config.dimension)
  }

  /// Converts database errors to VectorDbError
  fn convert_error(error: sqlx::Error) -> VectorDbError {
    VectorDbError::Permanent(error.to_string())
  }
}

/// Search parameter variants for type-safe query building
enum SearchParams<'a> {
  WithFilter { vector: &'a Vector, filter: Value },
  NoFilter { vector: &'a Vector },
}

#[async_trait]
impl VectorDatabase for PgVectorDatabase {
  /// Inserts a single vector record with automatic timestamping
  async fn insert(
    &self,
    record_id: &str,
    embedding_vector: &[f32],
    record_metadata: Value,
    _provided_timestamp: DateTime<Utc>, // Ignored in favor of unified time management
  ) -> Result<(), CoreError> {
    let creation_time = TimeManager::now();
    let validated_vector = self.validate_vector(embedding_vector)?;

    sqlx::query(
      r#"
      INSERT INTO message_embeddings (id, embedding, metadata, created_at, updated_at) 
      VALUES ($1, $2, $3, $4, $5)
      "#,
    )
    .bind(record_id)
    .bind(validated_vector.as_pgvector())
    .bind(record_metadata)
    .bind(creation_time)
    .bind(creation_time)
    .execute(&self.pool)
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(())
  }

  /// Upserts a vector record (insert or update on conflict)
  async fn upsert(
    &self,
    record_id: &str,
    embedding_vector: &[f32],
    record_metadata: Value,
    _provided_timestamp: DateTime<Utc>,
  ) -> Result<(), CoreError> {
    let operation_time = TimeManager::now();
    let validated_vector = self
      .validate_vector(embedding_vector)
      .map_err(CoreError::VectorDbError)?;

    sqlx::query(
      r#"
      INSERT INTO message_embeddings (id, embedding, metadata, created_at, updated_at)
      VALUES ($1, $2, $3, $4, $5)
      ON CONFLICT (id)
      DO UPDATE SET
        embedding = EXCLUDED.embedding,
        metadata = EXCLUDED.metadata,
        updated_at = $6
      "#,
    )
    .bind(record_id)
    .bind(validated_vector.as_pgvector())
    .bind(&record_metadata)
    .bind(operation_time)
    .bind(operation_time)
    .bind(operation_time)
    .execute(&self.pool)
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(())
  }

  /// Batch inserts vector records with optimized chunking
  async fn batch_insert(
    &self,
    records: &[(&str, Vec<f32>, Value, DateTime<Utc>)],
  ) -> Result<(), CoreError> {
    if records.is_empty() {
      return Ok(());
    }

    // Process in configured batch sizes
    for batch in records.chunks(self.config.batch_size) {
      self.process_batch(batch).await?;
    }
    Ok(())
  }

  /// Performs vector similarity search with optional metadata filtering
  async fn search(
    &self,
    query_vector: &[f32],
    result_limit: usize,
    metadata_filter: Option<MetadataFilter>,
  ) -> Result<Vec<VectorSearchResult>, CoreError> {
    let validated_query_vector = self
      .validate_vector(query_vector)
      .map_err(CoreError::VectorDbError)?;
    let has_filter = metadata_filter.is_some();

    let rows = if has_filter {
      let filter_params = metadata_filter.unwrap().to_query_params();
      let filter_map: serde_json::Map<String, Value> = filter_params
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();
      let filter = Value::Object(filter_map);
      sqlx::query(
        r#"
        SELECT id, 1 - (embedding <=> $1) as similarity, metadata
        FROM message_embeddings 
        WHERE metadata @> $2
        ORDER BY embedding <=> $1 
        LIMIT $3
        "#,
      )
      .bind(validated_query_vector.as_pgvector())
      .bind(&filter)
      .bind(result_limit as i64)
      .fetch_all(&self.pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?
    } else {
      sqlx::query(
        r#"
        SELECT id, 1 - (embedding <=> $1) as similarity, metadata
        FROM message_embeddings 
        ORDER BY embedding <=> $1 
        LIMIT $2
        "#,
      )
      .bind(validated_query_vector.as_pgvector())
      .bind(result_limit as i64)
      .fetch_all(&self.pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?
    };

    Ok(
      rows
        .into_iter()
        .map(|row| VectorSearchResult {
          id: row.get("id"),
          score: row.get("similarity"),
          metadata: None,
          payload: row.get("metadata"),
        })
        .collect(),
    )
  }

  /// Updates metadata for an existing vector record
  async fn update_metadata(&self, record_id: &str, new_metadata: Value) -> Result<(), CoreError> {
    let update_time = TimeManager::now();

    let result =
      sqlx::query("UPDATE message_embeddings SET metadata = $1, updated_at = $2 WHERE id = $3")
        .bind(&new_metadata)
        .bind(update_time)
        .bind(record_id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Internal(e.to_string()))?;

    match result.rows_affected() {
      0 => Err(CoreError::NotFound(format!(
        "Record not found: {record_id}"
      ))),
      _ => Ok(()),
    }
  }

  /// Deletes a vector record by ID
  async fn delete(&self, record_id: &str) -> Result<(), CoreError> {
    let result = sqlx::query("DELETE FROM message_embeddings WHERE id = $1")
      .bind(record_id)
      .execute(&self.pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    match result.rows_affected() {
      0 => Err(CoreError::NotFound(format!(
        "Record not found: {record_id}"
      ))),
      _ => Ok(()),
    }
  }

  /// Deletes records matching metadata filter (with safety check)
  async fn delete_by_filter(&self, filter_conditions: MetadataFilter) -> Result<usize, CoreError> {
    // Check if filter is empty - equivalent to is_empty() method
    if filter_conditions.conditions.is_empty()
      && filter_conditions.chat_id.is_none()
      && filter_conditions.sender_id.is_none()
      && filter_conditions.time_range.is_none()
    {
      return Err(CoreError::Validation(
        "Filter conditions cannot be empty for safety".to_string(),
      ));
    }

    let filter_params = filter_conditions.to_query_params();
    let filter_map: serde_json::Map<String, Value> = filter_params
      .into_iter()
      .map(|(k, v)| (k, Value::String(v)))
      .collect();
    let filter_json = Value::Object(filter_map);

    let result = sqlx::query("DELETE FROM message_embeddings WHERE metadata @> $1")
      .bind(&filter_json)
      .execute(&self.pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(result.rows_affected() as usize)
  }

  /// Retrieves vector and metadata by record ID
  async fn get(&self, record_id: &str) -> Result<Option<(Vec<f32>, Value)>, CoreError> {
    let row = sqlx::query("SELECT embedding, metadata FROM message_embeddings WHERE id = $1")
      .bind(record_id)
      .fetch_optional(&self.pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(row.map(|r| {
      let vector: Vector = r.get("embedding");
      (vector.into(), r.get("metadata"))
    }))
  }
}

impl PgVectorDatabase {
  /// Processes a single batch of records for batch insert using regular sqlx::query for type compatibility
  async fn process_batch(
    &self,
    batch: &[(&str, Vec<f32>, Value, DateTime<Utc>)],
  ) -> Result<(), CoreError> {
    let batch_time = TimeManager::now();

    // Process records individually with proper error handling
    for (id, vector, metadata, _) in batch {
      let validated_vector = self
        .validate_vector(&vector)
        .map_err(CoreError::VectorDbError)?;

      sqlx::query(
        r#"
        INSERT INTO message_embeddings (id, embedding, metadata, created_at, updated_at) 
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) 
        DO UPDATE SET 
          embedding = EXCLUDED.embedding, 
          metadata = EXCLUDED.metadata, 
          updated_at = EXCLUDED.updated_at
        "#,
      )
      .bind(id)
      .bind(validated_vector.as_pgvector())
      .bind(metadata)
      .bind(batch_time)
      .bind(batch_time)
      .execute(&self.pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;
    }

    Ok(())
  }
}

/// Message vector storage implementation
///
/// Optimized for chat message vector storage and retrieval
#[async_trait]
impl MessageVectorRepository for PgVectorDatabase {
  /// Search similar messages
  ///
  /// # Arguments
  /// - `query_vector`: Query vector
  /// - `target_chat_id`: Optional chat room ID filter
  /// - `result_limit`: Maximum number of results to return
  ///
  /// # Performance characteristics
  /// - Uses vector similarity ordering
  /// - Supports filtering by chat room to reduce search scope
  async fn search_messages(
    &self,
    query_vector: &[f32],
    target_chat_id: ChatId,
    result_limit: usize,
  ) -> Result<Vec<VectorSearchResult>, CoreError> {
    let validated_query_vector = self
      .validate_vector(query_vector)
      .map_err(CoreError::VectorDbError)?;

    let rows = sqlx::query(
      r#"
      SELECT
        message_id,
        chat_id,
        chunk_index,
        chunk_content,
        1 - (embedding <=> $1) as similarity,
        metadata,
        created_at
      FROM message_embeddings
      WHERE chat_id = $2
      ORDER BY embedding <=> $1 
      LIMIT $3
      "#,
    )
    .bind(validated_query_vector.as_pgvector())
    .bind(target_chat_id.0)
    .bind(result_limit as i64)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(
      rows
        .into_iter()
        .map(|row| VectorSearchResult {
          id: format!(
            "{}-{}",
            row.get::<i64, _>("message_id"),
            row.get::<i32, _>("chunk_index")
          ),
          score: row.get("similarity"),
          metadata: None,
          payload: serde_json::json!({
            "message_id": row.get::<i64, _>("message_id"),
            "chat_id": row.get::<i64, _>("chat_id"),
            "chunk_index": row.get::<i32, _>("chunk_index"),
            "chunk_content": row.get::<String, _>("chunk_content"),
            "created_at": row.get::<DateTime<Utc>, _>("created_at"),
          }),
        })
        .collect(),
    )
  }

  /// Delete all vector embeddings for a specific message
  ///
  /// # Purpose
  /// Clean up vector indices when a message is deleted
  async fn delete_message(&self, target_message_id: MessageId) -> Result<(), CoreError> {
    sqlx::query("DELETE FROM message_embeddings WHERE message_id = $1")
      .bind(target_message_id.0)
      .execute(&self.pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))
      .map(|_| ())
  }

  /// Create vector index for a message
  ///
  /// # Note
  /// This is a placeholder implementation. In production:
  /// 1. Use dedicated embedding service to generate vectors
  /// 2. Support long text chunking
  /// 3. Process asynchronously to avoid blocking message sending
  async fn index_message(
    &self,
    target_message_id: MessageId,
    source_chat_id: ChatId,
    author_user_id: UserId,
    message_content: &str,
    _provided_timestamp: DateTime<Utc>,
  ) -> Result<(), CoreError> {
    let indexing_time = TimeManager::now();

    // TODO: 生产环境中应该调用embedding服务
    // 目前使用零向量作为占位符
    let placeholder_vector = vec![0.0f32; self.config.dimension];
    let validated_placeholder =
      EmbeddingVector::from_vec(placeholder_vector, self.config.dimension)
        .map_err(CoreError::VectorDbError)?;

    let message_metadata = serde_json::json!({
        "message_id": target_message_id.0,
        "chat_id": source_chat_id.0,
        "sender_id": author_user_id.0,
        "timestamp": indexing_time,
        "content": message_content,
    });

    sqlx::query(
      r#"
      INSERT INTO message_embeddings
      (message_id, chat_id, chunk_index, chunk_content, embedding, metadata, created_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
      ON CONFLICT (message_id, chunk_index)
      DO UPDATE SET
        chunk_content = EXCLUDED.chunk_content,
        embedding = EXCLUDED.embedding,
        metadata = EXCLUDED.metadata
      "#,
    )
    .bind(target_message_id.0)
    .bind(source_chat_id.0)
    .bind(0) // Default chunk index
    .bind(message_content)
    .bind(validated_placeholder.as_pgvector())
    .bind(&message_metadata)
    .bind(indexing_time)
    .execute(&self.pool)
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))
    .map(|_| ())
  }

  /// Retrieves all message embeddings for a specified chat room
  ///
  /// # Use cases
  /// - Chat room data export
  /// - Batch vector analysis
  /// - Debugging and monitoring
  async fn get_chat_embeddings(
    &self,
    target_chat_id: ChatId,
  ) -> Result<Vec<MessageEmbedding>, CoreError> {
    let rows = sqlx::query(
      r#"
      SELECT
        message_id,
        chat_id,
        chunk_index,
        chunk_content,
        embedding,
        metadata,
        created_at
      FROM message_embeddings
      WHERE chat_id = $1
      ORDER BY created_at DESC
      "#,
    )
    .bind(target_chat_id.0)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(
      rows
        .into_iter()
        .map(|row| {
          let stored_vector: Vector = row.get("embedding");
          let vector_data: Vec<f32> = stored_vector.into();
          MessageEmbedding {
            id: row.get::<i64, _>("message_id"),
            message_id: MessageId(row.get("message_id")),
            chat_id: ChatId(row.get("chat_id")),
            chunk_index: row.get("chunk_index"),
            chunk_content: row.get("chunk_content"),
            embedding: vector_data,
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
          }
        })
        .collect(),
    )
  }
}

/// Message search parameter variants for type-safe query building
enum MessageSearchParams {
  WithChatFilter { vector: Vector, chat_id: ChatId },
  NoFilter { vector: Vector },
}

impl PgVectorDatabase {
  /// Builds message search parameters in a type-safe manner
  fn build_message_search_params(
    &self,
    vector: &EmbeddingVector,
    chat_filter: Option<ChatId>,
    limit: usize,
  ) -> (String, MessageSearchParams) {
    // Single Option destructuring to avoid move-after-use errors
    let has_chat_filter = chat_filter.is_some();

    let base_query = format!(
      r#"
      SELECT
        message_id,
        chat_id,
        chunk_index,
        chunk_content,
        1 - (embedding <=> $1) as similarity,
        metadata,
        created_at
      FROM message_embeddings
      WHERE 1=1{chat_filter}
      ORDER BY embedding <=> $1 
      LIMIT {limit}
      "#,
      chat_filter = if has_chat_filter {
        " AND chat_id = $2"
      } else {
        ""
      }
    );

    // Convert to pgvector for database binding
    let pg_vector = vector.as_pgvector().clone();

    let params = match chat_filter {
      Some(chat_id) => MessageSearchParams::WithChatFilter {
        vector: pg_vector,
        chat_id,
      },
      None => MessageSearchParams::NoFilter { vector: pg_vector },
    };

    (base_query, params)
  }

  /// Converts database row to MessageEmbedding with type safety
  fn row_to_message_embedding(&self, row: sqlx::postgres::PgRow) -> MessageEmbedding {
    let stored_vector: Vector = row.get("embedding");
    let vector_data: Vec<f32> = stored_vector.into();

    MessageEmbedding {
      id: row.get::<i64, _>("message_id"),
      message_id: MessageId(row.get("message_id")),
      chat_id: ChatId(row.get("chat_id")),
      chunk_index: row.get("chunk_index"),
      chunk_content: row.get("chunk_content"),
      embedding: vector_data,
      metadata: row.get("metadata"),
      created_at: row.get("created_at"),
    }
  }
}
