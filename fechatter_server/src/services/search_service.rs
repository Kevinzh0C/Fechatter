use std::sync::Arc;

use anyhow::Result;
use meilisearch_sdk::client::Client as MeilisearchClient;
use meilisearch_sdk::indexes::Index;
use tracing::{error, info, warn};

use crate::config::SearchConfig;
use crate::error::AppError;
use fechatter_core::models::Message;
use fechatter_core::models::{
  DateFacet, FacetCount, PaginationInfo, SearchFacets, SearchMessages, SearchMetadata,
  SearchResult, SearchType, SearchableMessage, SenderFacet, TextHighlight,
};

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
    let index_config = &self.config.meilisearch.indexes.messages;

    // Set searchable attributes
    index
      .set_searchable_attributes(&index_config.searchable_fields)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set searchable attributes: {}", e)))?;

    // Set filterable attributes
    index
      .set_filterable_attributes(&index_config.filterable_fields)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set filterable attributes: {}", e)))?;

    // Set sortable attributes
    index
      .set_sortable_attributes(&index_config.sortable_fields)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set sortable attributes: {}", e)))?;

    // Set displayed attributes
    index
      .set_displayed_attributes(&index_config.displayed_fields)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set displayed attributes: {}", e)))?;

    Ok(())
  }

  /// Index a message for search
  pub async fn index_message(
    &self,
    message: &Message,
    chat_name: &str,
    sender_name: &str,
    chat_type: &str,
    workspace_id: i64,
  ) -> Result<(), AppError> {
    if !self.config.enabled {
      return Ok(());
    }

    let index_config = &self.config.meilisearch.indexes.messages;
    let index = self.meilisearch_client.index(&index_config.name);

    // Extract file names from file URLs for searchable text
    let file_names = if let Some(ref files) = message.files {
      files
        .iter()
        .filter_map(|url| {
          // Extract filename from URL like "/files/123/abc123.jpg"
          url.split('/').last().and_then(|filename| {
            // Remove hash prefix and get original name if possible
            // Format: "hash.ext" -> keep as is for now
            Some(filename.to_string())
          })
        })
        .collect::<Vec<_>>()
        .join(" ")
    } else {
      String::new()
    };

    let searchable_message = SearchableMessage {
      id: message.id,
      chat_id: message.chat_id,
      workspace_id,
      sender_id: message.sender_id,
      sender_name: sender_name.to_string(),
      content: message.content.clone(),
      content_highlights: vec![], // Will be populated during search
      files: message.files.clone(),
      file_names,
      created_at: message.created_at,
      chat_name: chat_name.to_string(),
      chat_type: chat_type.to_string(),
      relevance_score: None, // Will be populated during search
    };

    index
      .add_documents(&[searchable_message], Some(&index_config.primary_key))
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to index message: {}", e)))?;

    Ok(())
  }

  /// Search messages with chat-type-aware strategy
  pub async fn search_messages(
    &self,
    search_request: &SearchMessages,
  ) -> Result<SearchResult, AppError> {
    if !self.config.enabled {
      return Err(AppError::SearchError("Search is disabled".to_string()));
    }

    let start_time = std::time::Instant::now();
    let index_config = &self.config.meilisearch.indexes.messages;
    let index = self.meilisearch_client.index(&index_config.name);

    // Build search query
    let mut search_query = index.search();
    search_query.with_query(&search_request.query);

    // Apply filters - chat_id is mandatory for in-chat search
    let mut filters = vec![];

    if let Some(chat_id) = search_request.chat_id {
      filters.push(format!("chat_id = {}", chat_id));
    } else {
      return Err(AppError::InvalidInput(
        "chat_id is required for in-chat search".to_string(),
      ));
    }

    // Add workspace filter (useful for data isolation even in private chats)
    filters.push(format!("workspace_id = {}", search_request.workspace_id));

    // Optional sender filter
    if let Some(sender_id) = search_request.sender_id {
      filters.push(format!("sender_id = {}", sender_id));
    }

    // Optional date range filter
    if let Some(date_range) = &search_request.date_range {
      if let Some(start) = date_range.start {
        filters.push(format!("created_at >= {}", start.timestamp()));
      }
      if let Some(end) = date_range.end {
        filters.push(format!("created_at <= {}", end.timestamp()));
      }
    }

    // Join filters before applying to search query
    let filter_string = if !filters.is_empty() {
      Some(filters.join(" AND "))
    } else {
      None
    };

    // Apply filter if exists
    if let Some(ref filter) = filter_string {
      search_query.with_filter(filter);
    }

    // Apply pagination
    let offset = search_request.offset.unwrap_or(0);
    let limit = search_request.limit.unwrap_or(20).min(100); // Cap at 100

    search_query.with_offset(offset as usize);
    search_query.with_limit(limit as usize);

    // Apply sorting
    match search_request
      .sort_order
      .as_ref()
      .unwrap_or(&fechatter_core::models::SortOrder::Relevance)
    {
      fechatter_core::models::SortOrder::Newest => {
        search_query.with_sort(&["created_at:desc"]);
      }
      fechatter_core::models::SortOrder::Oldest => {
        search_query.with_sort(&["created_at:asc"]);
      }
      fechatter_core::models::SortOrder::Relevance => {
        // Default Meilisearch relevance sorting
      }
    }

    // Enable highlighting for better search results
    search_query.with_show_matches_position(true);

    // Execute search
    let search_result = search_query
      .execute::<SearchableMessage>()
      .await
      .map_err(|e| AppError::SearchError(format!("Search execution failed: {}", e)))?;

    let query_time_ms = start_time.elapsed().as_millis() as u64;

    // Process results and add highlights
    let messages: Vec<SearchableMessage> = search_result
      .hits
      .into_iter()
      .map(|hit| {
        let mut message = hit.result;

        // Add highlights if matches are available
        if let Some(matches) = hit.formatted_result {
          if let Some(content_match) = matches.get("content") {
            message.content_highlights =
              self.extract_highlights(content_match, &search_request.query);
          }
        }

        message.relevance_score = Some(hit.ranking_score.unwrap_or(0.0));
        message
      })
      .collect();

    // Calculate pagination info
    let total_hits = search_result.estimated_total_hits.unwrap_or(messages.len());
    let has_more = (offset + limit as i64) < total_hits as i64;
    let total_pages = if limit > 0 {
      Some((total_hits + limit as usize - 1) / limit as usize)
    } else {
      None
    };

    let pagination = PaginationInfo {
      offset,
      limit,
      has_more,
      total_pages,
    };

    // Generate facets optimized for in-chat search
    let facets = self.generate_facets(search_request, &filter_string).await?;

    // Build search metadata
    let metadata = SearchMetadata {
      original_query: search_request.query.clone(),
      search_type: search_request.search_type.clone(),
      filters_applied: filters,
      indexed_fields: index_config.searchable_fields.clone(),
      facets: Some(facets),
    };

    Ok(SearchResult {
      messages,
      pagination,
      total_hits,
      query_time_ms,
      search_metadata: metadata,
    })
  }

  /// Generate facets optimized for in-chat search
  async fn generate_facets(
    &self,
    search_request: &SearchMessages,
    base_filter: &Option<String>,
  ) -> Result<fechatter_core::models::SearchFacets, AppError> {
    let index_config = &self.config.meilisearch.indexes.messages;
    let index = self.meilisearch_client.index(&index_config.name);

    // For in-chat search, chat_types facet is meaningless since we're already in a specific chat
    // Return empty chat_types but keep meaningful facets
    let chat_types = vec![];

    // Generate date histogram (useful for temporal analysis)
    let date_histogram = self
      .generate_date_facets(&index, search_request, base_filter)
      .await?;

    // Generate top senders (useful for finding messages from specific people)
    let top_senders = self
      .generate_sender_facets(&index, search_request, base_filter)
      .await?;

    Ok(fechatter_core::models::SearchFacets {
      chat_types,
      date_histogram,
      top_senders,
    })
  }

  /// Generate date facets (histogram)
  async fn generate_date_facets(
    &self,
    index: &Index,
    search_request: &SearchMessages,
    base_filter: &Option<String>,
  ) -> Result<Vec<fechatter_core::models::DateFacet>, AppError> {
    // For date histogram, we'll create buckets for the last 30 days
    let now = chrono::Utc::now();
    let mut date_facets = Vec::new();

    for days_ago in 0..30 {
      let date = now - chrono::Duration::days(days_ago);
      let start_of_day = date.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
      let end_of_day = date.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

      // Create date-specific query
      let mut date_query = index.search();
      date_query.with_query(&search_request.query);

      let mut date_filters = Vec::new();
      if let Some(filter) = base_filter {
        date_filters.push(filter.clone());
      }
      date_filters.push(format!("created_at >= {}", start_of_day.timestamp()));
      date_filters.push(format!("created_at <= {}", end_of_day.timestamp()));

      let date_filter_string = date_filters.join(" AND ");
      date_query.with_filter(&date_filter_string);
      date_query.with_limit(0); // Just count

      let result = date_query
        .execute::<SearchableMessage>()
        .await
        .map_err(|e| AppError::SearchError(format!("Date facet query failed: {}", e)))?;

      let count = result.estimated_total_hits.unwrap_or(0);
      if count > 0 {
        date_facets.push(fechatter_core::models::DateFacet {
          date: start_of_day,
          count,
        });
      }
    }

    Ok(date_facets)
  }

  /// Generate sender facets
  async fn generate_sender_facets(
    &self,
    index: &Index,
    search_request: &SearchMessages,
    base_filter: &Option<String>,
  ) -> Result<Vec<fechatter_core::models::SenderFacet>, AppError> {
    // This is a simplified implementation
    // In a real scenario, you'd use Meilisearch's faceting capabilities
    let mut sender_query = index.search();
    sender_query.with_query(&search_request.query);

    if let Some(filter) = base_filter {
      sender_query.with_filter(filter);
    }

    sender_query.with_limit(100); // Get some results to analyze
    // Note: Meilisearch facets might not be available in all versions
    // sender_query.with_facets(meilisearch_sdk::search::Selectors::Some(&["sender_id", "sender_name"]));

    let result = sender_query
      .execute::<SearchableMessage>()
      .await
      .map_err(|e| AppError::SearchError(format!("Sender facet query failed: {}", e)))?;

    // Count senders from results (simplified approach)
    let mut sender_counts = std::collections::HashMap::new();
    let mut sender_names = std::collections::HashMap::new();

    for hit in result.hits {
      let message = hit.result;
      *sender_counts.entry(message.sender_id).or_insert(0) += 1;
      sender_names.insert(message.sender_id, message.sender_name);
    }

    // Convert to facets and sort by count
    let mut sender_facets: Vec<_> = sender_counts
      .into_iter()
      .map(|(sender_id, count)| fechatter_core::models::SenderFacet {
        sender_id,
        sender_name: sender_names
          .get(&sender_id)
          .cloned()
          .unwrap_or_else(|| "Unknown".to_string()),
        count,
      })
      .collect();

    sender_facets.sort_by(|a, b| b.count.cmp(&a.count));
    sender_facets.truncate(10); // Top 10 senders

    Ok(sender_facets)
  }

  /// Delete a message from search index
  pub async fn delete_message(&self, message_id: i64) -> Result<(), AppError> {
    if !self.config.enabled {
      return Ok(());
    }

    let index_config = &self.config.meilisearch.indexes.messages;
    let index = self.meilisearch_client.index(&index_config.name);

    index
      .delete_document(&message_id.to_string())
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to delete message from index: {}", e)))?;

    Ok(())
  }

  /// Update a message in search index
  pub async fn update_message(
    &self,
    message: &Message,
    chat_name: &str,
    sender_name: &str,
    chat_type: &str,
    workspace_id: i64,
  ) -> Result<(), AppError> {
    // For updates, we can just re-index the message
    self
      .index_message(message, chat_name, sender_name, chat_type, workspace_id)
      .await
  }

  /// Extract text highlights from search matches
  fn extract_highlights(
    &self,
    content_match: &serde_json::Value,
    query: &str,
  ) -> Vec<TextHighlight> {
    let mut highlights = Vec::new();

    if let Some(matches) = content_match.as_array() {
      for match_obj in matches {
        if let (Some(start), Some(length)) = (
          match_obj.get("start").and_then(|v| v.as_u64()),
          match_obj.get("length").and_then(|v| v.as_u64()),
        ) {
          let start = start as usize;
          let end = start + length as usize;

          highlights.push(TextHighlight {
            start,
            end,
            matched_text: query.to_string(), // Simplified - in production, extract actual matched text
          });
        }
      }
    }

    highlights
  }

  /// Check if search is enabled
  pub fn is_enabled(&self) -> bool {
    self.config.enabled
  }
}
