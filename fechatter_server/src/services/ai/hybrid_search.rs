use std::collections::HashMap;
use std::sync::Arc;

use crate::{
  error::AppError,
  services::{
    ai::{CohereClient, OpenAIClient},
    infrastructure::{
      search::meilisearch::MeilisearchClient,
      vector_db::{PgVectorDatabase, VectorDatabase},
    },
  },
};

/// Configuration for hybrid search
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
  pub semantic_weight: f32, // Weight for semantic search (0.0 - 1.0)
  pub keyword_weight: f32,  // Weight for keyword search (0.0 - 1.0)
  pub use_reranking: bool,  // Whether to use Cohere reranking
  pub chunk_size: usize,    // Message chunk size
  pub chunk_overlap: usize, // Chunk overlap
}

impl Default for HybridSearchConfig {
  fn default() -> Self {
    Self {
      semantic_weight: 0.7,
      keyword_weight: 0.3,
      use_reranking: true,
      chunk_size: 200,
      chunk_overlap: 50,
    }
  }
}

/// Hybrid search result combining semantic and keyword search
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
  pub message_id: i64,
  pub chat_id: i64,
  pub content: String,
  pub semantic_score: f32,
  pub keyword_score: f32,
  pub combined_score: f32,
  pub snippet: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Hybrid Search Service combining pgvector and Meilisearch
pub struct HybridSearchService {
  pg_vector: Arc<PgVectorDatabase>,
  meilisearch: Arc<MeilisearchClient>,
  cohere_client: Option<Arc<CohereClient>>,
  openai_client: Arc<OpenAIClient>,
  config: HybridSearchConfig,
}

impl HybridSearchService {
  pub fn new(
    pg_vector: Arc<PgVectorDatabase>,
    meilisearch: Arc<MeilisearchClient>,
    cohere_client: Option<Arc<CohereClient>>,
    openai_client: Arc<OpenAIClient>,
    config: HybridSearchConfig,
  ) -> Self {
    Self {
      pg_vector,
      meilisearch,
      cohere_client,
      openai_client,
      config,
    }
  }

  /// Initialize both search systems
  pub async fn initialize(&self) -> Result<(), AppError> {
    // Database setup is now handled by migrations (0010_vector_db_setup.sql)
    // No need to call setup methods

    // Setup Meilisearch
    self
      .meilisearch
      .create_index("messages", "message_id")
      .await?;
    self
      .meilisearch
      .configure_index("messages", vec!["content", "sender_id", "chat_id"])
      .await?;

    Ok(())
  }

  /// Index a message in both systems
  pub async fn index_message(
    &self,
    message_id: i64,
    chat_id: i64,
    sender_id: i64,
    content: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
  ) -> Result<(), AppError> {
    // 1. Index in Meilisearch for full-text search
    self
      .meilisearch
      .index_message(message_id, chat_id, sender_id, content, timestamp)
      .await?;

    // 2. Chunk and index in pgvector for semantic search
    let chunks = self.chunk_message(content);

    for (index, chunk) in chunks.iter().enumerate() {
      // Generate embedding
      let embedding = if let Some(cohere) = &self.cohere_client {
        cohere
          .embed_for_search(vec![chunk.clone()])
          .await?
          .into_iter()
          .next()
          .ok_or_else(|| AppError::ServerError("Failed to generate embedding".to_string()))?
      } else {
        self.openai_client.generate_embedding(chunk).await?
      };

      // Store in pgvector
      let chunk_id = format!("{}-{}", message_id, index);
      let metadata = serde_json::json!({
          "message_id": message_id,
          "chat_id": chat_id,
          "sender_id": sender_id,
          "chunk_index": index,
          "chunk_content": chunk,
          "timestamp": timestamp,
          "full_content": content,
      });

      self
        .pg_vector
        .insert(&chunk_id, &embedding, metadata, timestamp)
        .await?;
    }

    Ok(())
  }

  /// Perform hybrid search combining semantic and keyword search
  pub async fn search(
    &self,
    query: &str,
    chat_id: Option<i64>,
    limit: usize,
  ) -> Result<Vec<HybridSearchResult>, AppError> {
    // 1. Perform parallel searches
    let (semantic_results, keyword_results) = tokio::join!(
      self.semantic_search(query, chat_id, limit * 2),
      self.keyword_search(query, chat_id, limit * 2)
    );

    let semantic_results = semantic_results?;
    let keyword_results = keyword_results?;

    // 2. Merge results
    let mut combined_results = self.merge_results(semantic_results, keyword_results);

    // 3. Optional reranking with Cohere
    if self.config.use_reranking && self.cohere_client.is_some() {
      combined_results = self.rerank_results(query, combined_results, limit).await?;
    }

    // 4. Take top results
    combined_results.truncate(limit);

    Ok(combined_results)
  }

  /// Perform semantic search using pgvector
  async fn semantic_search(
    &self,
    query: &str,
    chat_id: Option<i64>,
    limit: usize,
  ) -> Result<Vec<(i64, String, f32, chrono::DateTime<chrono::Utc>)>, AppError> {
    // Generate query embedding
    let query_embedding = if let Some(cohere) = &self.cohere_client {
      cohere
        .embed_for_search(vec![query.to_string()])
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::ServerError("Failed to generate embedding".to_string()))?
    } else {
      self.openai_client.generate_embedding(query).await?
    };

