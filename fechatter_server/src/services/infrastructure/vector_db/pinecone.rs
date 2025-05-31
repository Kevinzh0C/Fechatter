use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use reqwest::{
  Client, StatusCode,
  header::{CONTENT_TYPE, HeaderMap},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use tokio_stream::StreamExt as TokioStreamExt;

use crate::error::AppError;
use fechatter_core::{
  error::{CoreError, VectorDbError},
  models::time_management::TimeManager,
  models::vector_db::{MetadataFilter, VectorDatabase, VectorSearchResult},
  models::{ChatId, MessageId, UserId},
};

/// Default namespace for all Pinecone operations
const DEFAULT_NAMESPACE: &str = "fechatter";

/// Pinecone API header constants
const API_KEY_HDR: &str = "Api-Key";
const API_VERSION_HDR: &str = "X-Pinecone-API-Version";
const API_VERSION: &str = "2025-01";

/// Metadata field constants
const META_CREATED: &str = "app_created_at";
const META_UPDATED: &str = "app_updated_at";

/// Pinecone vector database client
pub struct PineconeClient {
  client: Client,
  api_key: String,
  environment: String,
  index_name: String,
  project_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UpsertRequest {
  vectors: Vec<Vector>,
  namespace: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Vector {
  id: String,
  values: Vec<f32>,
  metadata: Value,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct QueryRequest {
  vector: Vec<f32>,
  top_k: usize,
  filter: Option<Value>,
  include_values: bool,
  include_metadata: bool,
  namespace: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct QueryResponse {
  matches: Vec<Match>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Match {
  id: String,
  score: f32,
  values: Option<Vec<f32>>,
  metadata: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UpdateRequest {
  id: String,
  set_metadata: Value,
  namespace: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DeleteRequest {
  ids: Option<Vec<String>>,
  filter: Option<Value>,
  namespace: Option<String>,
}

impl PineconeClient {
  pub fn new(
    api_key: String,
    environment: String,
    index_name: String,
    project_id: String,
  ) -> Result<Self, VectorDbError> {
    let mut default_headers = HeaderMap::new();

    default_headers.insert(
      API_KEY_HDR,
      api_key
        .parse()
        .map_err(|e| VectorDbError::Validation(format!("Invalid API key format: {}", e)))?,
    );

    default_headers.insert(
      API_VERSION_HDR,
      API_VERSION
        .parse()
        .map_err(|e| VectorDbError::Validation(format!("Invalid API version: {}", e)))?,
    );

    default_headers.insert(
      CONTENT_TYPE,
      "application/json"
        .parse()
        .map_err(|e| VectorDbError::Validation(format!("Invalid content type: {}", e)))?,
    );

    let client = Client::builder()
      .default_headers(default_headers)
      .build()
      .map_err(|e| VectorDbError::Permanent(format!("Failed to create HTTP client: {}", e)))?;

    Ok(Self {
      client,
      api_key,
      environment,
      index_name,
      project_id,
    })
  }

  fn get_index_url(&self) -> String {
    format!(
      "https://{}-{}.svc.{}.pinecone.io",
      &self.index_name, &self.project_id, &self.environment
    )
  }

  fn ep(&self, path: &str) -> String {
    format!("{}/{}", self.get_index_url(), path)
  }

  fn map_status_error(&self, status: StatusCode, error_text: String) -> VectorDbError {
    match status {
      StatusCode::BAD_REQUEST => VectorDbError::Validation(error_text),
      StatusCode::UNAUTHORIZED => {
        VectorDbError::Permanent(format!("Authentication failed: {}", error_text))
      }
      StatusCode::FORBIDDEN => VectorDbError::Permanent(format!("Access denied: {}", error_text)),
      StatusCode::NOT_FOUND => VectorDbError::NotFound(error_text),
      StatusCode::TOO_MANY_REQUESTS => {
        VectorDbError::Transient(format!("Rate limited: {}", error_text))
      }
      s if s.is_server_error() => VectorDbError::Transient(format!("Server error: {}", error_text)),
      _ => VectorDbError::Permanent(format!("Unexpected error {}: {}", status, error_text)),
    }
  }
}

impl From<AppError> for VectorDbError {
  fn from(err: AppError) -> Self {
    match err {
      AppError::ExternalService(msg) => VectorDbError::Transient(msg),
      AppError::NotFound(msg) => VectorDbError::NotFound(msg.join(", ")),
      AppError::InvalidInput(msg) => VectorDbError::Validation(msg),
      AppError::ServerError(msg) => VectorDbError::Permanent(msg),
      _ => VectorDbError::Permanent("Unknown error".to_string()),
    }
  }
}

#[async_trait]
impl VectorDatabase for PineconeClient {
  async fn insert(
    &self,
    id: &str,
    vector: &[f32],
    metadata: Value,
    _created_at: DateTime<Utc>,
  ) -> Result<(), CoreError> {
    let mut enhanced_metadata = metadata;
    if let Value::Object(ref mut map) = enhanced_metadata {
      map.insert(
        META_CREATED.to_string(),
        Value::String(TimeManager::format_iso(TimeManager::now())),
      );
    }

    let request = UpsertRequest {
      vectors: vec![Vector {
        id: id.to_string(),
        values: vector.to_vec(),
        metadata: enhanced_metadata,
      }],
      namespace: Some(DEFAULT_NAMESPACE.to_string()),
    };

    let response = self
      .client
      .post(self.ep("vectors/upsert"))
      .json(&request)
      .send()
      .await
      .map_err(|e| {
        CoreError::VectorDbError(VectorDbError::Transient(format!(
          "Pinecone connection error: {}",
          e
        )))
      })?;

    let status = response.status();
    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(CoreError::VectorDbError(
        self.map_status_error(status, error_text),
      ));
    }

    Ok(())
  }

  async fn upsert(
    &self,
    id: &str,
    vector: &[f32],
    metadata: Value,
    created_at: DateTime<Utc>,
  ) -> Result<(), CoreError> {
    // Pinecone upsert is the same as insert
    self.insert(id, vector, metadata, created_at).await
  }

  async fn batch_insert(
    &self,
    items: &[(&str, Vec<f32>, Value, DateTime<Utc>)],
  ) -> Result<(), CoreError> {
    let app_timestamp = TimeManager::format_iso(TimeManager::now());

    let vectors: Vec<Vector> = items
      .iter()
      .map(|(id, values, metadata, _)| {
        let mut enhanced_metadata = (*metadata).clone();
        if let Value::Object(ref mut map) = enhanced_metadata {
          map.insert(
            META_CREATED.to_string(),
            Value::String(app_timestamp.clone()),
          );
        }

        Vector {
          id: (*id).to_string(),
          values: values.clone(),
          metadata: enhanced_metadata,
        }
      })
      .collect();

    // Pinecone has a limit of 100 vectors per request
    // Process chunks concurrently using tokio::spawn
    let chunks: Vec<_> = vectors.chunks(100).collect();
    let mut handles = Vec::new();

    for chunk in chunks {
      let request = UpsertRequest {
        vectors: chunk.to_vec(),
        namespace: Some(DEFAULT_NAMESPACE.to_string()),
      };
      let client = self.client.clone();
      let endpoint = self.ep("vectors/upsert");
      let error_mapper = self.clone();

      let handle = tokio::spawn(async move {
        let response = client
          .post(endpoint)
          .json(&request)
          .send()
          .await
          .map_err(|e| {
            CoreError::VectorDbError(VectorDbError::Transient(format!(
              "Pinecone connection error: {}",
              e
            )))
          })?;

        let status = response.status();
        if !status.is_success() {
          let error_text = response.text().await.unwrap_or_default();
          return Err(CoreError::VectorDbError(
            error_mapper.map_status_error(status, error_text),
          ));
        }

        Ok(())
      });

      handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
      match handle.await {
        Ok(result) => result?,
        Err(e) => {
          return Err(CoreError::VectorDbError(VectorDbError::Permanent(format!(
            "Task join error: {}",
            e
          ))));
        }
      }
    }

    Ok(())
  }

  async fn search(
    &self,
    query_vector: &[f32],
    limit: usize,
    filter: Option<MetadataFilter>,
  ) -> Result<Vec<VectorSearchResult>, CoreError> {
    let filter_value = filter.as_ref().map(|f| {
      let filter_params = f.to_query_params();
      let filter_map: serde_json::Map<String, Value> = filter_params
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();
      Value::Object(filter_map)
    });

    let request = QueryRequest {
      vector: query_vector.to_vec(),
      top_k: limit,
      filter: filter_value,
      include_values: false,
      include_metadata: true,
      namespace: Some(DEFAULT_NAMESPACE.to_string()),
    };

    let response = self
      .client
      .post(self.ep("query"))
      .json(&request)
      .send()
      .await
      .map_err(|e| {
        CoreError::VectorDbError(VectorDbError::Transient(format!(
          "Pinecone connection error: {}",
          e
        )))
      })?;

    let status = response.status();
    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(CoreError::VectorDbError(
        self.map_status_error(status, error_text),
      ));
    }

    let query_response: QueryResponse = response.json().await.map_err(|e| {
      CoreError::VectorDbError(VectorDbError::Permanent(format!(
        "Failed to parse response: {}",
        e
      )))
    })?;

    Ok(
      query_response
        .matches
        .into_iter()
        .map(|m| VectorSearchResult {
          id: m.id,
          score: m.score,
          metadata: None,
          payload: m.metadata.unwrap_or(Value::Null),
        })
        .collect(),
    )
  }

  async fn update_metadata(&self, id: &str, metadata: Value) -> Result<(), CoreError> {
    // Check if metadata is null or empty object
    if metadata.is_null() {
      return Err(CoreError::VectorDbError(VectorDbError::Validation(
        "Metadata cannot be null".to_string(),
      )));
    }

    if let Value::Object(ref map) = metadata {
      if map.is_empty() {
        return Err(CoreError::VectorDbError(VectorDbError::Validation(
          "Metadata cannot be empty".to_string(),
        )));
      }
    }

    let mut enhanced_metadata = metadata;
    if let Value::Object(ref mut map) = enhanced_metadata {
      map.insert(
        META_UPDATED.to_string(),
        Value::String(TimeManager::format_iso(TimeManager::now())),
      );
    }

    let request = UpdateRequest {
      id: id.to_string(),
      set_metadata: enhanced_metadata,
      namespace: DEFAULT_NAMESPACE.to_string(),
    };

    let response = self
      .client
      .post(self.ep("vectors/update"))
      .json(&request)
      .send()
      .await
      .map_err(|e| {
        CoreError::VectorDbError(VectorDbError::Transient(format!(
          "Pinecone connection error: {}",
          e
        )))
      })?;

    let status = response.status();
    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(CoreError::VectorDbError(
        self.map_status_error(status, error_text),
      ));
    }

    Ok(())
  }

  async fn delete(&self, id: &str) -> Result<(), CoreError> {
    let request = DeleteRequest {
      ids: Some(vec![id.to_string()]),
      filter: None,
      namespace: Some(DEFAULT_NAMESPACE.to_string()),
    };

    let response = self
      .client
      .post(self.ep("vectors/delete"))
      .json(&request)
      .send()
      .await
      .map_err(|e| {
        CoreError::VectorDbError(VectorDbError::Transient(format!(
          "Pinecone connection error: {}",
          e
        )))
      })?;

    let status = response.status();
    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(CoreError::VectorDbError(
        self.map_status_error(status, error_text),
      ));
    }

    Ok(())
  }

  async fn delete_by_filter(&self, filter: MetadataFilter) -> Result<usize, CoreError> {
    // Check if filter is empty - equivalent to is_empty() method
    if filter.conditions.is_empty()
      && filter.chat_id.is_none()
      && filter.sender_id.is_none()
      && filter.time_range.is_none()
    {
      return Err(CoreError::VectorDbError(VectorDbError::Validation(
        "Cannot delete with empty filter - would delete all vectors".to_string(),
      )));
    }

    let filter_params = filter.to_query_params();
    let filter_map: serde_json::Map<String, Value> = filter_params
      .into_iter()
      .map(|(k, v)| (k, Value::String(v)))
      .collect();
    let filter_obj = Value::Object(filter_map);

    let request = DeleteRequest {
      ids: None,
      filter: Some(filter_obj),
      namespace: Some(DEFAULT_NAMESPACE.to_string()),
    };

    let response = self
      .client
      .post(self.ep("vectors/delete"))
      .json(&request)
      .send()
      .await
      .map_err(|e| {
        CoreError::VectorDbError(VectorDbError::Transient(format!(
          "Pinecone connection error: {}",
          e
        )))
      })?;

    let status = response.status();
    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(CoreError::VectorDbError(
        self.map_status_error(status, error_text),
      ));
    }

    // Pinecone doesn't return the count of deleted vectors
    // Return 1 to indicate success
    Ok(1)
  }

  async fn get(&self, id: &str) -> Result<Option<(Vec<f32>, Value)>, CoreError> {
    // Pinecone doesn't have a direct "get by ID" endpoint
    // We need to use the fetch endpoint
    let url = format!(
      "{}?ids={}&namespace={}",
      self.ep("vectors/fetch"),
      id,
      DEFAULT_NAMESPACE
    );

    let response = self.client.get(&url).send().await.map_err(|e| {
      CoreError::VectorDbError(VectorDbError::Transient(format!(
        "Pinecone connection error: {}",
        e
      )))
    })?;

    let status = response.status();
    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(CoreError::VectorDbError(
        self.map_status_error(status, error_text),
      ));
    }

    let body: Value = response.json().await.map_err(|e| {
      CoreError::VectorDbError(VectorDbError::Permanent(format!(
        "Failed to parse response: {}",
        e
      )))
    })?;

    let v = body["vectors"][id]
      .as_object()
      .ok_or(CoreError::VectorDbError(VectorDbError::NotFound(format!(
        "Vector {} not found",
        id
      ))))?;

    let values = v["values"]
      .as_array()
      .ok_or(CoreError::VectorDbError(VectorDbError::Permanent(
        "Missing vector values in response".to_string(),
      )))?
      .iter()
      .filter_map(|v| v.as_f64().map(|f| f as f32))
      .collect();

    let meta = v["metadata"].clone();

    Ok(Some((values, meta)))
  }
}

// Implement Clone for PineconeClient to support FuturesUnordered
impl Clone for PineconeClient {
  fn clone(&self) -> Self {
    Self {
      client: self.client.clone(),
      api_key: self.api_key.clone(),
      environment: self.environment.clone(),
      index_name: self.index_name.clone(),
      project_id: self.project_id.clone(),
    }
  }
}
