// 统一响应格式框架
//
// 定义了所有API响应的标准格式，确保API的一致性
// 支持成功响应、错误响应、批量操作响应等多种场景

use chrono::{DateTime, Utc};
use fechatter_core::error::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 统一API响应格式 - 所有API的标准返回结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
  /// 响应状态
  pub success: bool,

  /// 响应数据
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<T>,

  /// 错误信息
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<ApiError>,

  /// 响应元信息
  pub meta: ResponseMeta,

  /// 扩展信息
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub extensions: HashMap<String, serde_json::Value>,
}

/// 响应元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
  /// 请求ID
  pub request_id: String,

  /// 响应时间戳
  pub timestamp: DateTime<Utc>,

  /// API版本
  pub version: String,

  /// 响应时间(毫秒)
  pub duration_ms: u64,

  /// 服务器信息
  #[serde(skip_serializing_if = "Option::is_none")]
  pub server_info: Option<ServerInfo>,
}

/// API错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
  /// 错误代码
  pub code: String,

  /// 错误消息
  pub message: String,

  /// 详细描述
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<String>,

  /// 错误字段(用于验证错误)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub field: Option<String>,

  /// 错误堆栈
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub stack: Vec<ErrorFrame>,

  /// 建议的解决方案
  #[serde(skip_serializing_if = "Option::is_none")]
  pub suggestion: Option<String>,

  /// 帮助链接
  #[serde(skip_serializing_if = "Option::is_none")]
  pub help_url: Option<String>,
}

/// 错误堆栈帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorFrame {
  /// 错误位置
  pub location: String,

  /// 错误类型
  pub error_type: String,

  /// 附加信息
  pub context: HashMap<String, String>,
}

/// 服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
  /// 服务器节点ID
  pub node_id: String,

  /// 服务器区域
  pub region: String,

  /// 环境信息
  pub environment: String,
}

/// 批量操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse<T> {
  /// 成功的项目
  pub succeeded: Vec<BatchItem<T>>,

  /// 失败的项目
  pub failed: Vec<BatchError>,

  /// 批量操作统计
  pub stats: BatchStats,
}

/// 批量操作项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem<T> {
  /// 项目索引
  pub index: usize,

  /// 项目ID(如果有)
  pub id: Option<String>,

  /// 项目数据
  pub data: T,
}

/// 批量操作错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
  /// 项目索引
  pub index: usize,

  /// 项目ID(如果有)
  pub id: Option<String>,

  /// 错误信息
  pub error: ApiError,
}

/// 批量操作统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStats {
  /// 总数量
  pub total: usize,

  /// 成功数量
  pub succeeded: usize,

  /// 失败数量
  pub failed: usize,

  /// 成功率
  pub success_rate: f32,

  /// 处理时间(毫秒)
  pub processing_time_ms: u64,
}

/// 操作结果响应 - 用于增删改操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse<T> {
  /// 操作类型
  pub operation: OperationType,

  /// 操作结果
  pub result: T,

  /// 受影响的记录数
  pub affected_count: u64,

  /// 操作警告
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub warnings: Vec<String>,

  /// 相关资源链接
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub links: HashMap<String, String>,
}

/// 操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
  Create,
  Update,
  Delete,
  Patch,
  Upsert,
  BulkCreate,
  BulkUpdate,
  BulkDelete,
}

// 响应构造器
impl<T> ApiResponse<T> {
  /// 创建成功响应
  pub fn success(data: T, request_id: String) -> Self {
    Self {
      success: true,
      data: Some(data),
      error: None,
      meta: ResponseMeta::new(request_id),
      extensions: HashMap::new(),
    }
  }

  /// 创建简单成功响应（for backward compatibility）
  pub fn ok(data: T) -> Self {
    Self::success(data, uuid::Uuid::new_v4().to_string())
  }

  /// 创建错误响应
  pub fn error(error: ApiError, request_id: String) -> Self {
    Self {
      success: false,
      data: None,
      error: Some(error),
      meta: ResponseMeta::new(request_id),
      extensions: HashMap::new(),
    }
  }