    // Build filter
    let filter = chat_id.map(|id| {
      fechatter_core::models::vector_db::MetadataFilter::for_chat(fechatter_core::ChatId(id))
    });

    // Search
    let results = self
      .pg_vector
      .search(&query_embedding, limit, filter)
      .await
      .map_err(|e| AppError::Internal(e.to_string()))?;

    // Group by message and reconstruct
    let mut message_map: HashMap<i64, (String, f32, chrono::DateTime<chrono::Utc>)> =
      HashMap::new();

    for result in results {
      if let Some(metadata) = result.metadata {
        let message_id = i64::from(metadata.message_id);
        let content = metadata.additional["full_content"]
          .as_str()
          .unwrap_or("")
          .to_string();
        let timestamp = metadata.timestamp;

        message_map
          .entry(message_id)
          .and_modify(|(_, score, _)| *score = score.max(result.score))
          .or_insert((content, result.score, timestamp));
      } else {
        // Fallback to payload if metadata is None
        if let Some(message_id) = result.payload["message_id"].as_i64() {
          let content = result.payload["full_content"]
            .as_str()
            .unwrap_or("")
            .to_string();
          let timestamp = chrono::DateTime::parse_from_rfc3339(
            result.payload["timestamp"]
              .as_str()
              .unwrap_or("1970-01-01T00:00:00Z"),
          )
          .unwrap_or_default()
          .with_timezone(&chrono::Utc);

          message_map
            .entry(message_id)
            .and_modify(|(_, score, _)| *score = score.max(result.score))
            .or_insert((content, result.score, timestamp));
        }
      }
    }

    Ok(
      message_map
        .into_iter()
        .map(|(id, (content, score, timestamp))| (id, content, score, timestamp))
        .collect(),
    )
  }

  /// Perform keyword search using Meilisearch
  async fn keyword_search(
    &self,
    query: &str,
    chat_id: Option<i64>,
    limit: usize,
  ) -> Result<Vec<(i64, String, f32)>, AppError> {
    self
      .meilisearch
      .search_messages(query, chat_id, limit as u32)
      .await
      .map_err(|e| AppError::SearchError(e.to_string()))
  }

  /// Merge semantic and keyword search results
  fn merge_results(
    &self,
    semantic_results: Vec<(i64, String, f32, chrono::DateTime<chrono::Utc>)>,
    keyword_results: Vec<(i64, String, f32)>,
  ) -> Vec<HybridSearchResult> {
    let mut combined_map: HashMap<i64, HybridSearchResult> = HashMap::new();

    // Add semantic results
    for (message_id, content, score, timestamp) in semantic_results {
      combined_map.insert(
        message_id,
        HybridSearchResult {
          message_id,
          chat_id: 0, // Will be filled later
          content: content.clone(),
          semantic_score: score,
          keyword_score: 0.0,
          combined_score: score * self.config.semantic_weight,
          snippet: self.generate_snippet(&content, ""),
          timestamp,
        },
      );
    }

    // Add/update with keyword results
    for (message_id, content, score) in keyword_results {
      combined_map
        .entry(message_id)
        .and_modify(|result| {
          result.keyword_score = score;
          result.combined_score = result.semantic_score * self.config.semantic_weight
            + score * self.config.keyword_weight;
        })
        .or_insert_with(|| HybridSearchResult {
          message_id,
          chat_id: 0,
          content: content.clone(),
          semantic_score: 0.0,
          keyword_score: score,
          combined_score: score * self.config.keyword_weight,
          snippet: self.generate_snippet(&content, ""),
          timestamp: chrono::Utc::now(), // Default timestamp
        });
    }

    // Sort by combined score
    let mut results: Vec<HybridSearchResult> = combined_map.into_values().collect();
    results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());

    results
  }

  /// Rerank results using Cohere
  async fn rerank_results(
    &self,
    query: &str,
    results: Vec<HybridSearchResult>,
    limit: usize,
  ) -> Result<Vec<HybridSearchResult>, AppError> {
    if results.is_empty() {
      return Ok(results);
    }

    let documents: Vec<String> = results.iter().map(|r| r.content.clone()).collect();

    let reranked = self
      .cohere_client
      .as_ref()
      .unwrap()
      .rerank_results(query.to_string(), documents, Some(limit))
      .await?;

    let mut reranked_results = Vec::new();
    for (index, score) in reranked {
      let mut result = results[index].clone();
      result.combined_score = score;
      reranked_results.push(result);
    }

    Ok(reranked_results)
  }

  /// Chunk message for semantic search
  fn chunk_message(&self, content: &str) -> Vec<String> {
    let chars: Vec<char> = content.chars().collect();
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < chars.len() {
      let end = (start + self.config.chunk_size).min(chars.len());
      let chunk: String = chars[start..end].iter().collect();
      chunks.push(chunk);

      if end >= chars.len() {
        break;
      }

      start += self.config.chunk_size - self.config.chunk_overlap;
    }

    chunks
  }

  /// Generate snippet for display
  fn generate_snippet(&self, content: &str, _query: &str) -> String {
    if content.len() <= 150 {
      content.to_string()
    } else {
      format!("{}...", &content[..150])
    }
  }
}
