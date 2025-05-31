use std::sync::Arc;

use crate::{
  error::AppError,
  services::{
    ai::{CohereClient, OpenAIClient},
    infrastructure::vector_db::{VectorDatabase, VectorSearchResult},
  },
};

/// Configuration for semantic search
#[derive(Debug, Clone)]
pub struct SearchConfig {
  pub chunk_size: usize,
  pub chunk_overlap: usize,
  pub use_reranking: bool,
  pub embedding_batch_size: usize,
}

impl Default for SearchConfig {
  fn default() -> Self {
    Self {
      chunk_size: 200,
      chunk_overlap: 50,
      use_reranking: true,
      embedding_batch_size: 50,
    }
  }
}

/// Semantic Search Enhancement Agent
/// Combines vector search with reranking for optimal results
pub struct SemanticSearchAgent {
  vector_db: Arc<dyn VectorDatabase>,
  cohere_client: Option<Arc<CohereClient>>,
  openai_client: Arc<OpenAIClient>,
  config: SearchConfig,
}

/// Message chunk for RAG
#[derive(Debug, Clone)]
pub struct MessageChunk {
  pub id: String,
  pub chat_id: i64,
  pub message_id: i64,
  pub content: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub sender_id: i64,
  pub chunk_index: usize,
  pub total_chunks: usize,
}

/// Search result with context
#[derive(Debug, Clone)]
pub struct EnhancedSearchResult {
  pub message_id: i64,
  pub chat_id: i64,
  pub content: String,
  pub snippet: String,
  pub relevance_score: f32,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub context_before: Option<String>,
  pub context_after: Option<String>,
}

impl SemanticSearchAgent {
  pub fn new(
    vector_db: Arc<dyn VectorDatabase>,
    cohere_client: Option<Arc<CohereClient>>,
    openai_client: Arc<OpenAIClient>,
    config: SearchConfig,
  ) -> Self {
    Self {
      vector_db,
      cohere_client,
      openai_client,
      config,
    }
  }

  /// Index messages with chunking for RAG
  pub async fn index_messages(
    &self,
    chat_id: i64,
    messages: Vec<(i64, i64, String, chrono::DateTime<chrono::Utc>)>, // (message_id, sender_id, content, timestamp)
  ) -> Result<(), AppError> {
    let mut all_chunks = Vec::new();

    for (message_id, sender_id, content, timestamp) in messages {
      // Chunk long messages (sliding window approach)
      let chunks = self.chunk_message(&content, self.config.chunk_size, self.config.chunk_overlap);
      let total_chunks = chunks.len();

      for (index, chunk_content) in chunks.into_iter().enumerate() {
        let chunk = MessageChunk {
          id: format!("{}-{}", message_id, index),
          chat_id,
          message_id,
          content: chunk_content,
          timestamp,
          sender_id,
          chunk_index: index,
          total_chunks,
        };
        all_chunks.push(chunk);
      }
    }

    // Generate embeddings in batches
    let batch_size = self.config.embedding_batch_size;
    for chunk_batch in all_chunks.chunks(batch_size) {
      let texts: Vec<String> = chunk_batch.iter().map(|c| c.content.clone()).collect();

      // Use Cohere if available for embeddings, otherwise use OpenAI
      let embeddings = if let Some(cohere) = &self.cohere_client {
        cohere.embed_for_search(texts).await?
      } else {
        // Fallback to OpenAI embeddings
        let mut embeddings = Vec::new();
        for text in texts {
          let embedding = self.openai_client.generate_embedding(&text).await?;
          embeddings.push(embedding);
        }
        embeddings
      };

      // Store in vector database
      let items: Vec<(
        &str,
        Vec<f32>,
        serde_json::Value,
        chrono::DateTime<chrono::Utc>,
      )> = chunk_batch
        .iter()
        .zip(embeddings.into_iter())
        .map(|(chunk, embedding)| {
          (
            chunk.id.as_str(),
            embedding,
            serde_json::json!({
                "chat_id": chunk.chat_id,
                "message_id": chunk.message_id,
                "content": chunk.content,
                "timestamp": chunk.timestamp,
                "sender_id": chunk.sender_id,
                "chunk_index": chunk.chunk_index,
                "total_chunks": chunk.total_chunks,
            }),
            chunk.timestamp,
          )
        })
        .collect();

      self.vector_db.batch_insert(&items).await?;
    }

    Ok(())
  }

