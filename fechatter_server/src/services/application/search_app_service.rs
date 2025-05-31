//! # Search Application Service
//!
//! ## Single Responsibility
//! - Message search orchestration
//! - Search result ranking and filtering
//! - Search indexing coordination
//! - Search analytics and tracking

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use utoipa::ToSchema;

// Core domain models
use fechatter_core::{
  contracts::{Document, SearchQuery, SearchResult, SearchService as CoreSearchService},
  error::CoreError,
  models::{ChatId, MessageId, UserId, WorkspaceId},
};

// Application services
use super::application_event_publisher::{ApplicationEvent, ApplicationEventPublisher};

// Configuration
use crate::config::AppConfig;
use crate::error::AppError;

// ================================================================================================
// Search Application Service Traits
// ================================================================================================

/// Search Application Service Contract - Defines search use cases
#[async_trait]
pub trait SearchApplicationServiceTrait: Send + Sync {
  /// Search messages in a specific chat
  async fn search_messages_in_chat(
    &self,
    chat_id: ChatId,
    query: &str,
    user_id: UserId,
    limit: u32,
    offset: u32,
  ) -> Result<MessageSearchResults, AppError>;

  /// Global message search across all accessible chats
  async fn global_search_messages(
    &self,
    query: &str,
    user_id: UserId,
    workspace_id: WorkspaceId,
    limit: u32,
    offset: u32,
  ) -> Result<MessageSearchResults, AppError>;

  /// Index a message for search
  async fn index_message(&self, message: &SearchableMessage) -> Result<(), AppError>;

  /// Remove message from search index
  async fn remove_message_from_index(&self, message_id: MessageId) -> Result<(), AppError>;

  /// Update message in search index
  async fn update_message_in_index(&self, message: &SearchableMessage) -> Result<(), AppError>;

  /// Reindex all messages for a chat
  async fn reindex_chat_messages(&self, chat_id: ChatId) -> Result<u64, AppError>;

  /// Get search suggestions
  async fn get_search_suggestions(
    &self,
    partial_query: &str,
    limit: u32,
  ) -> Result<Vec<String>, AppError>;
}

// ================================================================================================
// Data Transfer Objects
// ================================================================================================

/// Searchable message representation for indexing
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchableMessage {
  pub id: MessageId,
  pub chat_id: ChatId,
  pub sender_id: UserId,
  pub sender_name: String,
  pub content: String,
  pub files: Option<Vec<String>>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub workspace_id: WorkspaceId,
  /// Search-specific metadata
  pub chat_name: String,
  pub chat_type: String,
}

/// Search results container
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageSearchResults {
  pub hits: Vec<SearchableMessage>,
  pub total: u64,
  pub took_ms: u64,
  pub query: String,
  pub page: SearchPage,
}

/// Search pagination info
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchPage {
  pub offset: u32,
  pub limit: u32,
  pub has_more: bool,
}

/// Search configuration
#[derive(Debug, Clone)]
pub struct SearchConfig {
  pub enabled: bool,
  pub index_name: String,
  pub max_results: u32,
  pub default_limit: u32,
  pub search_timeout_ms: u64,
}

impl Default for SearchConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      index_name: "messages".to_string(),
      max_results: 1000,
      default_limit: 20,
      search_timeout_ms: 5000,
    }
  }
}

// ================================================================================================
// Search Application Service Implementation
// ================================================================================================

/// Search Application Service - Orchestrates search operations
pub struct SearchApplicationService {
  /// Core search service (Meilisearch/Elasticsearch etc.)
  search_service: Arc<dyn CoreSearchService>,
  /// Event publisher for search analytics
  event_publisher: Arc<ApplicationEventPublisher>,
  /// Search configuration
  config: SearchConfig,
}

impl SearchApplicationService {
  /// Create new search application service
  pub fn new(
    search_service: Arc<dyn CoreSearchService>,
    event_publisher: Arc<ApplicationEventPublisher>,
    config: SearchConfig,
  ) -> Self {
    Self {
      search_service,
      event_publisher,
      config,
    }
  }

  /// Create search service from app config
  pub fn from_config(
    search_service: Arc<dyn CoreSearchService>,
    event_publisher: Arc<ApplicationEventPublisher>,
    app_config: &AppConfig,
  ) -> Self {
    let config = SearchConfig {
      enabled: app_config.search.enabled,
      index_name: app_config.search.meilisearch.indexes.messages.name.clone(),
      max_results: app_config.search.meilisearch.settings.pagination_limit,
      default_limit: 20,
      search_timeout_ms: app_config.search.meilisearch.request_timeout_ms,
    };

    Self::new(search_service, event_publisher, config)
  }

