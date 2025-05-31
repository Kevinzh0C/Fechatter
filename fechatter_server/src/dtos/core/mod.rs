// DTOs Core Framework - 统一的数据传输对象框架
//
// 这个模块定义了整个DTOs层的核心抽象和通用功能
// 从Clean Architecture角度，DTOs属于Interface Adapters层
// 负责外部世界(HTTP API)和内部领域(Domain)之间的适配

pub mod conversion;
pub mod pagination;
pub mod response;
pub mod validation;

// 重新导出核心类型
pub use conversion::*;
pub use pagination::*;
pub use response::*;
pub use validation::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 基础DTO特征 - 所有DTO都应实现的核心能力
pub trait BaseDto: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync {
  /// DTO的唯一标识符，用于缓存和追踪
  fn dto_type() -> &'static str;

  /// 验证DTO数据的完整性和业务规则
  fn validate(&self) -> Result<(), DtoValidationError>;

  /// 获取DTO的元数据信息
  fn metadata(&self) -> DtoMetadata {
    DtoMetadata::default()
  }
}

/// 请求DTO特征 - 所有请求DTO的统一接口
pub trait RequestDto: BaseDto {
  type DomainModel;

  /// 转换为领域模型，包含完整的错误处理
  fn to_domain(&self) -> Result<Self::DomainModel, ConversionError>;

  /// 预处理请求数据（如数据清洗、格式化等）
  fn preprocess(&mut self) -> Result<(), DtoValidationError> {
    Ok(())
  }

  /// 获取请求的业务上下文信息
  fn business_context(&self) -> BusinessContext {
    BusinessContext::default()
  }
}

/// 响应DTO特征 - 所有响应DTO的统一接口  
pub trait ResponseDto: BaseDto {
  type DomainModel;

  /// 从领域模型创建响应DTO
  fn from_domain(domain: &Self::DomainModel) -> Result<Self, ConversionError>;

  /// 从领域模型集合创建响应DTO集合
  fn from_domain_collection(domains: &[Self::DomainModel]) -> Result<Vec<Self>, ConversionError> {
    domains.iter().map(Self::from_domain).collect()
  }

  /// 应用响应过滤器（如字段选择、数据脱敏）
  fn apply_filters(&mut self, filters: &ResponseFilters) -> Result<(), ConversionError> {
    Ok(())
  }
}

/// DTO元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtoMetadata {
  pub version: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub tags: HashMap<String, String>,
}

impl Default for DtoMetadata {
  fn default() -> Self {
    Self {
      version: "1.0".to_string(),
      created_at: chrono::Utc::now(),
      tags: HashMap::new(),
    }
  }
}

/// 业务上下文
#[derive(Debug, Clone, Default)]
pub struct BusinessContext {
  pub user_id: Option<i64>,
  pub workspace_id: Option<i64>,
  pub operation_type: Option<String>,
  pub correlation_id: Option<String>,
}

/// 响应过滤器
#[derive(Debug, Clone, Default)]
pub struct ResponseFilters {
  pub include_fields: Option<Vec<String>>,
  pub exclude_fields: Option<Vec<String>>,
  pub sensitive_data_policy: SensitiveDataPolicy,
}

/// 敏感数据处理策略
#[derive(Debug, Clone)]
pub enum SensitiveDataPolicy {
  ShowAll,       // 显示所有数据
  MaskSensitive, // 脱敏敏感数据
  HideSensitive, // 隐藏敏感数据
}

impl Default for SensitiveDataPolicy {
  fn default() -> Self {
    Self::MaskSensitive
  }
}
