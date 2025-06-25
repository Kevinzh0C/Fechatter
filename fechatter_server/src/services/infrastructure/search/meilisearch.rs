use async_trait::async_trait;
use reqwest::{Client, header::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Import core types from fechatter_core
use fechatter_core::contracts::infrastructure::{
  Document, SearchQuery, SearchResult, SearchService,
};
use fechatter_core::error::CoreError;

use crate::error::AppError;

/// Meilisearch client for full-text search
pub struct MeilisearchClient {
  client: Client,
  host: String,
}

#[cfg_attr(debug_assertions, derive(Debug))] // Only derive Debug in debug builds to avoid log spam
#[derive(Serialize)]
pub struct MeilisearchDocument {
  pub id: String,
  #[serde(flatten)]
  pub fields: Value,
}

#[derive(Debug, Deserialize)]
struct MeilisearchSearchResponse {
  hits: Vec<Value>,
  #[serde(rename = "estimatedTotalHits")]
  estimated_total_hits: u64,
  #[serde(rename = "processingTimeMs")]
  processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
struct MeilisearchSearchRequest {
  q: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  filter: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  limit: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  offset: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "attributesToHighlight")]
  attributes_to_highlight: Option<Vec<String>>,
}

/// Meilisearch error response structure
#[derive(Debug, Deserialize)]
struct MeilisearchErrorResponse {
  message: String,
  code: String,
  #[serde(rename = "type")]
  error_type: String,
}

impl Default for MeilisearchErrorResponse {
  fn default() -> Self {
    Self {
      message: "Unknown error".to_string(),
      code: "unknown_error".to_string(),
      error_type: "unknown".to_string(),
    }
  }
}

impl MeilisearchClient {
  pub fn new(host: String, api_key: Option<String>) -> Result<Self, AppError> {
    // Build client with default headers to avoid repetitive header setting
    let client = {
      let mut headers = HeaderMap::new();
      if let Some(key) = &api_key {
        let auth_value = format!("Bearer {}", key)
          .parse()
          .map_err(|e| AppError::Configuration(format!("Invalid API key format: {}", e)))?;
        headers.insert("Authorization", auth_value);
      }
      Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| AppError::Configuration(format!("Failed to create HTTP client: {}", e)))?
    };

    Ok(Self { client, host })
  }

  /// Parse Meilisearch error response for better error reporting
  async fn parse_error_response(&self, response: reqwest::Response) -> CoreError {
    let status = response.status();
    match response.json::<MeilisearchErrorResponse>().await {
      Ok(error_body) => CoreError::Internal(format!(
        "Meilisearch error ({}): {} [{}]",
        status, error_body.message, error_body.code
      )),
      Err(_) => CoreError::Internal(format!(
        "Meilisearch error ({}): Failed to parse error response",
        status
      )),
    }
  }

  /// Convert filters to Meilisearch filter expression format
  fn build_filter_expression(filters: &Value) -> String {
    match filters {
      Value::Object(map) => {
        let expressions: Vec<String> = map
          .iter()
          .map(|(key, value)| match value {
            Value::String(s) => format!("{} = \"{}\"", key, s),
            Value::Number(n) => format!("{} = {}", key, n),
            Value::Bool(b) => format!("{} = {}", key, b),
            Value::Array(arr) => {
              let values: Vec<String> = arr
                .iter()
                .map(|v| match v {
                  Value::String(s) => format!("\"{}\"", s),
                  Value::Number(n) => n.to_string(),
                  Value::Bool(b) => b.to_string(),
                  _ => "null".to_string(),
                })
                .collect();
              format!("{} IN [{}]", key, values.join(", "))
            }
            _ => format!("{} = null", key),
          })
          .collect();
        expressions.join(" AND ")
      }
      _ => String::new(),
    }
  }

  /// Create index if not exists
  pub async fn create_index(&self, name: &str, primary_key: &str) -> Result<(), CoreError> {
    let url = format!("{}/indexes", self.host);

    let body = serde_json::json!({
        "uid": name,
        "primaryKey": primary_key
    });

    let response = self
      .client
      .post(&url)
      .json(&body)
      .send()
      .await
      .map_err(|e| CoreError::Internal(format!("Meilisearch connection error: {}", e)))?;

    if !response.status().is_success() && response.status().as_u16() != 409 {
      return Err(self.parse_error_response(response).await);
    }

    Ok(())
  }

  /// Configure searchable attributes
  pub async fn configure_index(
    &self,
    index: &str,
    searchable_attributes: Vec<&str>,
  ) -> Result<(), CoreError> {
    let url = format!(
      "{}/indexes/{}/settings/searchable-attributes",
      self.host, index
    );

    let response = self
      .client
      .put(&url)
      .json(&searchable_attributes)
      .send()
      .await
      .map_err(|e| CoreError::Internal(format!("Meilisearch connection error: {}", e)))?;

    if !response.status().is_success() {
      return Err(self.parse_error_response(response).await);
    }

    Ok(())
  }
}