  /// Convert SearchableMessage to search document
  fn message_to_document(&self, message: &SearchableMessage) -> Document {
    let mut fields = serde_json::Map::new();
    fields.insert(
      "id".to_string(),
      serde_json::Value::Number(serde_json::Number::from(i64::from(message.id))),
    );
    fields.insert(
      "chat_id".to_string(),
      serde_json::Value::Number(serde_json::Number::from(i64::from(message.chat_id))),
    );
    fields.insert(
      "sender_id".to_string(),
      serde_json::Value::Number(serde_json::Number::from(i64::from(message.sender_id))),
    );
    fields.insert(
      "sender_name".to_string(),
      serde_json::Value::String(message.sender_name.clone()),
    );
    fields.insert(
      "content".to_string(),
      serde_json::Value::String(message.content.clone()),
    );
    fields.insert(
      "chat_name".to_string(),
      serde_json::Value::String(message.chat_name.clone()),
    );
    fields.insert(
      "chat_type".to_string(),
      serde_json::Value::String(message.chat_type.clone()),
    );
    fields.insert(
      "workspace_id".to_string(),
      serde_json::Value::Number(serde_json::Number::from(i64::from(message.workspace_id))),
    );
    fields.insert(
      "created_at".to_string(),
      serde_json::Value::String(message.created_at.to_rfc3339()),
    );

    if let Some(files) = &message.files {
      fields.insert(
        "files".to_string(),
        serde_json::Value::Array(
          files
            .iter()
            .map(|f| serde_json::Value::String(f.clone()))
            .collect(),
        ),
      );
      fields.insert(
        "has_files".to_string(),
        serde_json::Value::Bool(!files.is_empty()),
      );
    } else {
      fields.insert("has_files".to_string(), serde_json::Value::Bool(false));
    }

    Document {
      id: message.id.to_string(),
      fields: serde_json::Value::Object(fields),
    }
  }

  /// Convert search result to SearchableMessage
  fn document_to_message(&self, doc: &Document) -> Result<SearchableMessage, AppError> {
    let fields = doc
      .fields
      .as_object()
      .ok_or_else(|| AppError::InvalidInput("Invalid document format".to_string()))?;

    let get_field = |name: &str| -> Result<&serde_json::Value, AppError> {
      fields
        .get(name)
        .ok_or_else(|| AppError::InvalidInput(format!("Missing field: {}", name)))
    };

    let get_string = |name: &str| -> Result<String, AppError> {
      get_field(name)?
        .as_str()
        .ok_or_else(|| AppError::InvalidInput(format!("Field {} is not a string", name)))
        .map(|s| s.to_string())
    };

    let get_i64 = |name: &str| -> Result<i64, AppError> {
      get_field(name)?
        .as_i64()
        .ok_or_else(|| AppError::InvalidInput(format!("Field {} is not a number", name)))
    };

    let created_at = chrono::DateTime::parse_from_rfc3339(&get_string("created_at")?)
      .map_err(|e| AppError::InvalidInput(format!("Invalid date format: {}", e)))?
      .with_timezone(&chrono::Utc);

    let files = fields.get("files").and_then(|v| v.as_array()).map(|arr| {
      arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
    });

    Ok(SearchableMessage {
      id: MessageId(get_i64("id")?),
      chat_id: ChatId(get_i64("chat_id")?),
      sender_id: UserId(get_i64("sender_id")?),
      sender_name: get_string("sender_name")?,
      content: get_string("content")?,
      files,
      created_at,
      workspace_id: WorkspaceId(get_i64("workspace_id")?),
      chat_name: get_string("chat_name")?,
      chat_type: get_string("chat_type")?,
    })
  }

  /// Build search query with filters
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

