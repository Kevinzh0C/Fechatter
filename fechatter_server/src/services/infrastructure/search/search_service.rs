use std::sync::Arc;

use async_trait::async_trait;
use meilisearch_sdk::client::Client as MeilisearchClient;
use meilisearch_sdk::search::SearchResult as MeilisearchSearchResult;
use meilisearch_sdk::task_info::TaskInfo;
use tracing::info;

use crate::{config::SearchConfig, error::AppError};
use fechatter_core::models::{SearchMessages, SearchableMessage};

// ================================================================================================
// Domain Types with Strong Typing (Newtype Pattern)
// ================================================================================================

/// Unified result type for all search operations
pub type SearchServiceResult<T> = std::result::Result<T, AppError>;

/// Strongly-typed batch size to prevent mixing with other numeric values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchSize(usize);

impl BatchSize {
  /// Maximum safe batch size for Meilisearch (stays under 10MB limit)
  pub const MAX_SAFE: Self = Self(500);

  /// Creates a new batch size, clamping to maximum safe value
  pub fn new(size: usize) -> Self {
    Self(size.min(Self::MAX_SAFE.0))
  }

  pub fn get(self) -> usize {
    self.0
  }
}

/// Strongly-typed timeout duration to prevent confusion with other durations
#[derive(Debug, Clone, Copy)]
pub struct TaskTimeout(std::time::Duration);

impl TaskTimeout {
  /// Default timeout for Meilisearch operations (30 seconds)
  pub const DEFAULT: Self = Self(std::time::Duration::from_secs(30));

  pub fn from_millis(millis: u64) -> Self {
    Self(std::time::Duration::from_millis(millis))
  }

  pub fn get(self) -> std::time::Duration {
    self.0
  }
}

/// Type-safe pagination with built-in validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
  offset: usize,
  limit: usize,
}

impl Pagination {
  /// Maximum allowed pagination limit to prevent resource exhaustion
  const MAX_LIMIT: usize = 1_000;

  /// Creates validated pagination parameters
  pub fn new(offset: i64, limit: i64) -> SearchServiceResult<Self> {
    if offset < 0 {
      return Err(AppError::InvalidInput(
        "Pagination offset cannot be negative".into(),
      ));
    }

    if limit <= 0 || limit > Self::MAX_LIMIT as i64 {
      return Err(AppError::InvalidInput(format!(
        "Pagination limit must be between 1 and {} (got: {})",
        Self::MAX_LIMIT,
        limit
      )));
    }

    Ok(Self {
      offset: offset as usize,
      limit: limit as usize,
    })
  }

  pub fn offset(&self) -> usize {
    self.offset
  }

  pub fn limit(&self) -> usize {
    self.limit
  }
}

// ================================================================================================
// Traits for Testability and Abstraction
// ================================================================================================

/// Trait for search backend operations to enable testing and multiple implementations
#[async_trait]
pub trait SearchBackend {
  async fn search_messages(
    &self,
    query: &str,
    filters: &SearchFilters,
    pagination: Pagination,
  ) -> SearchServiceResult<RawSearchResults>;

  async fn index_documents(&self, documents: &[SearchableMessage]) -> SearchServiceResult<()>;

  async fn delete_documents(&self, ids: &[String]) -> SearchServiceResult<()>;

  async fn ensure_index_exists(&self) -> SearchServiceResult<()>;
}

/// Search filters with type safety
#[derive(Debug, Clone)]
pub struct SearchFilters {
  pub chat_id: i64,
  pub workspace_id: i64,
}

impl SearchFilters {
  pub fn new(chat_id: i64, workspace_id: i64) -> Self {
    Self {
      chat_id,
      workspace_id,
    }
  }

  /// Builds a safe filter expression for Meilisearch
  /// Uses integer formatting which is immune to injection attacks
  pub fn to_meilisearch_expression(&self) -> String {
    format!(
      "chat_id = {} AND workspace_id = {}",
      self.chat_id, self.workspace_id
    )
  }
}

/// Raw search results before processing
#[derive(Debug)]
pub struct RawSearchResults {
  pub hits: Vec<SearchHit>,
  pub total_hits: Option<usize>,
  pub query_time_ms: u64,
}

/// Individual search hit with score information
#[derive(Debug)]
pub struct SearchHit {
  pub document: SearchableMessage,
  pub score: Option<f32>,
}

