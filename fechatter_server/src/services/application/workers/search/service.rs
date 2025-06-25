//! # Search Application Service
//!

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{Duration, timeout};
use tracing::{debug, info, instrument};
use utoipa::ToSchema;

use fechatter_core::{
  contracts::{Document, SearchQuery, SearchResult, SearchService as CoreSearchService},
  models::{ChatId, MessageId, UserId, WorkspaceId},
};

use super::cache::SearchCacheService;
use crate::config::AppConfig;
use crate::error::AppError;

#[async_trait]
pub trait SearchApplicationServiceTrait: Send + Sync {
  async fn search_messages_in_chat(
    &self,
    chat_id: ChatId,
    query: &str,
    user_id: UserId,
    limit: u32,
    offset: u32,
  ) -> Result<MessageSearchResults, AppError>;

  async fn global_search_messages(
    &self,
    query: &str,
    user_id: UserId,
    workspace_id: WorkspaceId,
    limit: u32,
    offset: u32,
  ) -> Result<MessageSearchResults, AppError>;

  async fn index_messages_batch(&self, messages: &[SearchableMessage]) -> Result<(), AppError>;

  async fn remove_messages_from_index_batch(
    &self,
    message_ids: &[MessageId],
  ) -> Result<(), AppError>;

  async fn update_messages_in_index_batch(
    &self,
    messages: &[SearchableMessage],
  ) -> Result<(), AppError>;

  async fn reindex_chat_messages(&self, chat_id: ChatId) -> Result<u64, AppError>;

  async fn get_search_suggestions(
    &self,
    partial_query: &str,
    limit: u32,
  ) -> Result<Vec<String>, AppError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchableMessage {
  pub id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub sender_name: String,
  pub content: String,
  pub files: Option<Vec<String>>,
  #[serde(with = "timestamp_serde")]
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub workspace_id: i64,
  pub chat_name: String,
  pub chat_type: String,
}

mod timestamp_serde {
  use chrono::{DateTime, Utc};
  use serde::{Deserialize, Deserializer, Serialize, Serializer};

  pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    date.timestamp().serialize(serializer)
  }

  pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
  where
    D: Deserializer<'de>,
  {
    let timestamp = i64::deserialize(deserializer)?;
    Ok(DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now()))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageSearchResults {
  pub hits: Vec<SearchableMessage>,
  pub total: u64,
  pub took_ms: u64,
  pub query: String,
  pub page: SearchPage,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchPage {
  pub offset: u32,
  pub limit: u32,
  pub has_more: bool,
}

#[derive(Debug, Clone)]
pub struct SearchConfig {
  pub enabled: bool,
  pub index_name: String,
  pub max_results: u32,
  pub default_limit: u32,
  pub search_timeout_ms: u64,
  pub batch_size: usize,
}

impl Default for SearchConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      index_name: "messages".to_string(),
      max_results: 1000,
      default_limit: 20,
      search_timeout_ms: 3000,
      batch_size: 1000,
    }
  }
}

pub struct SearchApplicationService {
  search_service: Arc<dyn CoreSearchService>,
  search_cache: Arc<SearchCacheService>,
  config: SearchConfig,
}

impl SearchApplicationService {
  pub fn new(
    search_service: Arc<dyn CoreSearchService>,
    search_cache: Arc<SearchCacheService>,
    config: SearchConfig,
  ) -> Self {
    Self {
      search_service,
      search_cache,
      config,
    }
  }

  pub fn from_app_config(
    search_service: Arc<dyn CoreSearchService>,
    search_cache: Arc<SearchCacheService>,
    app_config: &AppConfig,
  ) -> Self {
    let search_config = SearchConfig {
      enabled: app_config.features.search.enabled,
      search_timeout_ms: 3000,
      batch_size: 1000,
      ..Default::default()
    };

    Self::new(search_service, search_cache, search_config)
  }

  fn message_to_document(&self, message: &SearchableMessage) -> Result<Document, AppError> {
    let fields = serde_json::to_value(message)?;
    Ok(Document {
      id: message.id.to_string(),
      fields,
    })
  }

  fn document_to_message(&self, doc: &Document) -> Result<SearchableMessage, AppError> {
    serde_json::from_value(doc.fields.clone())
      .map_err(|e| AppError::InvalidInput(format!("Deserialization failed: {}", e)))
  }