#[async_trait]
impl SearchService for MeilisearchClient {
  async fn index_document(&self, index: &str, doc: Document) -> Result<(), CoreError> {
    let url = format!("{}/indexes/{}/documents", self.host, index);

    let meilisearch_doc = MeilisearchDocument {
      id: doc.id,
      fields: doc.fields,
    };

    let response = self
      .client
      .post(&url)
      .json(&[meilisearch_doc])
      .send()
      .await
      .map_err(|e| CoreError::Internal(format!("Meilisearch connection error: {}", e)))?;

    if !response.status().is_success() {
      return Err(self.parse_error_response(response).await);
    }

    Ok(())
  }

  async fn search(&self, index: &str, query: SearchQuery) -> Result<SearchResult, CoreError> {
    let url = format!("{}/indexes/{}/search", self.host, index);

    let search_request = MeilisearchSearchRequest {
      q: query.query,
      // Convert filters to proper Meilisearch filter expression format
      filter: query.filters.map(|f| Self::build_filter_expression(&f)),
      limit: Some(query.limit),
      offset: Some(query.offset),
      // Only send highlighting when needed to save bandwidth
      attributes_to_highlight: None, // TODO: Add to SearchQuery if highlighting is needed
    };

    let response = self
      .client
      .post(&url)
      .json(&search_request)
      .send()
      .await
      .map_err(|e| CoreError::Internal(format!("Meilisearch connection error: {}", e)))?;

    if !response.status().is_success() {
      return Err(self.parse_error_response(response).await);
    }

    let search_response: MeilisearchSearchResponse = response
      .json()
      .await
      .map_err(|e| CoreError::Internal(format!("Failed to parse search response: {}", e)))?;

    let hits = search_response
      .hits
      .into_iter()
      .map(|hit| Document {
        id: hit["id"].as_str().unwrap_or("").to_string(),
        fields: hit,
      })
      .collect();

    Ok(SearchResult {
      hits,
      total: search_response.estimated_total_hits,
      took_ms: search_response.processing_time_ms,
    })
  }

  async fn delete_document(&self, index: &str, id: &str) -> Result<(), CoreError> {
    let url = format!("{}/indexes/{}/documents/{}", self.host, index, id);

    let response = self
      .client
      .delete(&url)
      .send()
      .await
      .map_err(|e| CoreError::Internal(format!("Meilisearch connection error: {}", e)))?;

    if !response.status().is_success() {
      return Err(self.parse_error_response(response).await);
    }

    Ok(())
  }

  async fn update_document(&self, index: &str, id: &str, doc: Document) -> Result<(), CoreError> {
    // Meilisearch treats updates as upserts
    self.index_document(index, doc).await
  }

  fn as_any(&self) -> &dyn std::any::Any {
    self
  }
}

/// Message-specific search implementation
impl MeilisearchClient {
  /// Index a message for full-text search
  pub async fn index_message(
    &self,
    message_id: i64,
    chat_id: i64,
    sender_id: i64,
    content: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
  ) -> Result<(), CoreError> {
    let doc = Document {
      id: message_id.to_string(),
      fields: serde_json::json!({
          "message_id": message_id,
          "chat_id": chat_id,
          "sender_id": sender_id,
          "content": content,
          "timestamp": timestamp.to_rfc3339(),
      }),
    };

    self.index_document("messages", doc).await
  }

