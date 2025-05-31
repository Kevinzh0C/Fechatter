// 统一验证框架
//
// 提供组合验证器、自定义验证规则、验证结果聚合等功能
// 支持字段级验证、对象级验证、跨字段验证等复杂场景

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use validator::{Validate, ValidationError, ValidationErrors};

/// DTO验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtoValidationError {
  /// 错误类型
  pub error_type: ValidationErrorType,

  /// 错误消息
  pub message: String,

  /// 字段路径
  pub field_path: Option<String>,

  /// 验证规则
  pub rule: Option<String>,

  /// 期望值
  pub expected: Option<String>,

  /// 实际值
  pub actual: Option<String>,

  /// 建议修复方案
  pub suggestion: Option<String>,
}

/// 验证错误类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationErrorType {
  /// 必填字段缺失
  Required,

  /// 格式错误
  Format,

  /// 长度错误
  Length,

  /// 范围错误
  Range,

  /// 邮箱格式错误
  Email,

  /// URL格式错误
  Url,

  /// 自定义规则错误
  Custom,

  /// 业务规则错误
  Business,

  /// 依赖字段错误
  Dependency,

  /// 唯一性错误
  Unique,
}

/// 验证结果聚合器
#[derive(Debug, Clone)]
pub struct ValidationResultCollector {
  errors: Vec<DtoValidationError>,
  warnings: Vec<ValidationWarning>,
}

/// 验证警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
  pub field_path: String,
  pub message: String,
  pub suggestion: Option<String>,
}

/// 验证上下文 - 提供验证时的额外信息
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
  /// 用户ID
  pub user_id: Option<i64>,

  /// 工作空间ID
  pub workspace_id: Option<i64>,

  /// 操作类型
  pub operation: Option<String>,

  /// 当前时间
  pub current_time: chrono::DateTime<chrono::Utc>,

  /// 扩展数据
  pub extensions: HashMap<String, String>,
}

/// 自定义验证器特征
pub trait CustomValidator: Send + Sync {
  /// 验证函数
  fn validate(&self, value: &str, context: &ValidationContext) -> Result<(), DtoValidationError>;

  /// 验证器名称
  fn name(&self) -> &'static str;

  /// 验证器描述
  fn description(&self) -> &'static str;
}

/// 组合验证器 - 可以组合多个验证规则
pub struct CompositeValidator {
  validators: Vec<Box<dyn CustomValidator>>,
  mode: ValidationMode,
}

/// 验证模式
#[derive(Debug, Clone)]
pub enum ValidationMode {
  /// 快速失败 - 遇到第一个错误就停止
  FailFast,

  /// 收集所有错误
  CollectAll,

  /// 条件验证 - 根据条件决定是否继续
  Conditional(fn(&DtoValidationError) -> bool),
}

/// 常用验证器实现
pub struct LengthValidator {
  min: Option<usize>,
  max: Option<usize>,
}

pub struct EmailValidator;

pub struct PasswordStrengthValidator {
  min_length: usize,
  require_uppercase: bool,
  require_lowercase: bool,
  require_numbers: bool,
  require_symbols: bool,
}

pub struct UniqueEmailValidator;

pub struct WorkspaceNameValidator;

// 实现
impl DtoValidationError {
  pub fn new(error_type: ValidationErrorType, message: String, field_path: Option<String>) -> Self {
    Self {
      error_type,
      message,
      field_path,
      rule: None,
      expected: None,
      actual: None,
      suggestion: None,
    }
  }

  pub fn with_rule(mut self, rule: String) -> Self {
    self.rule = Some(rule);
    self
  }

  pub fn with_expected(mut self, expected: String) -> Self {
    self.expected = Some(expected);
    self
  }

  pub fn with_actual(mut self, actual: String) -> Self {
    self.actual = Some(actual);
    self
  }

  pub fn with_suggestion(mut self, suggestion: String) -> Self {
    self.suggestion = Some(suggestion);
    self
  }
}

impl fmt::Display for DtoValidationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(field) = &self.field_path {
      write!(f, "字段 '{}': {}", field, self.message)
    } else {
      write!(f, "{}", self.message)
    }
  }
}

impl ValidationResultCollector {
  pub fn new() -> Self {
    Self {
      errors: Vec::new(),
      warnings: Vec::new(),
    }
  }

  pub fn add_error(&mut self, error: DtoValidationError) {
    self.errors.push(error);
  }

  pub fn add_warning(&mut self, warning: ValidationWarning) {
    self.warnings.push(warning);
  }

  pub fn has_errors(&self) -> bool {
    !self.errors.is_empty()
  }

  pub fn has_warnings(&self) -> bool {
    !self.warnings.is_empty()
  }