  async fn documents_to_messages_parallel(&self, docs: &[Document]) -> Vec<SearchableMessage> {
    let docs_clone = docs.to_vec();

    tokio::task::spawn_blocking(move || {
      use rayon::prelude::*;

      docs_clone
        .into_par_iter()
        .filter_map(|doc| serde_json::from_value::<SearchableMessage>(doc.fields.clone()).ok())
        .collect()
    })
    .await
    .unwrap_or_else(|_| {
      docs
        .iter()
        .filter_map(|doc| serde_json::from_value(doc.fields.clone()).ok())
        .collect()
    })
  }

  fn build_search_query(
    &self,
    query: &str,
    filters: Option<serde_json::Value>,
    limit: u32,
    offset: u32,
  ) -> SearchQuery {
    SearchQuery {
      query: query.to_string(),
      filters,
      limit,
      offset,
    }
  }

  fn validate_search_params_fast(&self, query: &str, limit: u32) -> Result<(), AppError> {
    if query.is_empty() || query.chars().all(|c| c.is_whitespace()) {
      return Err(AppError::InvalidInput(
        "Search query cannot be empty".to_string(),
      ));
    }

    if query.len() > 500 {
      return Err(AppError::InvalidInput(
        "Search query too long (max 500 characters)".to_string(),
      ));
    }

    if limit > self.config.max_results {
      return Err(AppError::InvalidInput(format!(
        "Limit too high (max {})",
        self.config.max_results
      )));
    }

    Ok(())
  }

  async fn search_with_timeout(&self, search_query: SearchQuery) -> Result<SearchResult, AppError> {
    let timeout_duration = Duration::from_millis(self.config.search_timeout_ms);

    timeout(
      timeout_duration,
      self
        .search_service
        .search(&self.config.index_name, search_query),
    )
    .await
    .map_err(|_| AppError::ServiceUnavailable("Search timeout".to_string()))?
    .map_err(|e| AppError::ServiceUnavailable(format!("Search failed: {}", e)))
  }
}

#[async_trait]
impl SearchApplicationServiceTrait for SearchApplicationService {
  #[instrument(skip(self), fields(chat_id = %chat_id, user_id = %user_id, query = %query))]
  async fn search_messages_in_chat(
    &self,
    chat_id: ChatId,
    query: &str,
    user_id: UserId,
    limit: u32,
    offset: u32,
  ) -> Result<MessageSearchResults, AppError> {
    if !self.config.enabled {
      return Err(AppError::ServiceUnavailable(
        "Search service is disabled".to_string(),
      ));
    }

    self.validate_search_params_fast(query, limit)?;

    let chat_id_i64 = i64::from(chat_id);

    // 1. Check cache first
    let cache_key =
      self
        .search_cache
        .build_search_results_key(query, Some(chat_id_i64), None, limit, offset);

    if let Ok(Some(cached_result)) = self
      .search_cache
      .get_search_results::<MessageSearchResults>(&cache_key)
      .await
    {
      debug!("Cache hit for chat search: {}", query);
      return Ok(cached_result);
    }

    // 2. Perform search
    let filters = serde_json::json!({
      "chat_id": chat_id_i64
    });

    let search_query = self.build_search_query(query, Some(filters), limit, offset);
    let search_result = self.search_with_timeout(search_query).await?;

    let messages = self
      .documents_to_messages_parallel(&search_result.hits)
      .await;

    let results = MessageSearchResults {
      hits: messages,
      total: search_result.total,
      took_ms: search_result.took_ms,
      query: query.to_string(),
      page: SearchPage {
        offset,
        limit,
        has_more: (offset as u64 + limit as u64) < search_result.total,
      },
    };

    // 3. Cache results
    let _ = self
      .search_cache
      .cache_search_results(cache_key, results.clone())
      .await;

    debug!(
      chat_id = %chat_id,
      user_id = %user_id,
      query = %query,
      results_count = %results.hits.len(),
      total = %results.total,
      took_ms = %results.took_ms,
      "Chat search completed"
    );

    Ok(results)
  }