  /// Search messages with filters
  pub async fn search_messages(
    &self,
    query: &str,
    chat_id: Option<i64>,
    limit: u32,
  ) -> Result<Vec<(i64, String, f32)>, CoreError> {
    let filters = chat_id.map(|id| {
      serde_json::json!({
          "chat_id": id
      })
    });

    let search_query = SearchQuery {
      query: query.to_string(),
      filters,
      limit,
      offset: 0,
    };

    let results = self.search("messages", search_query).await?;

    Ok(
      results
        .hits
        .into_iter()
        .map(|doc| {
          let message_id = doc.fields["message_id"].as_i64().unwrap_or(0);
          let content = doc.fields["content"].as_str().unwrap_or("").to_string();
          // Use updated score field name for Meilisearch 1.4+
          let score = doc.fields["_score"].as_f64().unwrap_or_default() as f32;
          (message_id, content, score)
        })
        .collect(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_build_filter_expression_simple() {
    let filters = json!({
      "chat_id": 123,
      "status": "active"
    });

    let expression = MeilisearchClient::build_filter_expression(&filters);

    // Should contain both conditions with AND
    assert!(expression.contains("chat_id = 123"));
    assert!(expression.contains("status = \"active\""));
    assert!(expression.contains(" AND "));
  }

  #[test]
  fn test_build_filter_expression_array() {
    let filters = json!({
      "user_id": [1, 2, 3],
      "category": ["work", "personal"]
    });

    let expression = MeilisearchClient::build_filter_expression(&filters);

    // Should use IN syntax for arrays
    assert!(expression.contains("user_id IN [1, 2, 3]"));
    assert!(expression.contains("category IN [\"work\", \"personal\"]"));
  }

  #[test]
  fn test_build_filter_expression_boolean() {
    let filters = json!({
      "is_public": true,
      "is_deleted": false
    });

    let expression = MeilisearchClient::build_filter_expression(&filters);

    assert!(expression.contains("is_public = true"));
    assert!(expression.contains("is_deleted = false"));
  }

  #[test]
  fn test_build_filter_expression_null() {
    let filters = json!({
      "deleted_at": null
    });

    let expression = MeilisearchClient::build_filter_expression(&filters);

    assert!(expression.contains("deleted_at = null"));
  }

  #[test]
  fn test_build_filter_expression_empty() {
    let filters = json!({});

    let expression = MeilisearchClient::build_filter_expression(&filters);

    assert!(expression.is_empty());
  }

  #[test]
  fn test_build_filter_expression_non_object() {
    let filters = json!("not an object");

    let expression = MeilisearchClient::build_filter_expression(&filters);

    assert!(expression.is_empty());
  }

  #[test]
  fn test_meilisearch_error_response_default() {
    let default_error = MeilisearchErrorResponse::default();

    assert_eq!(default_error.message, "Unknown error");
    assert_eq!(default_error.code, "unknown_error");
    assert_eq!(default_error.error_type, "unknown");
  }

  #[test]
  fn test_meilisearch_client_new_with_api_key() {
    let result = MeilisearchClient::new(
      "http://localhost:7700".to_string(),
      Some("test_key".to_string()),
    );

    assert!(result.is_ok());
    let client = result.unwrap();
    assert_eq!(client.host, "http://localhost:7700");
  }

  #[test]
  fn test_meilisearch_client_new_without_api_key() {
    let result = MeilisearchClient::new("http://localhost:7700".to_string(), None);

    assert!(result.is_ok());
    let client = result.unwrap();
    assert_eq!(client.host, "http://localhost:7700");
  }

  #[cfg(debug_assertions)]
  #[test]
  fn test_debug_derive_in_debug_mode() {
    let doc = MeilisearchDocument {
      id: "test".to_string(),
      fields: json!({"content": "test content"}),
    };

    // This should compile and work in debug mode
    let debug_str = format!("{:?}", doc);
    assert!(debug_str.contains("test"));
  }

  #[test]
  fn test_search_request_serialization() {
    let request = MeilisearchSearchRequest {
      q: "test query".to_string(),
      filter: Some("chat_id = 123".to_string()),
      limit: Some(10),
      offset: Some(0),
      attributes_to_highlight: None, // Should be omitted from JSON
    };

    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("\"q\":\"test query\""));
    assert!(json.contains("\"filter\":\"chat_id = 123\""));
    assert!(json.contains("\"limit\":10"));
    assert!(!json.contains("attributesToHighlight")); // Should be omitted
  }

  #[test]
  fn test_search_request_with_highlighting() {
    let request = MeilisearchSearchRequest {
      q: "test".to_string(),
      filter: None,
      limit: None,
      offset: None,
      attributes_to_highlight: Some(vec!["content".to_string()]),
    };

    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("\"attributesToHighlight\":[\"content\"]"));
  }

  #[test]
  fn test_complex_filter_expression() {
    let filters = json!({
      "chat_id": 42,
      "sender_id": [1, 2, 3],
      "is_public": true,
      "content": "hello world",
      "created_at": null
    });

    let expression = MeilisearchClient::build_filter_expression(&filters);

    // Verify all filter types are handled correctly
    assert!(expression.contains("chat_id = 42"));
    assert!(expression.contains("sender_id IN [1, 2, 3]"));
    assert!(expression.contains("is_public = true"));
    assert!(expression.contains("content = \"hello world\""));
    assert!(expression.contains("created_at = null"));

    // Should have multiple AND connections
    let and_count = expression.matches(" AND ").count();
    assert_eq!(and_count, 4); // 5 conditions - 1 = 4 ANDs
  }
}