  /// 添加扩展信息
  pub fn with_extension(mut self, key: String, value: serde_json::Value) -> Self {
    self.extensions.insert(key, value);
    self
  }

  /// 设置处理时间
  pub fn with_duration(mut self, duration_ms: u64) -> Self {
    self.meta.duration_ms = duration_ms;
    self
  }
}

impl ResponseMeta {
  /// 创建新的响应元信息
  pub fn new(request_id: String) -> Self {
    Self {
      request_id,
      timestamp: Utc::now(),
      version: "v1".to_string(),
      duration_ms: 0,
      server_info: None,
    }
  }

  /// 添加服务器信息
  pub fn with_server_info(mut self, server_info: ServerInfo) -> Self {
    self.server_info = Some(server_info);
    self
  }
}

impl From<CoreError> for ApiError {
  fn from(error: CoreError) -> Self {
    match error {
      CoreError::Validation(msg) => Self {
        code: "VALIDATION_ERROR".to_string(),
        message: msg,
        details: None,
        field: None,
        stack: Vec::new(),
        suggestion: Some("请检查输入参数是否符合要求".to_string()),
        help_url: Some("/docs/validation".to_string()),
      },
      CoreError::NotFound(msg) => Self {
        code: "NOT_FOUND".to_string(),
        message: msg,
        details: None,
        field: None,
        stack: Vec::new(),
        suggestion: Some("请确认资源ID是否正确".to_string()),
        help_url: None,
      },
      CoreError::Database(msg) => Self {
        code: "DATABASE_ERROR".to_string(),
        message: "数据库操作失败".to_string(),
        details: Some(msg),
        field: None,
        stack: Vec::new(),
        suggestion: Some("请稍后重试，如问题持续请联系技术支持".to_string()),
        help_url: None,
      },
      // Handle all other CoreError variants with a generic conversion
      _ => Self {
        code: "INTERNAL_ERROR".to_string(),
        message: format!("系统内部错误: {:?}", error),
        details: None,
        field: None,
        stack: Vec::new(),
        suggestion: Some("请稍后重试，如问题持续请联系技术支持".to_string()),
        help_url: None,
      },
    }
  }
}

impl<T> BatchResponse<T> {
  /// 创建新的批量响应
  pub fn new() -> Self {
    Self {
      succeeded: Vec::new(),
      failed: Vec::new(),
      stats: BatchStats {
        total: 0,
        succeeded: 0,
        failed: 0,
        success_rate: 0.0,
        processing_time_ms: 0,
      },
    }
  }

  /// 添加成功项目
  pub fn add_success(&mut self, index: usize, id: Option<String>, data: T) {
    self.succeeded.push(BatchItem { index, id, data });
    self.update_stats();
  }

  /// 添加失败项目
  pub fn add_failure(&mut self, index: usize, id: Option<String>, error: ApiError) {
    self.failed.push(BatchError { index, id, error });
    self.update_stats();
  }

  /// 更新统计信息
  fn update_stats(&mut self) {
    self.stats.succeeded = self.succeeded.len();
    self.stats.failed = self.failed.len();
    self.stats.total = self.stats.succeeded + self.stats.failed;
    self.stats.success_rate = if self.stats.total > 0 {
      (self.stats.succeeded as f32) / (self.stats.total as f32) * 100.0
    } else {
      0.0
    };
  }
}

/// 响应类型枚举 - 用于类型安全的响应处理
pub type SuccessResponse<T> = ApiResponse<T>;
pub type ErrorResponse = ApiResponse<()>;
pub type ListResponse<T> = ApiResponse<super::PaginatedResponse<T>>;
pub type CreateResponse<T> = ApiResponse<OperationResponse<T>>;
pub type UpdateResponse<T> = ApiResponse<OperationResponse<T>>;
pub type DeleteResponse = ApiResponse<OperationResponse<()>>;
pub type BatchCreateResponse<T> = ApiResponse<BatchResponse<T>>;
pub type BatchUpdateResponse<T> = ApiResponse<BatchResponse<T>>;
pub type BatchDeleteResponse = ApiResponse<BatchResponse<()>>;