  /// Enhanced semantic search with context
  pub async fn search(
    &self,
    query: &str,
    chat_id: Option<i64>,
    limit: usize,
  ) -> Result<Vec<EnhancedSearchResult>, AppError> {
    // 1. Generate query embedding
    let query_embedding = if let Some(cohere) = &self.cohere_client {
      cohere
        .embed_for_search(vec![query.to_string()])
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::ServerError("Failed to generate query embedding".to_string()))?
    } else {
      self.openai_client.generate_embedding(query).await?
    };

    // 2. Vector search
    let filter = if let Some(chat_id) = chat_id {
      Some(fechatter_core::models::vector_db::MetadataFilter::for_chat(
        fechatter_core::ChatId(chat_id),
      ))
    } else {
      None
    };

    let vector_results = self
      .vector_db
      .search(
        &query_embedding,
        if self.config.use_reranking {
          limit * 3
        } else {
          limit
        }, // Get more results for reranking
        filter,
      )
      .await?;

    // 3. Group by message and reconstruct full messages
    let mut message_map = std::collections::HashMap::new();
    for result in vector_results {
      if let Some(metadata) = result.metadata {
        let message_id = i64::from(metadata.message_id);

        message_map
          .entry(message_id)
          .or_insert_with(Vec::new)
          .push((result.score, metadata));
      } else {
        // Skip results without metadata (fallback to payload if needed)
        if let Some(message_id) = result.payload["message_id"].as_i64() {
          // Create metadata from payload as fallback
          let metadata = fechatter_core::models::vector_db::Metadata {
            chat_id: fechatter_core::ChatId(result.payload["chat_id"].as_i64().unwrap_or(0)),
            message_id: fechatter_core::MessageId(message_id),
            sender_id: fechatter_core::UserId(result.payload["sender_id"].as_i64().unwrap_or(0)),
            timestamp: chrono::DateTime::parse_from_rfc3339(
              result.payload["timestamp"]
                .as_str()
                .unwrap_or("1970-01-01T00:00:00Z"),
            )
            .unwrap_or_default()
            .with_timezone(&chrono::Utc),
            additional: result.payload.clone(),
          };

          message_map
            .entry(message_id)
            .or_insert_with(Vec::new)
            .push((result.score, metadata));
        }
      }
    }

