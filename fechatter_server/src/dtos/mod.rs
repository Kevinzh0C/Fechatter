// 重新设计的DTOs架构
//
// 从Clean Architecture角度，DTOs属于Interface Adapters层
// 核心职责：
// 1. 边界隔离 - API层与Domain层之间的防腐层
// 2. 数据适配 - 外部格式↔内部模型的双向转换
// 3. 契约定义 - API接口的稳定契约，独立于内部实现变化
// 4. 验证前置 - 在数据进入核心业务逻辑前进行完整验证
//
// 架构设计原则：
// - 依赖方向：DTOs → Domain (单向依赖)
// - 职责分离：Request DTOs负责输入，Response DTOs负责输出
// - 可组合性：通过统一框架支持复杂业务场景
// - 类型安全：强类型的转换和验证机制
// - 性能优化：支持批量转换和缓存策略

// === 核心框架 ===
pub mod core;

// === 传统模块（重构中）===
pub mod mappers;
pub mod models;

// === 重新导出核心类型 ===
pub use core::*;

// === 传统类型重新导出（向后兼容）===
pub use models::*;

// === 新架构的统一入口 ===

use std::sync::Arc;

/// DTOs管理器 - 统一管理所有DTOs相关功能
pub struct DtoManager {
  validator_registry: Arc<ValidatorRegistry>,
  converter_registry: Arc<ConverterRegistry>,
  response_builder: Arc<ResponseBuilder>,
  pagination_config: PaginationConfig,
}

/// 验证器注册表
pub struct ValidatorRegistry {
  validators: std::collections::HashMap<String, Box<dyn CustomValidator>>,
}

/// 转换器注册表
pub struct ConverterRegistry {
  // 存储类型到转换器的映射
  // 这里简化为字符串键，实际实现中可能需要更复杂的类型系统
}

/// 响应构建器
pub struct ResponseBuilder {
  default_server_info: Option<ServerInfo>,
  request_id_generator: Box<dyn Fn() -> String + Send + Sync>,
}

/// 分页配置
#[derive(Debug, Clone)]
pub struct PaginationConfig {
  pub default_page_size: u32,
  pub max_page_size: u32,
  pub enable_cursor_pagination: bool,
}

impl Default for PaginationConfig {
  fn default() -> Self {
    Self {
      default_page_size: 20,
      max_page_size: 100,
      enable_cursor_pagination: true,
    }
  }
}

impl DtoManager {
  /// 创建新的DTOs管理器
  pub fn new() -> Self {
    Self {
      validator_registry: Arc::new(ValidatorRegistry::new()),
      converter_registry: Arc::new(ConverterRegistry::new()),
      response_builder: Arc::new(ResponseBuilder::new()),
      pagination_config: PaginationConfig::default(),
    }
  }

  /// 配置分页参数
  pub fn with_pagination_config(mut self, config: PaginationConfig) -> Self {
    self.pagination_config = config;
    self
  }

  /// 配置响应构建器
  pub fn with_response_builder(mut self, builder: ResponseBuilder) -> Self {
    self.response_builder = Arc::new(builder);
    self
  }

  /// 注册验证器
  pub fn register_validator(&mut self, name: String, validator: Box<dyn CustomValidator>) {
    Arc::get_mut(&mut self.validator_registry)
      .unwrap()
      .register(name, validator);
  }

  /// 验证DTO
  pub fn validate_dto<T: BaseDto>(
    &self,
    dto: &T,
    context: &ValidationContext,
  ) -> Result<(), Vec<DtoValidationError>> {
    // 1. 基础验证
    if let Err(error) = dto.validate() {
      return Err(vec![error]);
    }

    // 2. 自定义验证（如果需要）
    // 这里可以添加更复杂的验证逻辑

    Ok(())
  }

  /// 转换请求DTO到领域模型
  pub fn convert_request<R: RequestDto>(
    &self,
    request: &R,
    context: &ConversionContext,
  ) -> Result<R::DomainModel, ConversionError> {
    request.to_domain()
  }

