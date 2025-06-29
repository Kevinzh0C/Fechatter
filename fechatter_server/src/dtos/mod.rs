// Redesigned DTOs Architecture
//
// From the perspective of Clean Architecture, DTOs belong to the Interface Adapters layer.
// Core responsibilities:
// 1. Boundary isolation - an anti-corruption layer between the API layer and the Domain layer
// 2. Data adaptation - bidirectional conversion between external formats and internal models
// 3. Contract definition - stable API interface contracts, independent of internal implementation changes
// 4. Pre-validation - complete validation before data enters core business logic
//
// Architectural design principles:
// - Dependency direction: DTOs → Domain (one-way dependency)
// - Separation of concerns: Request DTOs handle input, Response DTOs handle output
// - Composability: Unified framework supports complex business scenarios
// - Type safety: Strongly-typed conversion and validation mechanisms
// - Performance optimization: Supports batch conversion and caching strategies

// === Core Framework ===
pub mod core;
pub mod bot;

// === Legacy Modules (under refactoring) ===
pub mod mappers;
pub mod models;

// === Re-export Core Types ===
pub use core::*;

// === Re-export Legacy Types (for backward compatibility) ===
pub use models::*;

// === Unified Entry Point for the New Architecture ===

use std::sync::Arc;

/// DTOs Manager - Unified management for all DTO-related functionality
pub struct DtoManager {
  validator_registry: Arc<ValidatorRegistry>,
  converter_registry: Arc<ConverterRegistry>,
  response_builder: Arc<ResponseBuilder>,
  pagination_config: PaginationConfig,
}

/// Validator Registry
pub struct ValidatorRegistry {
  validators: std::collections::HashMap<String, Box<dyn CustomValidator>>,
}

/// Converter Registry
pub struct ConverterRegistry {
  // Stores mapping from types to converters
  // Simplified as string keys here; actual implementation may require a more complex type system
}

/// Response Builder
pub struct ResponseBuilder {
  default_server_info: Option<ServerInfo>,
  request_id_generator: Box<dyn Fn() -> String + Send + Sync>,
}

/// Pagination Configuration
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
  /// Create a new DTOs manager
  pub fn new() -> Self {
    Self {
      validator_registry: Arc::new(ValidatorRegistry::new()),
      converter_registry: Arc::new(ConverterRegistry::new()),
      response_builder: Arc::new(ResponseBuilder::new()),
      pagination_config: PaginationConfig::default(),
    }
  }

  /// Configure pagination parameters
  pub fn with_pagination_config(mut self, config: PaginationConfig) -> Self {
    self.pagination_config = config;
    self
  }

  /// Configure response builder
  pub fn with_response_builder(mut self, builder: ResponseBuilder) -> Self {
    self.response_builder = Arc::new(builder);
    self
  }

  /// Register a validator
  pub fn register_validator(&mut self, name: String, validator: Box<dyn CustomValidator>) {
    Arc::get_mut(&mut self.validator_registry)
      .unwrap()
      .register(name, validator);
  }

  /// Validate DTO
  pub fn validate_dto<T: BaseDto>(
    &self,
    dto: &T,
    context: &ValidationContext,
  ) -> Result<(), Vec<DtoValidationError>> {
    // 1. Basic validation
    if let Err(error) = dto.validate() {
      return Err(vec![error]);
    }

    // 2. Custom validation (if needed)
    // More complex validation logic can be added here

    Ok(())
  }

  /// Convert request DTO to domain model
  pub fn convert_request<R: RequestDto>(
    &self,
    request: &R,
    context: &ConversionContext,
  ) -> Result<R::DomainModel, ConversionError> {
    request.to_domain()
  }

  /// Create response DTO from domain model
  pub fn create_response<R: ResponseDto>(
    &self,
    domain: &R::DomainModel,
    request_id: String,
  ) -> Result<ApiResponse<R>, ConversionError> {
    let response_dto = R::from_domain(domain)?;
    Ok(ApiResponse::success(response_dto, request_id))
  }

  /// Create paginated response
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

  /// Create batch operation response
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
              "Conversion failed".to_string(),
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

    // Register built-in validators
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
      // Initialize converter registry
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

/// Global DTOs manager instance
use std::sync::OnceLock;

static DTO_MANAGER: OnceLock<DtoManager> = OnceLock::new();

/// Get the global DTOs manager
pub fn get_dto_manager() -> &'static DtoManager {
  DTO_MANAGER.get_or_init(DtoManager::new)
}

/// Initialize the DTOs manager (custom configuration allowed)
pub fn init_dto_manager(manager: DtoManager) {
  // Only the first call to set will succeed; subsequent calls are ignored.
  let _ = DTO_MANAGER.set(manager);
}

/// Macro to define request DTOs
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
                  "Validation failed".to_string(),
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

/// Macro to define response DTOs
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
        // Response DTOs usually do not require validation
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

// === Example Usage (Documentation) ===
#[cfg(test)]
mod example_usage {
  use super::*;

  // Example of defining DTOs using the new macros:
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