  #[instrument(skip(self), fields(user_id = %user_id, workspace_id = %workspace_id, query = %query))]
  async fn global_search_messages(
    &self,
    query: &str,
    user_id: UserId,
    workspace_id: WorkspaceId,
    limit: u32,
    offset: u32,
  ) -> Result<MessageSearchResults, AppError> {
    if !self.config.enabled {
      return Err(AppError::ServiceUnavailable(
        "Search service is disabled".to_string(),
      ));
    }

    self.validate_search_params_fast(query, limit)?;

    let workspace_id_i64 = i64::from(workspace_id);

    // 1. Check cache first
    let cache_key = self.search_cache.build_search_results_key(
      query,
      None,
      Some(workspace_id_i64),
      limit,
      offset,
    );

    if let Ok(Some(cached_result)) = self
      .search_cache
      .get_search_results::<MessageSearchResults>(&cache_key)
      .await
    {
      debug!("Cache hit for global search: {}", query);
      return Ok(cached_result);
    }

    // 2. Perform search
    let filters = serde_json::json!({
      "workspace_id": workspace_id_i64
    });

    let search_query = self.build_search_query(query, Some(filters), limit, offset);
    let search_result = self.search_with_timeout(search_query).await?;

    let messages = self
      .documents_to_messages_parallel(&search_result.hits)
      .await;

    let results = MessageSearchResults {
      hits: messages,
      total: search_result.total,
      took_ms: search_result.took_ms,
      query: query.to_string(),
      page: SearchPage {
        offset,
        limit,
        has_more: (offset as u64 + limit as u64) < search_result.total,
      },
    };

    // 3. Cache results
    let _ = self
      .search_cache
      .cache_search_results(cache_key, results.clone())
      .await;

    debug!(
      user_id = %user_id,
      workspace_id = %workspace_id,
      query = %query,
      results_count = %results.hits.len(),
      total = %results.total,
      took_ms = %results.took_ms,
      "Global search completed"
    );

    Ok(results)
  }

  #[instrument(skip(self, messages), fields(count = %messages.len()))]
  async fn index_messages_batch(&self, messages: &[SearchableMessage]) -> Result<(), AppError> {
    if !self.config.enabled || messages.is_empty() {
      return Ok(());
    }

    for message in messages {
      let document = self.message_to_document(message)?;
      self
        .search_service
        .index_document(&self.config.index_name, document)
        .await
        .map_err(|e| AppError::ServiceUnavailable(format!("Indexing failed: {}", e)))?;
    }

    info!(count = %messages.len(), "Messages indexed successfully in batch");
    Ok(())
  }

  #[instrument(skip(self, message_ids), fields(count = %message_ids.len()))]
  async fn remove_messages_from_index_batch(
    &self,
    message_ids: &[MessageId],
  ) -> Result<(), AppError> {
    if !self.config.enabled || message_ids.is_empty() {
      return Ok(());
    }

    for message_id in message_ids {
      self
        .search_service
        .delete_document(&self.config.index_name, &message_id.to_string())
        .await
        .map_err(|e| AppError::ServiceUnavailable(format!("Deletion failed: {}", e)))?;
    }

    info!(count = %message_ids.len(), "Messages removed from index in batch");
    Ok(())
  }

  #[instrument(skip(self, messages), fields(count = %messages.len()))]
  async fn update_messages_in_index_batch(
    &self,
    messages: &[SearchableMessage],
  ) -> Result<(), AppError> {
    if !self.config.enabled || messages.is_empty() {
      return Ok(());
    }

    for message in messages {
      let document = self.message_to_document(message)?;
      self
        .search_service
        .index_document(&self.config.index_name, document)
        .await
        .map_err(|e| AppError::ServiceUnavailable(format!("Update failed: {}", e)))?;
    }

    info!(count = %messages.len(), "Messages updated in index in batch");
    Ok(())
  }

  #[instrument(skip(self), fields(chat_id = %chat_id))]
  async fn reindex_chat_messages(&self, chat_id: ChatId) -> Result<u64, AppError> {
    if !self.config.enabled {
      return Ok(0);
    }

    info!("Starting reindex for chat {}", chat_id);

    let total_indexed = 0u64;

    // TODO: Implement actual reindexing logic

    info!(
      "Completed reindex for chat {}, indexed {} messages",
      chat_id, total_indexed
    );
    Ok(total_indexed)
  }

  #[instrument(skip(self), fields(partial_query = %partial_query))]
  async fn get_search_suggestions(
    &self,
    partial_query: &str,
    limit: u32,
  ) -> Result<Vec<String>, AppError> {
    if !self.config.enabled {
      return Ok(vec![]);
    }

    debug!(
      "Search suggestions not yet implemented for: {}",
      partial_query
    );
    Ok(vec![])
  }
}

pub fn create_search_application_service(
  search_service: Arc<dyn CoreSearchService>,
  search_cache: Arc<SearchCacheService>,
) -> SearchApplicationService {
  SearchApplicationService::new(search_service, search_cache, SearchConfig::default())
}

pub fn create_search_application_service_from_config(
  search_service: Arc<dyn CoreSearchService>,
  search_cache: Arc<SearchCacheService>,
  app_config: &AppConfig,
) -> SearchApplicationService {
  SearchApplicationService::from_app_config(search_service, search_cache, app_config)
}