    // 4. If reranking is enabled and Cohere is available
    if self.config.use_reranking && self.cohere_client.is_some() {
      // Prepare documents for reranking
      let mut documents = Vec::new();
      let mut doc_metadata = Vec::new();

      for (message_id, chunks) in message_map.iter() {
        // Sort chunks by index - use additional field from metadata
        let mut sorted_chunks = chunks.clone();
        sorted_chunks.sort_by_key(|(_, meta)| meta.additional["chunk_index"].as_u64().unwrap_or(0));

        // Reconstruct full message content
        let full_content = sorted_chunks
          .iter()
          .map(|(_, meta)| meta.additional["content"].as_str().unwrap_or(""))
          .collect::<Vec<_>>()
          .join(" ");

        documents.push(full_content.clone());
        doc_metadata.push((*message_id, full_content, sorted_chunks[0].1.clone()));
      }

      // 5. Rerank with Cohere
      let reranked = self
        .cohere_client
        .as_ref()
        .unwrap()
        .rerank_results(query.to_string(), documents, Some(limit))
        .await?;

      // 6. Build enhanced results from reranked
      let mut results = Vec::new();
      for (index, score) in reranked {
        let (message_id, content, metadata) = &doc_metadata[index];

        // Generate snippet around the most relevant part
        let snippet = self.generate_snippet(content, query, 100)?;

        results.push(EnhancedSearchResult {
          message_id: *message_id,
          chat_id: i64::from(metadata.chat_id),
          content: content.clone(),
          snippet,
          relevance_score: score,
          timestamp: metadata.timestamp,
          context_before: None, // TODO: Fetch from DB
          context_after: None,  // TODO: Fetch from DB
        });
      }

      Ok(results)
    } else {
      // Without reranking, build results directly
      let mut results = Vec::new();
      for (message_id, chunks) in message_map {
        let mut sorted_chunks = chunks;
        sorted_chunks.sort_by_key(|(_, meta)| meta.additional["chunk_index"].as_u64().unwrap_or(0));

        let full_content = sorted_chunks
          .iter()
          .map(|(_, meta)| meta.additional["content"].as_str().unwrap_or(""))
          .collect::<Vec<_>>()
          .join(" ");

        let avg_score =
          sorted_chunks.iter().map(|(score, _)| score).sum::<f32>() / sorted_chunks.len() as f32;

        let metadata = &sorted_chunks[0].1;
        let snippet = self.generate_snippet(&full_content, query, 100)?;

        results.push(EnhancedSearchResult {
          message_id,
          chat_id: i64::from(metadata.chat_id),
          content: full_content,
          snippet,
          relevance_score: avg_score,
          timestamp: metadata.timestamp,
          context_before: None,
          context_after: None,
        });
      }

      // Sort by relevance score
      results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
      Ok(results.into_iter().take(limit).collect())
    }
  }

  /// Generate query-based summary for search results
  pub async fn generate_search_summary(
    &self,
    query: &str,
    results: &[EnhancedSearchResult],
  ) -> Result<String, AppError> {
    let context = results
      .iter()
      .take(5)
      .map(|r| format!("- {}: {}", r.timestamp.format("%Y-%m-%d %H:%M"), r.snippet))
      .collect::<Vec<_>>()
      .join("\n");

    let prompt = format!(
      "Based on the search query '{}' and these relevant messages:\n{}\n\n\
            Provide a brief summary of what was discussed about this topic.",
      query, context
    );

    self.openai_client.generate_summary(&prompt).await
  }

  /// Chunk message with sliding window
  fn chunk_message(&self, content: &str, window_size: usize, overlap: usize) -> Vec<String> {
    let chars: Vec<char> = content.chars().collect();
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < chars.len() {
      let end = (start + window_size).min(chars.len());
      let chunk: String = chars[start..end].iter().collect();
      chunks.push(chunk);

      if end >= chars.len() {
        break;
      }

      start += window_size - overlap;
    }

    chunks
  }

  /// Generate snippet highlighting relevant parts
  fn generate_snippet(
    &self,
    content: &str,
    query: &str,
    max_length: usize,
  ) -> Result<String, AppError> {
    // Simple implementation - find query terms and extract context
    let query_terms: Vec<&str> = query.split_whitespace().collect();
    let content_lower = content.to_lowercase();

    // Find the position of the first query term
    let mut best_position = 0;
    for term in query_terms {
      if let Some(pos) = content_lower.find(&term.to_lowercase()) {
        best_position = pos;
        break;
      }
    }

    // Extract snippet around the position
    let start = best_position.saturating_sub(max_length / 2);
    let end = (best_position + max_length / 2).min(content.len());

    let snippet = &content[start..end];
    let snippet = if start > 0 {
      format!("...{}", snippet)
    } else {
      snippet.to_string()
    };
    let snippet = if end < content.len() {
      format!("{}...", snippet)
    } else {
      snippet
    };

    Ok(snippet)
  }
}
