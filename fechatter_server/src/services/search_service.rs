use std::sync::Arc;

use anyhow::Result;
use meilisearch_sdk::client::Client as MeilisearchClient;
use meilisearch_sdk::indexes::Index;
use tracing::info;

use crate::config::SearchConfig;
use crate::error::AppError;
use fechatter_core::models::{SearchMessages, SearchResult, SearchableMessage};

#[derive(Clone)]
pub struct SearchService {
  meilisearch_client: Arc<MeilisearchClient>,
  config: SearchConfig,
}

impl SearchService {
  pub fn new(search_config: SearchConfig) -> Result<Self, AppError> {
    // Handle empty API key - pass None instead of Some("") when API key is empty
    let api_key = if search_config.meilisearch.api_key.is_empty() {
      None
    } else {
      Some(search_config.meilisearch.api_key.as_str())
    };

    let client = MeilisearchClient::new(&search_config.meilisearch.url, api_key)
      .map_err(|e| AppError::SearchError(format!("Failed to create Meilisearch client: {}", e)))?;

    Ok(Self {
      meilisearch_client: Arc::new(client),
      config: search_config,
    })
  }

  /// Initialize search indexes
  pub async fn initialize_indexes(&self) -> Result<(), AppError> {
    if !self.config.enabled {
      info!("Search is disabled, skipping index initialization");
      return Ok(());
    }

    let index_config = &self.config.meilisearch.indexes.messages;

    // Create or get the messages index
    let _task = self
      .meilisearch_client
      .create_index(&index_config.name, Some(&index_config.primary_key))
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to create index: {}", e)))?;

    // Get the index for configuration
    let index = self.meilisearch_client.index(&index_config.name);

    // Configure index settings
    self.configure_index_settings(&index).await?;

    info!(
      "Search index '{}' initialized successfully",
      index_config.name
    );
    Ok(())
  }

  async fn configure_index_settings(&self, index: &Index) -> Result<(), AppError> {
    // Set searchable attributes - 只搜索内容和发送者名称
    let searchable_fields = vec!["content", "sender_name"];
    index
      .set_searchable_attributes(&searchable_fields)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set searchable attributes: {}", e)))?;

    // Set filterable attributes - 用于聊天内搜索的必要过滤器
    let filterable_fields = vec!["chat_id", "workspace_id"];
    index
      .set_filterable_attributes(&filterable_fields)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set filterable attributes: {}", e)))?;

    // Set sortable attributes - 只按创建时间排序
    let sortable_fields = vec!["created_at"];
    index
      .set_sortable_attributes(&sortable_fields)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set sortable attributes: {}", e)))?;

    Ok(())
  }

  /// Search messages in a chat - 简化的搜索实现
  pub async fn search_messages(
    &self,
    search_request: &SearchMessages,
  ) -> Result<SearchResult, AppError> {
    if !self.config.enabled {
      return Err(AppError::SearchError("Search is disabled".to_string()));
    }

    let start_time = std::time::Instant::now();
    let index = self
      .meilisearch_client
      .index(&self.config.meilisearch.indexes.messages.name);

    // Build search query
    let mut search_query = index.search();
    search_query.with_query(&search_request.query);

    // Apply mandatory filters
    let chat_id = search_request
      .chat_id
      .ok_or_else(|| AppError::InvalidInput("chat_id is required for search".to_string()))?;

    let filter = format!(
      "chat_id = {} AND workspace_id = {}",
      chat_id, search_request.workspace_id
    );
    search_query.with_filter(&filter);

    // Apply pagination
    search_query.with_offset(search_request.offset as usize);
    search_query.with_limit(search_request.limit as usize);

    // Sort by relevance (default) or time
    search_query.with_sort(&["created_at:desc"]);

    // Execute search
    let search_result = search_query
      .execute::<SearchableMessage>()
      .await
      .map_err(|e| AppError::SearchError(format!("Search execution failed: {}", e)))?;

    let query_time_ms = start_time.elapsed().as_millis() as u64;

    // Process results
    let messages: Vec<SearchableMessage> = search_result
      .hits
      .into_iter()
      .map(|hit| {
        let mut message = hit.result;
        message.relevance_score = hit.ranking_score.map(|s| s as f32);
        message
      })
      .collect();

    // Calculate result info
    let total_hits = search_result.estimated_total_hits.unwrap_or(messages.len());
    let has_more = (search_request.offset + search_request.limit) < total_hits as i64;

    Ok(SearchResult {
      messages,
      total_hits,
      has_more,
      query_time_ms,
    })
  }

  /// Check if search is enabled
  pub fn is_enabled(&self) -> bool {
    self.config.enabled
  }

  /// 批量索引消息（异步索引专用）
  pub async fn batch_index_messages(&self, messages: &[SearchableMessage]) -> Result<(), AppError> {
    if !self.config.enabled || messages.is_empty() {
      return Ok(());
    }

    let index_config = &self.config.meilisearch.indexes.messages;
    let index = self.meilisearch_client.index(&index_config.name);

    // 批量添加文档
    index
      .add_documents(messages, Some(&index_config.primary_key))
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to batch index messages: {}", e)))?;

    info!("Successfully batch indexed {} messages", messages.len());
    Ok(())
  }

  /// 批量删除消息（异步索引专用）
  pub async fn batch_delete_messages(&self, message_ids: &[i64]) -> Result<(), AppError> {
    if !self.config.enabled || message_ids.is_empty() {
      return Ok(());
    }

    let index_config = &self.config.meilisearch.indexes.messages;
    let index = self.meilisearch_client.index(&index_config.name);

    // 批量删除文档
    let string_ids: Vec<String> = message_ids.iter().map(|id| id.to_string()).collect();
    index
      .delete_documents(&string_ids)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to batch delete messages: {}", e)))?;

    info!("Successfully batch deleted {} messages", message_ids.len());
    Ok(())
  }
}