  /// 从领域模型创建响应DTO
  pub fn create_response<R: ResponseDto>(
    &self,
    domain: &R::DomainModel,
    request_id: String,
  ) -> Result<ApiResponse<R>, ConversionError> {
    let response_dto = R::from_domain(domain)?;
    Ok(ApiResponse::success(response_dto, request_id))
  }

  /// 创建分页响应
  pub fn create_paginated_response<R: ResponseDto>(
    &self,
    domains: &[R::DomainModel],
    pagination: PaginationRequest,
    total_items: u64,
    request_id: String,
  ) -> Result<ListResponse<R>, ConversionError> {
    let response_dtos = R::from_domain_collection(domains)?;
    let paginated = PaginatedResponse::new(
      response_dtos,
      pagination.page,
      pagination.page_size,
      total_items,
    );
    Ok(ApiResponse::success(paginated, request_id))
  }

  /// 创建批量操作响应
  pub fn create_batch_response<R: ResponseDto>(
    &self,
    results: Vec<Result<R::DomainModel, fechatter_core::error::CoreError>>,
    request_id: String,
  ) -> BatchCreateResponse<R> {
    let mut batch_response = BatchResponse::new();

    for (index, result) in results.into_iter().enumerate() {
      match result {
        Ok(domain) => {
          if let Ok(dto) = R::from_domain(&domain) {
            batch_response.add_success(index, None, dto);
          } else {
            let error = ApiError::from(fechatter_core::error::CoreError::Validation(
              "转换失败".to_string(),
            ));
            batch_response.add_failure(index, None, error);
          }
        }
        Err(core_error) => {
          let error = ApiError::from(core_error);
          batch_response.add_failure(index, None, error);
        }
      }
    }

    ApiResponse::success(batch_response, request_id)
  }
}

impl ValidatorRegistry {
  pub fn new() -> Self {
    let mut registry = Self {
      validators: std::collections::HashMap::new(),
    };

    // 注册内置验证器
    registry.register("email".to_string(), ValidatorFactory::email());
    registry.register(
      "password_strength".to_string(),
      ValidatorFactory::password_strength(),
    );
    registry.register(
      "password_strong".to_string(),
      ValidatorFactory::password_strong(),
    );

    registry
  }

  pub fn register(&mut self, name: String, validator: Box<dyn CustomValidator>) {
    self.validators.insert(name, validator);
  }

  pub fn get(&self, name: &str) -> Option<&Box<dyn CustomValidator>> {
    self.validators.get(name)
  }
}

impl ConverterRegistry {
  pub fn new() -> Self {
    Self {
      // 初始化转换器注册表
    }
  }
}

impl ResponseBuilder {
  pub fn new() -> Self {
    Self {
      default_server_info: None,
      request_id_generator: Box::new(|| uuid::Uuid::new_v4().to_string()),
    }
  }

  pub fn with_server_info(mut self, server_info: ServerInfo) -> Self {
    self.default_server_info = Some(server_info);
    self
  }

  pub fn with_request_id_generator<F>(mut self, generator: F) -> Self
  where
    F: Fn() -> String + Send + Sync + 'static,
  {
    self.request_id_generator = Box::new(generator);
    self
  }

  pub fn generate_request_id(&self) -> String {
    (self.request_id_generator)()
  }

  pub fn build_success_response<T>(&self, data: T) -> ApiResponse<T> {
    let request_id = self.generate_request_id();
    let mut response = ApiResponse::success(data, request_id);

    if let Some(server_info) = &self.default_server_info {
      response.meta = response.meta.with_server_info(server_info.clone());
    }

    response
  }

  pub fn build_error_response(&self, error: ApiError) -> ErrorResponse {
    let request_id = self.generate_request_id();
    let mut response = ApiResponse::error(error, request_id);

    if let Some(server_info) = &self.default_server_info {
      response.meta = response.meta.with_server_info(server_info.clone());
    }

    response
  }
}

/// 全局DTOs管理器实例
static mut DTO_MANAGER: Option<DtoManager> = None;
static mut DTO_MANAGER_INIT: std::sync::Once = std::sync::Once::new();

/// 获取全局DTOs管理器
pub fn get_dto_manager() -> &'static DtoManager {
  unsafe {
    DTO_MANAGER_INIT.call_once(|| {
      DTO_MANAGER = Some(DtoManager::new());
    });
    DTO_MANAGER.as_ref().unwrap()
  }
}

