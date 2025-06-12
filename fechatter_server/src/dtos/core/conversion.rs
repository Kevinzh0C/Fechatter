// Conversion errors and utilities
//
// Handles conversion between DTOs and Domain models
// Provides type-safe conversion, error handling, batch conversion and other functionality

use fechatter_core::error::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Conversion error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionError {
  /// Error type
  pub error_type: ConversionErrorType,

  /// Error message
  pub message: String,

  /// Source type
  pub source_type: String,

  /// Target type
  pub target_type: String,

  /// Failed field
  pub failed_field: Option<String>,

  /// Original value
  pub original_value: Option<String>,

  /// Error details
  pub details: Option<String>,

  /// Conversion context
  pub context: ConversionContext,
}

/// Conversion error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversionErrorType {
  /// Missing required field
  MissingField,

  /// Type mismatch
  TypeMismatch,

  /// Value out of valid range
  ValueOutOfRange,

  /// Invalid format
  InvalidFormat,

  /// Business rule violation
  BusinessRuleViolation,

  /// Missing dependency
  MissingDependency,

  /// Circular reference
  CircularReference,

  /// Data integrity error
  DataIntegrityError,

  /// Insufficient permissions
  InsufficientPermissions,

  /// Unknown error
  Unknown,
}

/// Conversion context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversionContext {
  /// Conversion path
  pub path: Vec<String>,

  /// Operation type
  pub operation: Option<String>,

  /// User ID
  pub user_id: Option<i64>,

  /// Workspace ID
  pub workspace_id: Option<i64>,

  /// Timestamp
  pub timestamp: chrono::DateTime<chrono::Utc>,

  /// Additional metadata
  pub metadata: HashMap<String, String>,
}

/// Converter trait - defines type-safe bidirectional conversion
pub trait Converter<From, To>: Send + Sync {
  /// Forward conversion
  fn convert(&self, from: &From, context: &ConversionContext) -> Result<To, ConversionError>;

  /// Reverse conversion (if supported)
  fn convert_back(&self, to: &To, context: &ConversionContext) -> Result<From, ConversionError> {
    Err(ConversionError::new(
      ConversionErrorType::Unknown,
      "Reverse conversion not implemented".to_string(),
      std::any::type_name::<To>().to_string(),
      std::any::type_name::<From>().to_string(),
    ))
  }

  /// Converter name
  fn name(&self) -> &'static str;

  /// Whether reverse conversion is supported
  fn supports_reverse(&self) -> bool {
    false
  }
}

/// Batch converter
pub struct BatchConverter<From, To> {
  converter: Box<dyn Converter<From, To>>,
  error_strategy: BatchErrorStrategy,
}

/// Batch error handling strategies
#[derive(Debug, Clone)]
pub enum BatchErrorStrategy {
  /// Stop on first error
  FailFast,

  /// Collect all errors, return successful items and error list
  CollectErrors,

  /// Skip error items, return only successful items
  SkipErrors,
}

/// Batch conversion result
#[derive(Debug, Clone)]
pub struct BatchConversionResult<T> {
  /// Successfully converted items
  pub successful: Vec<BatchConversionItem<T>>,

  /// Failed items
  pub failed: Vec<BatchConversionError>,

  /// Statistics
  pub stats: BatchConversionStats,
}

/// Batch conversion item
#[derive(Debug, Clone)]
pub struct BatchConversionItem<T> {
  /// Original index
  pub index: usize,

  /// Converted item
  pub item: T,

  /// Conversion time (microseconds)
  pub conversion_time_us: u64,
}

/// Batch conversion error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConversionError {
  /// Original index
  pub index: usize,

  /// Conversion error
  pub error: ConversionError,
}

/// Batch conversion statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConversionStats {
  /// Total count
  pub total: usize,

  /// Success count
  pub successful: usize,

  /// Failure count
  pub failed: usize,

  /// Success rate
  pub success_rate: f32,

  /// Total time (milliseconds)
  pub total_time_ms: u64,

  /// Average time (microseconds)
  pub average_time_us: u64,
}

/// Conversion chain - supports multi-step conversion
pub struct ConversionChain<T> {
  steps: Vec<Box<dyn ConversionStep<T>>>,
  context: ConversionContext,
}