  pub fn errors(&self) -> &[DtoValidationError] {
    &self.errors
  }

  pub fn warnings(&self) -> &[ValidationWarning] {
    &self.warnings
  }

  pub fn into_result(self) -> Result<Vec<ValidationWarning>, Vec<DtoValidationError>> {
    if self.has_errors() {
      Err(self.errors)
    } else {
      Ok(self.warnings)
    }
  }
}

impl ValidationContext {
  pub fn new() -> Self {
    Self {
      user_id: None,
      workspace_id: None,
      operation: None,
      current_time: chrono::Utc::now(),
      extensions: HashMap::new(),
    }
  }

  pub fn with_user(mut self, user_id: i64) -> Self {
    self.user_id = Some(user_id);
    self
  }

  pub fn with_workspace(mut self, workspace_id: i64) -> Self {
    self.workspace_id = Some(workspace_id);
    self
  }

  pub fn with_operation(mut self, operation: String) -> Self {
    self.operation = Some(operation);
    self
  }
}

impl CompositeValidator {
  pub fn new(mode: ValidationMode) -> Self {
    Self {
      validators: Vec::new(),
      mode,
    }
  }

  pub fn add_validator(mut self, validator: Box<dyn CustomValidator>) -> Self {
    self.validators.push(validator);
    self
  }

  pub fn validate(
    &self,
    value: &str,
    context: &ValidationContext,
  ) -> Result<(), Vec<DtoValidationError>> {
    let mut errors = Vec::new();

    for validator in &self.validators {
      match validator.validate(value, context) {
        Ok(_) => continue,
        Err(error) => {
          errors.push(error);

          // 根据验证模式决定是否继续
          match &self.mode {
            ValidationMode::FailFast => break,
            ValidationMode::CollectAll => continue,
            ValidationMode::Conditional(predicate) => {
              if !predicate(errors.last().unwrap()) {
                break;
              }
            }
          }
        }
      }
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }
}

// 常用验证器实现
impl LengthValidator {
  pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
    Self { min, max }
  }
}

impl CustomValidator for LengthValidator {
  fn validate(&self, value: &str, _context: &ValidationContext) -> Result<(), DtoValidationError> {
    let len = value.chars().count();

    if let Some(min) = self.min {
      if len < min {
        return Err(
          DtoValidationError::new(
            ValidationErrorType::Length,
            format!("长度不能少于{}个字符", min),
            None,
          )
          .with_expected(format!("至少{}个字符", min))
          .with_actual(format!("{}个字符", len))
          .with_suggestion(format!("请输入至少{}个字符", min)),
        );
      }
    }

    if let Some(max) = self.max {
      if len > max {
        return Err(
          DtoValidationError::new(
            ValidationErrorType::Length,
            format!("长度不能超过{}个字符", max),
            None,
          )
          .with_expected(format!("最多{}个字符", max))
          .with_actual(format!("{}个字符", len))
          .with_suggestion(format!("请将内容缩减到{}个字符以内", max)),
        );
      }
    }

    Ok(())
  }

  fn name(&self) -> &'static str {
    "length"
  }

  fn description(&self) -> &'static str {
    "验证字符串长度"
  }
}

impl CustomValidator for EmailValidator {
  fn validate(&self, value: &str, _context: &ValidationContext) -> Result<(), DtoValidationError> {
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();

    if !email_regex.is_match(value) {
      return Err(
        DtoValidationError::new(
          ValidationErrorType::Email,
          "邮箱格式不正确".to_string(),
          None,
        )
        .with_expected("有效的邮箱地址".to_string())
        .with_actual(value.to_string())
        .with_suggestion("请输入有效的邮箱地址，如: user@example.com".to_string()),
      );
    }

    Ok(())
  }

  fn name(&self) -> &'static str {
    "email"
  }

  fn description(&self) -> &'static str {
    "验证邮箱格式"
  }
}

impl PasswordStrengthValidator {
  pub fn new() -> Self {
    Self {
      min_length: 8,
      require_uppercase: true,
      require_lowercase: true,
      require_numbers: true,
      require_symbols: false,
    }
  }

  pub fn min_length(mut self, min_length: usize) -> Self {
    self.min_length = min_length;
    self
  }

  pub fn require_uppercase(mut self, require: bool) -> Self {
    self.require_uppercase = require;
    self
  }

  pub fn require_lowercase(mut self, require: bool) -> Self {
    self.require_lowercase = require;
    self
  }

  pub fn require_numbers(mut self, require: bool) -> Self {
    self.require_numbers = require;
    self
  }

  pub fn require_symbols(mut self, require: bool) -> Self {
    self.require_symbols = require;
    self
  }
}

