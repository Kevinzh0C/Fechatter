// 转换错误和转换工具
//
// 处理DTOs与Domain模型之间的转换问题
// 提供类型安全的转换、错误处理、批量转换等功能

use fechatter_core::error::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 转换错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionError {
  /// 错误类型
  pub error_type: ConversionErrorType,

  /// 错误消息
  pub message: String,

  /// 源类型
  pub source_type: String,

  /// 目标类型
  pub target_type: String,

  /// 失败的字段
  pub failed_field: Option<String>,

  /// 原始值
  pub original_value: Option<String>,

  /// 错误详情
  pub details: Option<String>,

  /// 转换上下文
  pub context: ConversionContext,
}

/// 转换错误类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversionErrorType {
  /// 字段缺失
  MissingField,

  /// 类型不匹配
  TypeMismatch,

  /// 值超出范围
  ValueOutOfRange,

  /// 格式错误
  InvalidFormat,

  /// 业务规则冲突
  BusinessRuleViolation,

  /// 依赖缺失
  MissingDependency,

  /// 循环引用
  CircularReference,

  /// 数据完整性错误
  DataIntegrityError,

  /// 权限不足
  InsufficientPermissions,

  /// 未知错误
  Unknown,
}

/// 转换上下文
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversionContext {
  /// 转换路径
  pub path: Vec<String>,

  /// 操作类型
  pub operation: Option<String>,

  /// 用户ID
  pub user_id: Option<i64>,

  /// 工作空间ID
  pub workspace_id: Option<i64>,

  /// 时间戳
  pub timestamp: chrono::DateTime<chrono::Utc>,

  /// 扩展信息
  pub metadata: HashMap<String, String>,
}

/// 转换器特征 - 定义类型安全的双向转换
pub trait Converter<From, To>: Send + Sync {
  /// 正向转换
  fn convert(&self, from: &From, context: &ConversionContext) -> Result<To, ConversionError>;

  /// 反向转换(如果支持)
  fn convert_back(&self, to: &To, context: &ConversionContext) -> Result<From, ConversionError> {
    Err(ConversionError::new(
      ConversionErrorType::Unknown,
      "反向转换未实现".to_string(),
      std::any::type_name::<To>().to_string(),
      std::any::type_name::<From>().to_string(),
    ))
  }

  /// 转换器名称
  fn name(&self) -> &'static str;

  /// 是否支持反向转换
  fn supports_reverse(&self) -> bool {
    false
  }
}

/// 批量转换器
pub struct BatchConverter<From, To> {
  converter: Box<dyn Converter<From, To>>,
  error_strategy: BatchErrorStrategy,
}

/// 批量转换策略
#[derive(Debug, Clone)]
pub enum BatchErrorStrategy {
  /// 遇到错误立即停止
  FailFast,

  /// 收集所有错误，返回成功的项目和错误列表
  CollectErrors,

  /// 跳过错误项目，只返回成功的项目
  SkipErrors,
}

/// 批量转换结果
#[derive(Debug, Clone)]
pub struct BatchConversionResult<T> {
  /// 成功转换的项目
  pub successful: Vec<BatchConversionItem<T>>,

  /// 失败的项目
  pub failed: Vec<BatchConversionError>,

  /// 统计信息
  pub stats: BatchConversionStats,
}

/// 批量转换项目
#[derive(Debug, Clone)]
pub struct BatchConversionItem<T> {
  /// 原始索引
  pub index: usize,

  /// 转换结果
  pub item: T,

  /// 转换时间(微秒)
  pub conversion_time_us: u64,
}

/// 批量转换错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConversionError {
  /// 原始索引
  pub index: usize,

  /// 转换错误
  pub error: ConversionError,
}

/// 批量转换统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConversionStats {
  /// 总数量
  pub total: usize,

  /// 成功数量
  pub successful: usize,

  /// 失败数量
  pub failed: usize,

  /// 成功率
  pub success_rate: f32,

  /// 总转换时间(毫秒)
  pub total_time_ms: u64,

  /// 平均转换时间(微秒)
  pub average_time_us: u64,
}

/// 转换链 - 支持多步转换
pub struct ConversionChain<T> {
  steps: Vec<Box<dyn ConversionStep<T>>>,
  context: ConversionContext,
}

/// 转换步骤
pub trait ConversionStep<T>: Send + Sync {
  /// 执行转换步骤
  fn execute(&self, input: T, context: &ConversionContext) -> Result<T, ConversionError>;

  /// 步骤名称
  fn name(&self) -> &'static str;

  /// 是否是可选步骤
  fn is_optional(&self) -> bool {
    false
  }
}