// ================================================================================================
// Configuration Builder Pattern
// ================================================================================================

/// Search service configuration builder with validation
#[derive(Debug, Clone)]
pub struct SearchServiceBuilder {
  config: SearchConfig,
  batch_size: BatchSize,
  task_timeout: TaskTimeout,
}

impl SearchServiceBuilder {
  pub fn new(config: SearchConfig) -> Self {
    Self {
      config,
      batch_size: BatchSize::MAX_SAFE,
      task_timeout: TaskTimeout::DEFAULT,
    }
  }

  pub fn with_batch_size(mut self, size: usize) -> Self {
    self.batch_size = BatchSize::new(size);
    self
  }

  pub fn with_task_timeout(mut self, timeout: TaskTimeout) -> Self {
    self.task_timeout = timeout;
    self
  }

  pub fn build(self) -> SearchServiceResult<SearchService> {
    SearchService::new(self.config, self.batch_size, self.task_timeout)
  }
}

// ================================================================================================
// Core Search Service
// ================================================================================================

/// High-performance, type-safe search service for messages
///
/// Features:
/// - Strongly-typed API to prevent runtime errors
/// - Configurable batch processing for large datasets  
/// - Comprehensive error handling with context
/// - Async-first design optimized for high concurrency
/// - Test-friendly architecture with dependency injection
pub struct SearchService {
  backend: Box<dyn SearchBackend + Send + Sync>,
  config: SearchConfig,
  batch_size: BatchSize,
}

impl SearchService {
  /// Creates a new search service with Meilisearch backend
  pub fn new(
    config: SearchConfig,
    batch_size: BatchSize,
    task_timeout: TaskTimeout,
  ) -> SearchServiceResult<Self> {
    let backend = MeilisearchBackend::new(&config, task_timeout)?;

    Ok(Self {
      backend: Box::new(backend),
      config,
      batch_size,
    })
  }

  /// Creates a service with custom backend (useful for testing)
  pub fn with_backend(backend: Box<dyn SearchBackend + Send + Sync>, config: SearchConfig) -> Self {
    Self {
      backend,
      config,
      batch_size: BatchSize::MAX_SAFE,
    }
  }

  /// Initializes search indexes if search is enabled
  pub async fn initialize(&self) -> SearchServiceResult<()> {
    if !self.is_enabled() {
      info!("Search is disabled - skipping initialization");
      return Ok(());
    }

    self.backend.ensure_index_exists().await?;

    info!("Search service initialized successfully");
    Ok(())
  }

  /// Searches for messages with comprehensive validation and error handling
  pub async fn search_messages(
    &self,
    request: &SearchMessages,
  ) -> SearchServiceResult<fechatter_core::models::SearchResult> {
    self.ensure_enabled()?;

    let pagination = Pagination::new(request.offset, request.limit)?;
    let filters = self.build_filters_from_request(request)?;

    let start_time = std::time::Instant::now();
    let raw_results = self
      .backend
      .search_messages(&request.query, &filters, pagination)
      .await?;

    let search_duration = start_time.elapsed();

    self.convert_to_api_results(raw_results, request, search_duration)
  }

  /// Indexes messages in optimally-sized batches
  pub async fn index_messages(&self, messages: &[SearchableMessage]) -> SearchServiceResult<()> {
    if !self.is_enabled() || messages.is_empty() {
      return Ok(());
    }

    let plan = BatchPlan::new(messages.len(), self.batch_size);
    info!(
      "Indexing {} messages in {} batches",
      plan.total_items, plan.total_batches
    );

    for (batch_num, chunk) in messages.chunks(self.batch_size.get()).enumerate() {
      let batch_info = BatchInfo::new(batch_num + 1, plan.total_batches, chunk.len());

      self
        .backend
        .index_documents(chunk)
        .await
        .map_err(|e| AppError::SearchError(format!("Batch {}: {}", batch_info.number, e)))?;

      info!(
        "✓ Indexed batch {}/{} ({} messages)",
        batch_info.number, batch_info.total, batch_info.size
      );
    }

    info!("Successfully indexed {} messages", plan.total_items);
    Ok(())
  }

