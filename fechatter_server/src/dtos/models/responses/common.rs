use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 通用 API 响应包装器
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "Operation completed successfully")]
  pub message: Option<String>,

  pub data: Option<T>,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
  pub fn success(data: T) -> Self {
    Self {
      success: true,
      message: None,
      data: Some(data),
      timestamp: chrono::Utc::now(),
    }
  }

  pub fn success_with_message(data: T, message: String) -> Self {
    Self {
      success: true,
      message: Some(message),
      data: Some(data),
      timestamp: chrono::Utc::now(),
    }
  }

  pub fn error(message: String) -> Self {
    Self {
      success: false,
      message: Some(message),
      data: None,
      timestamp: chrono::Utc::now(),
    }
  }
}

/// 分页响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
  pub items: Vec<T>,

  #[schema(example = 100)]
  pub total: i64,

  #[schema(example = 1)]
  pub page: i32,

  #[schema(example = 20)]
  pub per_page: i32,

  #[schema(example = 5)]
  pub total_pages: i32,

  #[schema(example = true)]
  pub has_next: bool,

  #[schema(example = false)]
  pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
  pub fn new(items: Vec<T>, total: i64, page: i32, per_page: i32) -> Self {
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;
    let has_next = page < total_pages;
    let has_prev = page > 1;

    Self {
      items,
      total,
      page,
      per_page,
      total_pages,
      has_next,
      has_prev,
    }
  }
}

/// 操作结果响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OperationResponse {
  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "Operation completed successfully")]
  pub message: String,

  #[schema(example = 1)]
  pub affected_rows: Option<i64>,

  #[schema(example = "op_123456")]
  pub operation_id: Option<String>,
}

/// 错误详情响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
  #[schema(example = "VALIDATION_ERROR")]
  pub error_code: String,

  #[schema(example = "Validation failed")]
  pub error_message: String,

  #[schema(example = "[{\"field\": \"email\", \"message\": \"Invalid email format\"}]")]
  pub details: Option<serde_json::Value>,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub timestamp: chrono::DateTime<chrono::Utc>,

  #[schema(example = "req_123456")]
  pub request_id: Option<String>,
}

/// 统计数据响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatsResponse {
  #[schema(example = "2024-01-01T00:00:00Z")]
  pub start_date: chrono::DateTime<chrono::Utc>,

  #[schema(example = "2024-01-31T23:59:59Z")]
  pub end_date: chrono::DateTime<chrono::Utc>,

  pub metrics: serde_json::Value,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// 健康检查响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
  #[schema(example = "healthy")]
  pub status: String, // "healthy", "degraded", "unhealthy"

  #[schema(example = "1.0.0")]
  pub version: String,

  #[schema(example = 1234567890)]
  pub uptime_seconds: u64,

  pub services: Vec<ServiceHealth>,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceHealth {
  #[schema(example = "database")]
  pub name: String,

  #[schema(example = "healthy")]
  pub status: String,

  #[schema(example = 50)]
  pub response_time_ms: Option<u64>,

  #[schema(example = "Connected successfully")]
  pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadResponse {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub mime_type: String,
    pub size: u64,
    pub created_at: String,
}