/// 初始化DTOs管理器（可自定义配置）
pub fn init_dto_manager(manager: DtoManager) {
  unsafe {
    DTO_MANAGER_INIT.call_once(|| {
      DTO_MANAGER = Some(manager);
    });
  }
}

/// 定义请求DTO的宏
#[macro_export]
macro_rules! define_request_dto {
  (
    $(#[$meta:meta])*
    pub struct $name:ident {
      $(
        $(#[$field_meta:meta])*
        pub $field:ident: $field_type:ty,
      )*
    }

    domain_type = $domain_type:ty;

    convert = |$self_param:ident| {
      $($convert_body:tt)*
    };
  ) => {
    $(#[$meta])*
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, validator::Validate)]
    pub struct $name {
      $(
        $(#[$field_meta])*
        pub $field: $field_type,
      )*
    }

    impl $crate::dtos::core::BaseDto for $name {
      fn dto_type() -> &'static str {
        stringify!($name)
      }

      fn validate(&self) -> Result<(), $crate::dtos::core::DtoValidationError> {
        use validator::Validate;
        self.validate()
          .map_err(|errors| {
            $crate::dtos::core::ValidationErrorConverter::first_error_from_validation_errors(errors)
              .unwrap_or_else(|| {
                $crate::dtos::core::DtoValidationError::new(
                  $crate::dtos::core::ValidationErrorType::Custom,
                  "验证失败".to_string(),
                  None,
                )
              })
          })
      }
    }

    impl $crate::dtos::core::RequestDto for $name {
      type DomainModel = $domain_type;

      fn to_domain(&self) -> Result<Self::DomainModel, $crate::dtos::core::ConversionError> {
        let $self_param = self;
        Ok({
          $($convert_body)*
        })
      }
    }
  };
}

/// 定义响应DTO的宏
#[macro_export]
macro_rules! define_response_dto {
  (
    $(#[$meta:meta])*
    pub struct $name:ident {
      $(
        $(#[$field_meta:meta])*
        pub $field:ident: $field_type:ty,
      )*
    }

    domain_type = $domain_type:ty;

    from_domain = |$domain_param:ident| {
      $($from_domain_body:tt)*
    };
  ) => {
    $(#[$meta])*
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct $name {
      $(
        $(#[$field_meta])*
        pub $field: $field_type,
      )*
    }

    impl $crate::dtos::core::BaseDto for $name {
      fn dto_type() -> &'static str {
        stringify!($name)
      }

      fn validate(&self) -> Result<(), $crate::dtos::core::DtoValidationError> {
        // 响应DTO通常不需要验证
        Ok(())
      }
    }

    impl $crate::dtos::core::ResponseDto for $name {
      type DomainModel = $domain_type;

      fn from_domain(domain: &Self::DomainModel) -> Result<Self, $crate::dtos::core::ConversionError> {
        let $domain_param = domain;
        Ok(Self {
          $($from_domain_body)*
        })
      }
    }
  };
}

// === 示例用法（文档） ===
#[cfg(test)]
mod example_usage {
  use super::*;

  // 使用新的宏定义DTOs的示例：
  /*
  define_request_dto! {
    #[derive(Debug)]
    pub struct CreateUserRequestDto {
      #[validate(email)]
      pub email: String,

      #[validate(length(min = 2, max = 50))]
      pub fullname: String,

      #[validate(length(min = 8))]
      pub password: String,
    }

    domain_type = fechatter_core::CreateUser;

    convert = |req| {
      fechatter_core::CreateUser {
        email: req.email.clone(),
        fullname: req.fullname.clone(),
        password: req.password.clone(),
        workspace_id: None,
      }
    };
  }

  define_response_dto! {
    pub struct UserResponseDto {
      pub id: i64,
      pub email: String,
      pub fullname: String,
      pub created_at: chrono::DateTime<chrono::Utc>,
    }

    domain_type = fechatter_core::User;

    from_domain = |user| {
      id: user.id.into(),
      email: user.email.clone(),
      fullname: user.fullname.clone(),
      created_at: user.created_at,
    };
  }
  */
}