  /// Deletes messages from search index in batches
  pub async fn delete_messages(&self, message_ids: &[i64]) -> SearchServiceResult<()> {
    if !self.is_enabled() || message_ids.is_empty() {
      return Ok(());
    }

    let plan = BatchPlan::new(message_ids.len(), self.batch_size);
    info!(
      "Deleting {} messages in {} batches",
      plan.total_items, plan.total_batches
    );

    for (batch_num, chunk) in message_ids.chunks(self.batch_size.get()).enumerate() {
      let batch_info = BatchInfo::new(batch_num + 1, plan.total_batches, chunk.len());
      let ids: Vec<String> = chunk.iter().map(|id| id.to_string()).collect();

      self
        .backend
        .delete_documents(&ids)
        .await
        .map_err(|e| AppError::SearchError(format!("Batch {}: {}", batch_info.number, e)))?;

      info!(
        "✓ Deleted batch {}/{} ({} messages)",
        batch_info.number, batch_info.total, batch_info.size
      );
    }

    info!("Successfully deleted {} messages", plan.total_items);
    Ok(())
  }

  /// Returns whether search functionality is enabled
  pub fn is_enabled(&self) -> bool {
    self.config.enabled
  }
}

// ================================================================================================
// Private Implementation
// ================================================================================================

impl SearchService {
  fn ensure_enabled(&self) -> SearchServiceResult<()> {
    if !self.is_enabled() {
      return Err(AppError::SearchError(
        "Search functionality is disabled".into(),
      ));
    }
    Ok(())
  }

  fn build_filters_from_request(
    &self,
    request: &SearchMessages,
  ) -> SearchServiceResult<SearchFilters> {
    let chat_id = request.chat_id.ok_or_else(|| {
      AppError::InvalidInput("chat_id is required (workspace-wide search not yet supported)".into())
    })?;

    Ok(SearchFilters::new(chat_id.0, request.workspace_id.0))
  }

  fn convert_to_api_results(
    &self,
    raw_results: RawSearchResults,
    request: &SearchMessages,
    search_duration: std::time::Duration,
  ) -> SearchServiceResult<fechatter_core::models::SearchResult> {
    let messages: Vec<SearchableMessage> = raw_results
      .hits
      .into_iter()
      .map(|hit| {
        let mut message = hit.document;
        message.relevance_score = hit.score;
        message
      })
      .collect();

    let total_hits = raw_results.total_hits.unwrap_or(messages.len());
    let has_more = (request.offset + request.limit) < total_hits as i64;

    Ok(fechatter_core::models::SearchResult {
      messages,
      total_hits,
      has_more,
      query_time_ms: search_duration.as_millis() as u64,
    })
  }
}

// ================================================================================================
// Meilisearch Backend Implementation
// ================================================================================================

pub struct MeilisearchBackend {
  client: Arc<MeilisearchClient>,
  config: SearchConfig,
  task_timeout: TaskTimeout,
}

impl MeilisearchBackend {
  pub fn new(config: &SearchConfig, task_timeout: TaskTimeout) -> SearchServiceResult<Self> {
    let api_key = if config.meilisearch.api_key.is_empty() {
      None
    } else {
      Some(config.meilisearch.api_key.as_str())
    };

    let client = MeilisearchClient::new(&config.meilisearch.url, api_key)
      .map_err(|e| AppError::SearchError(format!("Failed to create Meilisearch client: {}", e)))?;

    Ok(Self {
      client: Arc::new(client),
      config: config.clone(),
      task_timeout,
    })
  }

  async fn wait_for_task(&self, task: TaskInfo, operation: &str) -> SearchServiceResult<()> {
    let completed = task
      .wait_for_completion(&self.client, None, Some(self.task_timeout.get()))
      .await
      .map_err(|e| AppError::SearchError(format!("Timeout waiting for {}: {}", operation, e)))?;

    match completed {
      meilisearch_sdk::tasks::Task::Succeeded { .. } => Ok(()),
      meilisearch_sdk::tasks::Task::Failed { content } => Err(AppError::SearchError(format!(
        "{} failed: {:?}",
        operation, content.error
      ))),
      status => Err(AppError::SearchError(format!(
        "{} ended with unexpected status: {:?}",
        operation, status
      ))),
    }
  }

  fn extract_score(hit: &MeilisearchSearchResult<SearchableMessage>) -> Option<f32> {
    // v0.28.0 uses ranking_score field
    hit.ranking_score.map(|score| score as f32)
  }
}