/// Conversion step
pub trait ConversionStep<T>: Send + Sync {
  /// Execute conversion step
  fn execute(&self, input: T, context: &ConversionContext) -> Result<T, ConversionError>;

  /// Step name
  fn name(&self) -> &'static str;

  /// Whether step is optional
  fn is_optional(&self) -> bool {
    false
  }
}

// Implementations
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

  /// Create missing field error
  pub fn missing_field(field: &str, source_type: &str, target_type: &str) -> Self {
    Self::new(
      ConversionErrorType::MissingField,
      format!("Required field missing during conversion: {}", field),
      source_type.to_string(),
      target_type.to_string(),
    )
    .with_field(field.to_string())
  }

  /// Create type mismatch error
  pub fn type_mismatch(field: &str, expected: &str, actual: &str) -> Self {
    Self::new(
      ConversionErrorType::TypeMismatch,
      format!(
        "Type mismatch for field {}: expected {}, got {}",
        field, expected, actual
      ),
      actual.to_string(),
      expected.to_string(),
    )
    .with_field(field.to_string())
  }

  /// Create value out of range error
  pub fn value_out_of_range(
    field: &str,
    value: &str,
    min: Option<&str>,
    max: Option<&str>,
  ) -> Self {
    let range_desc = match (min, max) {
      (Some(min), Some(max)) => format!("{} to {}", min, max),
      (Some(min), None) => format!("at least {}", min),
      (None, Some(max)) => format!("at most {}", max),
      (None, None) => "valid range".to_string(),
    };

    Self::new(
      ConversionErrorType::ValueOutOfRange,
      format!(
        "Value {} for field {} is out of range: {}",
        value, field, range_desc
      ),
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
      write!(f, "Error converting field '{}': {}", field, self.message)
    } else {
      write!(
        f,
        "Error converting {} -> {}: {}",
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

          // Continue based on strategy
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
          // For now, treat all errors the same way to avoid ownership issues
          return Err(error.with_context(step_context));
        }
      }
    }

    Ok(current)
  }
}

// Common conversion utilities
pub struct ConversionUtils;

impl ConversionUtils {
  /// Safe string to integer conversion
  pub fn string_to_i64(value: &str, field_name: &str) -> Result<i64, ConversionError> {
    value.parse().map_err(|_| {
      ConversionError::type_mismatch(field_name, "i64", "string")
        .with_value(value.to_string())
        .with_details(format!("Cannot convert string '{}' to integer", value))
    })
  }

  /// Safe string to float conversion
  pub fn string_to_f64(value: &str, field_name: &str) -> Result<f64, ConversionError> {
    value.parse().map_err(|_| {
      ConversionError::type_mismatch(field_name, "f64", "string")
        .with_value(value.to_string())
        .with_details(format!("Cannot convert string '{}' to float", value))
    })
  }

  /// Safe string to boolean conversion
  pub fn string_to_bool(value: &str, field_name: &str) -> Result<bool, ConversionError> {
    match value.to_lowercase().as_str() {
      "true" | "1" | "yes" | "on" => Ok(true),
      "false" | "0" | "no" | "off" => Ok(false),
      _ => Err(
        ConversionError::type_mismatch(field_name, "bool", "string")
          .with_value(value.to_string())
          .with_details("Valid values: true/false, 1/0, yes/no, on/off".to_string()),
      ),
    }
  }

  /// Option conversion - Convert Option<T> to Required<T>
  pub fn option_to_required<T>(
    value: Option<T>,
    field_name: &str,
    type_name: &str,
  ) -> Result<T, ConversionError> {
    value.ok_or_else(|| {
      ConversionError::missing_field(field_name, "Option", type_name)
        .with_details(format!("Field {} is required", field_name))
    })
  }

  /// Validate range
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

// Integration with CoreError
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
      // Handle all other CoreError variants with a generic conversion
      _ => Self::new(
        ConversionErrorType::Unknown,
        format!("CoreError conversion: {:?}", error),
        "core_error".to_string(),
        "conversion_error".to_string(),
      ),
    }
  }
}