  /// Validate search parameters
  fn validate_search_params(&self, query: &str, limit: u32) -> Result<(), AppError> {
    if query.trim().is_empty() {
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

    self.validate_search_params(query, limit)?;

    // Build filters for specific chat and user permissions
    let filters = serde_json::json!({
      "chat_id": i64::from(chat_id)
      // TODO: Add user permission filters
    });

    let search_query = self.build_search_query(query, Some(filters), limit, offset);

    match self
      .search_service
      .search(&self.config.index_name, search_query)
      .await
    {
      Ok(search_result) => {
        let mut messages = Vec::new();
        for doc in &search_result.hits {
          match self.document_to_message(doc) {
            Ok(message) => messages.push(message),
            Err(e) => {
              warn!("Failed to convert search result to message: {}", e);
              continue;
            }
          }
        }

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

        info!(
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
      Err(e) => {
        error!("Search failed: {}", e);
        Err(AppError::ServiceUnavailable(format!(
          "Search failed: {}",
          e
        )))
      }
    }
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

    self.validate_search_params(query, limit)?;

    // Build filters for workspace and user permissions
    let filters = serde_json::json!({
      "workspace_id": i64::from(workspace_id)
      // TODO: Add user permission filters for accessible chats
    });

    let search_query = self.build_search_query(query, Some(filters), limit, offset);

    match self
      .search_service
      .search(&self.config.index_name, search_query)
      .await
    {
      Ok(search_result) => {
        let mut messages = Vec::new();
        for doc in &search_result.hits {
          match self.document_to_message(doc) {
            Ok(message) => messages.push(message),
            Err(e) => {
              warn!("Failed to convert search result to message: {}", e);
              continue;
            }
          }
        }

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

        info!(
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
      Err(e) => {
        error!("Global search failed: {}", e);
        Err(AppError::ServiceUnavailable(format!(
          "Search failed: {}",
          e
        )))
      }
    }
  }

  #[instrument(skip(self, message), fields(message_id = %message.id, chat_id = %message.chat_id))]
  async fn index_message(&self, message: &SearchableMessage) -> Result<(), AppError> {
    if !self.config.enabled {
      return Ok(()); // Silently skip if search is disabled
    }

    let document = self.message_to_document(message);

    match self
      .search_service
      .index_document(&self.config.index_name, document)
      .await
    {
      Ok(()) => {
        info!(
          message_id = %message.id,
          chat_id = %message.chat_id,
          "Message indexed successfully"
        );
        Ok(())
      }
      Err(e) => {
        error!("Failed to index message {}: {}", message.id, e);
        Err(AppError::ServiceUnavailable(format!(
          "Indexing failed: {}",
          e
        )))
      }
    }
  }

  #[instrument(skip(self), fields(message_id = %message_id))]
  async fn remove_message_from_index(&self, message_id: MessageId) -> Result<(), AppError> {
    if !self.config.enabled {
      return Ok(()); // Silently skip if search is disabled
    }

    match self
      .search_service
      .delete_document(&self.config.index_name, &message_id.to_string())
      .await
    {
      Ok(()) => {
        info!(message_id = %message_id, "Message removed from index");
        Ok(())
      }
      Err(e) => {
        error!("Failed to remove message {} from index: {}", message_id, e);
        Err(AppError::ServiceUnavailable(format!(
          "Index removal failed: {}",
          e
        )))
      }
    }
  }

  #[instrument(skip(self, message), fields(message_id = %message.id))]
  async fn update_message_in_index(&self, message: &SearchableMessage) -> Result<(), AppError> {
    if !self.config.enabled {
      return Ok(()); // Silently skip if search is disabled
    }

    let document = self.message_to_document(message);

    match self
      .search_service
      .update_document(&self.config.index_name, &message.id.to_string(), document)
      .await
    {
      Ok(()) => {
        info!(message_id = %message.id, "Message updated in index");
        Ok(())
      }
      Err(e) => {
        error!("Failed to update message {} in index: {}", message.id, e);
        Err(AppError::ServiceUnavailable(format!(
          "Index update failed: {}",
          e
        )))
      }
    }
  }

  #[instrument(skip(self), fields(chat_id = %chat_id))]
  async fn reindex_chat_messages(&self, chat_id: ChatId) -> Result<u64, AppError> {
    if !self.config.enabled {
      return Ok(0);
    }

    // TODO: Implement batch reindexing of chat messages
    // This would involve:
    // 1. Fetching all messages for the chat from database
    // 2. Converting them to SearchableMessage format
    // 3. Batch indexing them
    // 4. Returning count of indexed messages

    warn!("Chat reindexing not yet implemented for chat {}", chat_id);
    Ok(0)
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

    // TODO: Implement search suggestions
    // This could use:
    // 1. Popular search terms
    // 2. User's recent searches
    // 3. Autocomplete based on indexed content

    warn!("Search suggestions not yet implemented");
    Ok(vec![])
  }
}

// ================================================================================================
// Factory Functions
// ================================================================================================

/// Create search application service with default configuration
pub fn create_search_application_service(
  search_service: Arc<dyn CoreSearchService>,
  event_publisher: Arc<ApplicationEventPublisher>,
) -> SearchApplicationService {
  SearchApplicationService::new(search_service, event_publisher, SearchConfig::default())
}

/// Create search application service from app configuration
pub fn create_search_application_service_from_config(
  search_service: Arc<dyn CoreSearchService>,
  event_publisher: Arc<ApplicationEventPublisher>,
  app_config: &AppConfig,
) -> SearchApplicationService {
  SearchApplicationService::from_config(search_service, event_publisher, app_config)
}