#[async_trait]
impl SearchBackend for MeilisearchBackend {
  async fn search_messages(
    &self,
    query: &str,
    filters: &SearchFilters,
    pagination: Pagination,
  ) -> SearchServiceResult<RawSearchResults> {
    let index = self
      .client
      .index(&self.config.meilisearch.indexes.messages.name);

    let start = std::time::Instant::now();
    let results = index
      .search()
      .with_query(query)
      .with_filter(&filters.to_meilisearch_expression())
      .with_offset(pagination.offset())
      .with_limit(pagination.limit())
      .with_sort(&["created_at:desc"])
      .execute::<SearchableMessage>()
      .await
      .map_err(|e| AppError::SearchError(format!("Search failed: {}", e)))?;

    let query_time = start.elapsed();

    let hits = results
      .hits
      .into_iter()
      .map(|hit| SearchHit {
        score: Self::extract_score(&hit),
        document: hit.result,
      })
      .collect();

    Ok(RawSearchResults {
      hits,
      total_hits: results.estimated_total_hits,
      query_time_ms: query_time.as_millis() as u64,
    })
  }

  async fn index_documents(&self, documents: &[SearchableMessage]) -> SearchServiceResult<()> {
    let index = self
      .client
      .index(&self.config.meilisearch.indexes.messages.name);
    let primary_key = &self.config.meilisearch.indexes.messages.primary_key;

    let task = index
      .add_documents(documents, Some(primary_key))
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to submit indexing task: {}", e)))?;

    self.wait_for_task(task, "document indexing").await
  }

  async fn delete_documents(&self, ids: &[String]) -> SearchServiceResult<()> {
    let index = self
      .client
      .index(&self.config.meilisearch.indexes.messages.name);

    let task = index
      .delete_documents(ids)
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to submit deletion task: {}", e)))?;

    self.wait_for_task(task, "document deletion").await
  }

  async fn ensure_index_exists(&self) -> SearchServiceResult<()> {
    let index_config = &self.config.meilisearch.indexes.messages;

    let create_task = self
      .client
      .create_index(&index_config.name, Some(&index_config.primary_key))
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to create index: {}", e)))?;

    self.wait_for_task(create_task, "index creation").await?;

    // Configure index settings
    let index = self.client.index(&index_config.name);

    // Set searchable attributes
    let searchable_task = index
      .set_searchable_attributes(&["content", "sender_name"])
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set searchable attributes: {}", e)))?;

    self
      .wait_for_task(searchable_task, "searchable attributes setup")
      .await?;

    // Set filterable attributes
    let filterable_task = index
      .set_filterable_attributes(&["chat_id", "workspace_id"])
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set filterable attributes: {}", e)))?;

    self
      .wait_for_task(filterable_task, "filterable attributes setup")
      .await?;

    // Set sortable attributes
    let sortable_task = index
      .set_sortable_attributes(&["created_at"])
      .await
      .map_err(|e| AppError::SearchError(format!("Failed to set sortable attributes: {}", e)))?;

    self
      .wait_for_task(sortable_task, "sortable attributes setup")
      .await?;

    info!(
      "Search index '{}' configured successfully",
      index_config.name
    );
    Ok(())
  }
}

// ================================================================================================
// Helper Types
// ================================================================================================

#[derive(Debug, Clone)]
struct BatchPlan {
  total_items: usize,
  total_batches: usize,
}

impl BatchPlan {
  fn new(total_items: usize, batch_size: BatchSize) -> Self {
    let total_batches = (total_items + batch_size.get() - 1) / batch_size.get();
    Self {
      total_items,
      total_batches,
    }
  }
}

#[derive(Debug, Clone)]
struct BatchInfo {
  number: usize,
  total: usize,
  size: usize,
}

impl BatchInfo {
  fn new(number: usize, total: usize, size: usize) -> Self {
    Self {
      number,
      total,
      size,
    }
  }
}

// ================================================================================================
// Public API for Service Creation
// ================================================================================================

/// Creates a search service with default configuration
pub fn create_search_service(config: SearchConfig) -> SearchServiceResult<SearchService> {
  SearchServiceBuilder::new(config).build()
}

/// Creates a search service with custom batch size
pub fn create_search_service_with_batch_size(
  config: SearchConfig,
  batch_size: usize,
) -> SearchServiceResult<SearchService> {
  SearchServiceBuilder::new(config)
    .with_batch_size(batch_size)
    .build()
}
