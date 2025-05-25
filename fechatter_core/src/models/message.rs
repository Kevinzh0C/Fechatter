use crate::error::CoreError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::future::Future;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::Message;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMessage {
  pub content: String,
  #[serde(default)]
  pub files: Vec<String>,
  #[serde(default = "Uuid::now_v7")] // Default to UUID v7 (time-based UUID)
  #[schema(value_type = String, format = "uuid", example = "01834abd-8c37-7d82-9206-54b2f6b4f7c4")]
  pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListMessages {
  #[serde(default)]
  pub last_id: Option<i64>,
  #[serde(default)]
  pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct SearchMessages {
  #[validate(length(min = 1, max = 1000))]
  pub query: String,
  pub workspace_id: i64,
  #[serde(default)]
  pub chat_id: Option<i64>,
  #[serde(default)]
  pub sender_id: Option<i64>,
  #[serde(default)]
  pub search_type: SearchType,
  #[serde(default)]
  pub date_range: Option<DateRange>,
  #[serde(default)]
  pub sort_order: Option<SortOrder>,
  #[serde(default)]
  #[validate(range(min = 0, max = 10000))]
  pub offset: Option<i64>,
  #[serde(default)]
  #[validate(range(min = 1, max = 100))]
  pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum SearchType {
  #[serde(rename = "exact")]
  ExactMatch,
  #[serde(rename = "fuzzy")]
  FuzzyMatch,
  #[serde(rename = "fulltext")]
  FullText,
  #[serde(rename = "regex")]
  Regex,
}

impl Default for SearchType {
  fn default() -> Self {
    SearchType::FullText
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DateRange {
  pub start: Option<DateTime<Utc>>,
  pub end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum SortOrder {
  #[serde(rename = "newest")]
  Newest,
  #[serde(rename = "oldest")]
  Oldest,
  #[serde(rename = "relevance")]
  Relevance,
}

impl Default for SortOrder {
  fn default() -> Self {
    SortOrder::Relevance
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TextHighlight {
  pub start: usize,
  pub end: usize,
  pub matched_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchableMessage {
  pub id: i64,
  pub chat_id: i64,
  pub workspace_id: i64,
  pub sender_id: i64,
  pub sender_name: String,
  pub content: String,
  pub content_highlights: Vec<TextHighlight>,
  pub files: Option<Vec<String>>,
  /// Extracted file names for search indexing
  pub file_names: String,
  pub created_at: DateTime<Utc>,
  pub chat_name: String,
  pub chat_type: String,
  pub relevance_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchResult {
  pub messages: Vec<SearchableMessage>,
  pub pagination: PaginationInfo,
  pub total_hits: usize,
  pub query_time_ms: u64,
  pub search_metadata: SearchMetadata,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationInfo {
  pub offset: i64,
  pub limit: i64,
  pub has_more: bool,
  pub total_pages: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchMetadata {
  pub original_query: String,
  pub search_type: SearchType,
  pub filters_applied: Vec<String>,
  pub indexed_fields: Vec<String>,
  pub facets: Option<SearchFacets>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchFacets {
  pub chat_types: Vec<FacetCount>,
  pub date_histogram: Vec<DateFacet>,
  pub top_senders: Vec<SenderFacet>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FacetCount {
  pub value: String,
  pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DateFacet {
  pub date: DateTime<Utc>,
  pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SenderFacet {
  pub sender_id: i64,
  pub sender_name: String,
  pub count: usize,
}

pub trait MessageRepository: Send + Sync {
  fn create_message(
    &self,
    input: &CreateMessage,
    chat_id: i64,
    user_id: i64,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Message, CoreError>> + Send>>;

  fn list_messages(
    &self,
    input: &ListMessages,
    chat_id: i64,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<Message>, CoreError>> + Send>>;
}