// 实现
impl ConversionError {
  pub fn new(
    error_type: ConversionErrorType,
    message: String,
    source_type: String,
    target_type: String,
  ) -> Self {
    Self {
      error_type,
      message,
      source_type,
      target_type,
      failed_field: None,
      original_value: None,
      details: None,
      context: ConversionContext::default(),
    }
  }

  pub fn with_field(mut self, field: String) -> Self {
    self.failed_field = Some(field);
    self
  }

  pub fn with_value(mut self, value: String) -> Self {
    self.original_value = Some(value);
    self
  }

  pub fn with_details(mut self, details: String) -> Self {
    self.details = Some(details);
    self
  }

  pub fn with_context(mut self, context: ConversionContext) -> Self {
    self.context = context;
    self
  }

  /// 创建字段缺失错误
  pub fn missing_field(field: &str, source_type: &str, target_type: &str) -> Self {
    Self::new(
      ConversionErrorType::MissingField,
      format!("转换时缺少必需字段: {}", field),
      source_type.to_string(),
      target_type.to_string(),
    )
    .with_field(field.to_string())
  }

  /// 创建类型不匹配错误
  pub fn type_mismatch(field: &str, expected: &str, actual: &str) -> Self {
    Self::new(
      ConversionErrorType::TypeMismatch,
      format!(
        "字段 {} 类型不匹配: 期望 {}, 实际 {}",
        field, expected, actual
      ),
      actual.to_string(),
      expected.to_string(),
    )
    .with_field(field.to_string())
  }

  /// 创建值超出范围错误
  pub fn value_out_of_range(
    field: &str,
    value: &str,
    min: Option<&str>,
    max: Option<&str>,
  ) -> Self {
    let range_desc = match (min, max) {
      (Some(min), Some(max)) => format!("{}到{}", min, max),
      (Some(min), None) => format!("至少{}", min),
      (None, Some(max)) => format!("至多{}", max),
      (None, None) => "有效范围".to_string(),
    };

    Self::new(
      ConversionErrorType::ValueOutOfRange,
      format!("字段 {} 的值 {} 超出范围: {}", field, value, range_desc),
      "input".to_string(),
      "valid_range".to_string(),
    )
    .with_field(field.to_string())
    .with_value(value.to_string())
  }
}

impl fmt::Display for ConversionError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(field) = &self.failed_field {
      write!(f, "转换字段 '{}' 时出错: {}", field, self.message)
    } else {
      write!(
        f,
        "转换 {} -> {} 时出错: {}",
        self.source_type, self.target_type, self.message
      )
    }
  }
}

impl ConversionContext {
  pub fn new() -> Self {
    Self {
      path: Vec::new(),
      operation: None,
      user_id: None,
      workspace_id: None,
      timestamp: chrono::Utc::now(),
      metadata: HashMap::new(),
    }
  }

  pub fn with_path(mut self, path: Vec<String>) -> Self {
    self.path = path;
    self
  }

  pub fn push_path(mut self, segment: String) -> Self {
    self.path.push(segment);
    self
  }

  pub fn with_operation(mut self, operation: String) -> Self {
    self.operation = Some(operation);
    self
  }

  pub fn with_user(mut self, user_id: i64) -> Self {
    self.user_id = Some(user_id);
    self
  }

  pub fn with_workspace(mut self, workspace_id: i64) -> Self {
    self.workspace_id = Some(workspace_id);
    self
  }

  pub fn current_path(&self) -> String {
    self.path.join(".")
  }
}

impl<From, To> BatchConverter<From, To> {
  pub fn new(converter: Box<dyn Converter<From, To>>, strategy: BatchErrorStrategy) -> Self {
    Self {
      converter,
      error_strategy: strategy,
    }
  }

  pub fn convert_batch(
    &self,
    items: Vec<From>,
    context: &ConversionContext,
  ) -> BatchConversionResult<To> {
    let total = items.len();
    let mut successful = Vec::new();
    let mut failed = Vec::new();
    let start_time = std::time::Instant::now();

    for (index, item) in items.into_iter().enumerate() {
      let item_start = std::time::Instant::now();

      match self.converter.convert(&item, context) {
        Ok(converted) => {
          let conversion_time = item_start.elapsed().as_micros() as u64;
          successful.push(BatchConversionItem {
            index,
            item: converted,
            conversion_time_us: conversion_time,
          });
        }
        Err(error) => {
          failed.push(BatchConversionError { index, error });

          // 根据策略决定是否继续
          match self.error_strategy {
            BatchErrorStrategy::FailFast => break,
            BatchErrorStrategy::CollectErrors | BatchErrorStrategy::SkipErrors => continue,
          }
        }
      }
    }

    let total_time = start_time.elapsed().as_millis() as u64;
    let successful_count = successful.len();
    let failed_count = failed.len();
    let success_rate = if total > 0 {
      (successful_count as f32) / (total as f32) * 100.0
    } else {
      0.0
    };

    let average_time = if successful_count > 0 {
      successful.iter().map(|s| s.conversion_time_us).sum::<u64>() / (successful_count as u64)
    } else {
      0
    };

    BatchConversionResult {
      successful,
      failed,
      stats: BatchConversionStats {
        total,
        successful: successful_count,
        failed: failed_count,
        success_rate,
        total_time_ms: total_time,
        average_time_us: average_time,
      },
    }
  }
}

