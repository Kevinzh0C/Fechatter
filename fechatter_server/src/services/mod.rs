//! # Services Layer - Main Entry Point
//!
//! **Design Philosophy**: Service layer architecture divided by responsibilities
//! **Layer**: Service Layer - Unified service provision layer

// =============================================================================
// APPLICATION SERVICES
// =============================================================================
pub mod application;

// =============================================================================
// INFRASTRUCTURE SERVICES
// =============================================================================
pub mod infrastructure;

// =============================================================================
// SERVICE PROVIDER
// =============================================================================
pub mod service_provider;

// =============================================================================
// AI SERVICES
// =============================================================================
pub mod ai;

// =============================================================================
// PUBLIC EXPORTS
// =============================================================================

// Re-export core services
pub use service_provider::{ServerTokenService, ServiceProvider};

// Re-export application services
pub use application::*;

// Re-export infrastructure services - focused by responsibility
pub use infrastructure::{EventPublisher, LocalStorage, RedisCacheService, SearchService};

// Re-export auth service (from application layer) 
pub use application::auth_app_service::AuthService;
