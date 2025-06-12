//! # Authentication and User Management Module
//!
//! **Responsibility**: User authentication, registration, login and user management
//!
//! Production-grade authentication service with multi-layer implementation:
//! - Basic authentication (AuthUserService)
//! - High availability features (HighAvailabilityAuthService)
//! - Full production features (ProductionAuthService)

pub mod service;

// Re-export core types for backward compatibility
pub use service::{
  AuthServiceBuilder,
  AuthServiceConfig,
  AuthServiceMetrics,
  // Basic service
  AuthUserService,
  // High availability service
  HighAvailabilityAuthService,

  // Production service
  ProductionAuthService,
  create_auth_user_service,

  create_custom_auth_service,
  create_production_auth_service,
};