impl<T> ConversionChain<T> {
  pub fn new() -> Self {
    Self {
      steps: Vec::new(),
      context: ConversionContext::new(),
    }
  }

  pub fn add_step(mut self, step: Box<dyn ConversionStep<T>>) -> Self {
    self.steps.push(step);
    self
  }

  pub fn with_context(mut self, context: ConversionContext) -> Self {
    self.context = context;
    self
  }

  pub fn execute(&self, input: T) -> Result<T, ConversionError> {
    let mut current = input;

    for (index, step) in self.steps.iter().enumerate() {
      let step_context = self.context.clone().push_path(format!("step_{}", index));

      match step.execute(current, &step_context) {
        Ok(result) => current = result,
        Err(error) => {
          if step.is_optional() {
            // 可选步骤失败时继续执行
            continue;
          } else {
            return Err(error.with_context(step_context));
          }
        }
      }
    }

    Ok(current)
  }
}

// 常用转换工具
pub struct ConversionUtils;

impl ConversionUtils {
  /// 安全的字符串到数字转换
  pub fn string_to_i64(value: &str, field_name: &str) -> Result<i64, ConversionError> {
    value.parse().map_err(|_| {
      ConversionError::type_mismatch(field_name, "i64", "string")
        .with_value(value.to_string())
        .with_details(format!("无法将字符串 '{}' 转换为整数", value))
    })
  }

  /// 安全的字符串到浮点数转换
  pub fn string_to_f64(value: &str, field_name: &str) -> Result<f64, ConversionError> {
    value.parse().map_err(|_| {
      ConversionError::type_mismatch(field_name, "f64", "string")
        .with_value(value.to_string())
        .with_details(format!("无法将字符串 '{}' 转换为浮点数", value))
    })
  }

  /// 安全的字符串到布尔值转换
  pub fn string_to_bool(value: &str, field_name: &str) -> Result<bool, ConversionError> {
    match value.to_lowercase().as_str() {
      "true" | "1" | "yes" | "on" => Ok(true),
      "false" | "0" | "no" | "off" => Ok(false),
      _ => Err(
        ConversionError::type_mismatch(field_name, "bool", "string")
          .with_value(value.to_string())
          .with_details("有效值: true/false, 1/0, yes/no, on/off".to_string()),
      ),
    }
  }

  /// 选项转换 - 将Option<T>转换为Required<T>
  pub fn option_to_required<T>(
    value: Option<T>,
    field_name: &str,
    type_name: &str,
  ) -> Result<T, ConversionError> {
    value.ok_or_else(|| {
      ConversionError::missing_field(field_name, "Option", type_name)
        .with_details(format!("字段 {} 是必需的", field_name))
    })
  }

  /// 验证范围
  pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: Option<T>,
    max: Option<T>,
    field_name: &str,
  ) -> Result<T, ConversionError> {
    if let Some(min_val) = &min {
      if value < *min_val {
        return Err(ConversionError::value_out_of_range(
          field_name,
          &value.to_string(),
          Some(&min_val.to_string()),
          max.as_ref().map(|m| m.to_string()).as_deref(),
        ));
      }
    }

    if let Some(max_val) = &max {
      if value > *max_val {
        return Err(ConversionError::value_out_of_range(
          field_name,
          &value.to_string(),
          min.as_ref().map(|m| m.to_string()).as_deref(),
          Some(&max_val.to_string()),
        ));
      }
    }

    Ok(value)
  }
}

// 与CoreError的集成
impl From<CoreError> for ConversionError {
  fn from(error: CoreError) -> Self {
    match error {
      CoreError::Validation(msg) => Self::new(
        ConversionErrorType::BusinessRuleViolation,
        msg,
        "input".to_string(),
        "validated_output".to_string(),
      ),
      CoreError::NotFound(msg) => Self::new(
        ConversionErrorType::MissingDependency,
        msg,
        "reference".to_string(),
        "entity".to_string(),
      ),
      CoreError::Database(msg) => Self::new(
        ConversionErrorType::DataIntegrityError,
        msg,
        "data".to_string(),
        "persisted_data".to_string(),
      ),
      CoreError::Permission(msg) => Self::new(
        ConversionErrorType::InsufficientPermissions,
        msg,
        "user_action".to_string(),
        "authorized_action".to_string(),
      ),
    }
  }
}