impl CustomValidator for PasswordStrengthValidator {
  fn validate(&self, value: &str, _context: &ValidationContext) -> Result<(), DtoValidationError> {
    if value.len() < self.min_length {
      return Err(
        DtoValidationError::new(
          ValidationErrorType::Length,
          format!("密码长度不能少于{}位", self.min_length),
          None,
        )
        .with_suggestion(format!("请使用至少{}位字符的密码", self.min_length)),
      );
    }

    if self.require_uppercase && !value.chars().any(|c| c.is_uppercase()) {
      return Err(
        DtoValidationError::new(
          ValidationErrorType::Custom,
          "密码必须包含大写字母".to_string(),
          None,
        )
        .with_suggestion("请在密码中添加至少一个大写字母".to_string()),
      );
    }

    if self.require_lowercase && !value.chars().any(|c| c.is_lowercase()) {
      return Err(
        DtoValidationError::new(
          ValidationErrorType::Custom,
          "密码必须包含小写字母".to_string(),
          None,
        )
        .with_suggestion("请在密码中添加至少一个小写字母".to_string()),
      );
    }

    if self.require_numbers && !value.chars().any(|c| c.is_numeric()) {
      return Err(
        DtoValidationError::new(
          ValidationErrorType::Custom,
          "密码必须包含数字".to_string(),
          None,
        )
        .with_suggestion("请在密码中添加至少一个数字".to_string()),
      );
    }

    if self.require_symbols && !value.chars().any(|c| !c.is_alphanumeric()) {
      return Err(
        DtoValidationError::new(
          ValidationErrorType::Custom,
          "密码必须包含特殊字符".to_string(),
          None,
        )
        .with_suggestion("请在密码中添加至少一个特殊字符(!@#$%^&*等)".to_string()),
      );
    }

    Ok(())
  }

  fn name(&self) -> &'static str {
    "password_strength"
  }

  fn description(&self) -> &'static str {
    "验证密码强度"
  }
}

/// 验证器工厂 - 创建常用验证器的便捷方法
pub struct ValidatorFactory;

impl ValidatorFactory {
  pub fn email() -> Box<dyn CustomValidator> {
    Box::new(EmailValidator)
  }

  pub fn length(min: Option<usize>, max: Option<usize>) -> Box<dyn CustomValidator> {
    Box::new(LengthValidator::new(min, max))
  }

  pub fn password_strength() -> Box<dyn CustomValidator> {
    Box::new(PasswordStrengthValidator::new())
  }

  pub fn password_strong() -> Box<dyn CustomValidator> {
    Box::new(
      PasswordStrengthValidator::new()
        .min_length(12)
        .require_uppercase(true)
        .require_lowercase(true)
        .require_numbers(true)
        .require_symbols(true),
    )
  }
}

/// 验证错误转换工具 - 替代From trait实现以避免orphan rule
pub struct ValidationErrorConverter;

impl ValidationErrorConverter {
  /// 将validator crate的ValidationErrors转换为我们的DtoValidationError
  pub fn from_validation_errors(errors: ValidationErrors) -> Vec<DtoValidationError> {
    let mut dto_errors = Vec::new();

    for (field, field_errors) in errors.field_errors() {
      for error in field_errors {
        let error_type = match error.code.as_ref() {
          "required" => ValidationErrorType::Required,
          "email" => ValidationErrorType::Email,
          "length" => ValidationErrorType::Length,
          "range" => ValidationErrorType::Range,
          "url" => ValidationErrorType::Url,
          _ => ValidationErrorType::Custom,
        };

        let message = error
          .message
          .as_ref()
          .map(|m| m.to_string())
          .unwrap_or_else(|| format!("字段验证失败: {}", error.code));

        dto_errors.push(DtoValidationError::new(
          error_type,
          message,
          Some(field.to_string()),
        ));
      }
    }

    dto_errors
  }

  /// 便捷方法：直接从验证结果获取第一个错误
  pub fn first_error_from_validation_errors(
    errors: ValidationErrors,
  ) -> Option<DtoValidationError> {
    Self::from_validation_errors(errors).into_iter().next()
  }

  /// 将单个ValidationError转换为DtoValidationError
  pub fn from_single_validation_error(field: &str, error: &ValidationError) -> DtoValidationError {
    let error_type = match error.code.as_ref() {
      "required" => ValidationErrorType::Required,
      "email" => ValidationErrorType::Email,
      "length" => ValidationErrorType::Length,
      "range" => ValidationErrorType::Range,
      "url" => ValidationErrorType::Url,
      _ => ValidationErrorType::Custom,
    };

    let message = error
      .message
      .as_ref()
      .map(|m| m.to_string())
      .unwrap_or_else(|| format!("字段验证失败: {}", error.code));

    DtoValidationError::new(error_type, message, Some(field.to_string()))
  }
}
